use std::collections::{btree_map::Entry, BTreeMap, VecDeque};
use std::path::{Component, Path};
use std::sync::{Arc, Mutex, Weak};

use crate::{
    DirEntry, ErrorCode, FileSystem, FileType, FsEvent, FsEventKind, FsWatcher, Metadata,
    MkdirOptions, MountTree, RemoveOptions, Result, WatchOptions, WosixError, WriteOptions,
};

#[derive(Debug, Default)]
pub struct InMemoryFileSystem {
    state: Mutex<FsState>,
}

impl InMemoryFileSystem {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug)]
struct FsState {
    root: Node,
    watchers: Vec<WatchRegistration>,
}

impl Default for FsState {
    fn default() -> Self {
        Self {
            root: Node::directory(),
            watchers: Vec::new(),
        }
    }
}

#[derive(Debug)]
struct Node {
    kind: NodeKind,
    _executable: bool,
}

#[derive(Debug)]
enum NodeKind {
    File(Vec<u8>),
    Directory(BTreeMap<String, Node>),
}

impl Node {
    fn directory() -> Self {
        Self {
            kind: NodeKind::Directory(BTreeMap::new()),
            _executable: false,
        }
    }

    fn file(data: Vec<u8>, executable: bool) -> Self {
        Self {
            kind: NodeKind::File(data),
            _executable: executable,
        }
    }

    fn file_type(&self) -> FileType {
        match self.kind {
            NodeKind::File(_) => FileType::File,
            NodeKind::Directory(_) => FileType::Directory,
        }
    }

    fn is_dir_empty(&self) -> bool {
        match &self.kind {
            NodeKind::Directory(children) => children.is_empty(),
            _ => true,
        }
    }
}

impl FileSystem for InMemoryFileSystem {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
        let components = path_components(path)?;
        let state = self.state.lock().unwrap();
        let node = get_node(&state.root, &components)?;
        match &node.kind {
            NodeKind::File(data) => Ok(data.clone()),
            _ => Err(invalid_input("path is not a file")),
        }
    }

    fn write_file(&self, path: &Path, data: &[u8], options: WriteOptions) -> Result<()> {
        let components = path_components(path)?;
        if components.is_empty() {
            return Err(invalid_input("cannot write to root"));
        }
        let mut state = self.state.lock().unwrap();
        let (parent, name) = get_parent_mut(&mut state.root, &components)?;
        let children = as_dir_mut(parent)?;
        let mut created = false;
        match children.get_mut(&name) {
            Some(node) => match &mut node.kind {
                NodeKind::File(contents) => {
                    if options.truncate {
                        contents.clear();
                    }
                    contents.extend_from_slice(data);
                }
                _ => return Err(invalid_input("path is not a file")),
            },
            None => {
                if !options.create {
                    return Err(not_found("file does not exist"));
                }
                children.insert(name, Node::file(data.to_vec(), false));
                created = true;
            }
        }
        emit_event_locked(
            &mut state,
            &components,
            if created {
                FsEventKind::Created
            } else {
                FsEventKind::Modified
            },
            None,
        );
        Ok(())
    }

    fn mkdir(&self, path: &Path, options: MkdirOptions) -> Result<()> {
        let components = path_components(path)?;
        if components.is_empty() {
            return Ok(());
        }
        let mut state = self.state.lock().unwrap();
        let mut current = &mut state.root;
        let mut created_last = false;
        for (index, component) in components.iter().enumerate() {
            let is_last = index + 1 == components.len();
            let children = as_dir_mut(current)?;
            match children.entry(component.clone()) {
                Entry::Occupied(entry) => {
                    if !matches!(entry.get().kind, NodeKind::Directory(_)) {
                        return Err(invalid_input("path exists and is not a directory"));
                    }
                    current = entry.into_mut();
                }
                Entry::Vacant(entry) => {
                    if !options.recursive && !is_last {
                        return Err(not_found("parent directory does not exist"));
                    }
                    current = entry.insert(Node::directory());
                    if is_last {
                        created_last = true;
                    }
                }
            };
        }
        if created_last {
            emit_event_locked(&mut state, &components, FsEventKind::Created, None);
        }
        Ok(())
    }

    fn readdir(&self, path: &Path) -> Result<Vec<DirEntry>> {
        let components = path_components(path)?;
        let state = self.state.lock().unwrap();
        let node = get_node(&state.root, &components)?;
        let children = match &node.kind {
            NodeKind::Directory(children) => children,
            _ => return Err(invalid_input("path is not a directory")),
        };
        Ok(children
            .iter()
            .map(|(name, child)| DirEntry {
                name: name.clone(),
                file_type: child.file_type(),
            })
            .collect())
    }

    fn stat(&self, path: &Path) -> Result<Metadata> {
        let components = path_components(path)?;
        let state = self.state.lock().unwrap();
        let node = get_node(&state.root, &components)?;
        let size = match &node.kind {
            NodeKind::File(data) => data.len() as u64,
            NodeKind::Directory(_) => 0,
        };
        Ok(Metadata {
            file_type: node.file_type(),
            size,
            created: None,
            modified: None,
            accessed: None,
        })
    }

    fn remove(&self, path: &Path, options: RemoveOptions) -> Result<()> {
        let components = path_components(path)?;
        if components.is_empty() {
            return Err(invalid_input("cannot remove root"));
        }
        let mut state = self.state.lock().unwrap();
        let (parent, name) = get_parent_mut(&mut state.root, &components)?;
        let children = as_dir_mut(parent)?;
        let node = match children.get(&name) {
            Some(node) => node,
            None => {
                if options.force {
                    return Ok(());
                }
                return Err(not_found("path does not exist"));
            }
        };
        if matches!(node.kind, NodeKind::Directory(_))
            && !options.recursive
            && !node.is_dir_empty()
        {
            return Err(invalid_input("directory is not empty"));
        }
        children.remove(&name);
        emit_event_locked(&mut state, &components, FsEventKind::Removed, None);
        Ok(())
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        let from_components = path_components(from)?;
        let to_components = path_components(to)?;
        if from_components.is_empty() || to_components.is_empty() {
            return Err(invalid_input("cannot rename root"));
        }
        if from_components == to_components {
            return Ok(());
        }
        if to_components.starts_with(&from_components) {
            return Err(invalid_input("cannot move a path into itself"));
        }
        let mut state = self.state.lock().unwrap();
        let _ = get_parent(&state.root, &to_components)?;
        let node = {
            let (parent, name) = get_parent_mut(&mut state.root, &from_components)?;
            let children = as_dir_mut(parent)?;
            children
                .remove(&name)
                .ok_or_else(|| not_found("source path does not exist"))?
        };
        {
            let (parent, name) = get_parent_mut(&mut state.root, &to_components)?;
            let children = as_dir_mut(parent)?;
            if let Some(existing) = children.get(&name) {
                if matches!(existing.kind, NodeKind::Directory(_)) && !existing.is_dir_empty() {
                    return Err(invalid_input("destination directory is not empty"));
                }
                children.remove(&name);
            }
            children.insert(name, node);
        }
        emit_event_locked(
            &mut state,
            &from_components,
            FsEventKind::Renamed,
            Some(components_to_path(&to_components)),
        );
        Ok(())
    }

    fn mount_tree(&self, path: &Path, tree: MountTree) -> Result<()> {
        let components = path_components(path)?;
        let mut state = self.state.lock().unwrap();
        if components.is_empty() {
            if let MountTree::Directory(_) = tree {
                state.root = node_from_tree(tree);
                return Ok(());
            }
            return Err(invalid_input("cannot mount a file at root"));
        }
        let (parent, name) = ensure_parent(&mut state.root, &components)?;
        let children = as_dir_mut(parent)?;
        children.insert(name, node_from_tree(tree));
        emit_event_locked(&mut state, &components, FsEventKind::Created, None);
        Ok(())
    }

    fn watch(&self, path: &Path, options: WatchOptions) -> Result<Box<dyn FsWatcher>> {
        let components = path_components(path)?;
        let mut state = self.state.lock().unwrap();
        prune_watchers(&mut state);
        let queue = Arc::new(Mutex::new(WatchQueue::default()));
        let entry = WatchRegistration {
            path: components,
            recursive: options.recursive,
            queue: Arc::downgrade(&queue),
        };
        state.watchers.push(entry);
        Ok(Box::new(InMemoryWatcher { queue }))
    }
}

#[derive(Debug, Default)]
struct WatchQueue {
    events: VecDeque<FsEvent>,
    closed: bool,
}

#[derive(Debug)]
struct WatchRegistration {
    path: Vec<String>,
    recursive: bool,
    queue: Weak<Mutex<WatchQueue>>,
}

struct InMemoryWatcher {
    queue: Arc<Mutex<WatchQueue>>,
}

impl FsWatcher for InMemoryWatcher {
    fn next_event(&mut self) -> Result<Option<FsEvent>> {
        let mut queue = self.queue.lock().unwrap();
        if queue.closed {
            return Ok(None);
        }
        Ok(queue.events.pop_front())
    }

    fn close(&mut self) -> Result<()> {
        let mut queue = self.queue.lock().unwrap();
        queue.closed = true;
        queue.events.clear();
        Ok(())
    }
}

fn node_from_tree(tree: MountTree) -> Node {
    match tree {
        MountTree::File(file) => Node::file(file.data, file.executable),
        MountTree::Directory(entries) => {
            let mut children = BTreeMap::new();
            for (name, entry) in entries {
                children.insert(name, node_from_tree(entry));
            }
            Node {
                kind: NodeKind::Directory(children),
                _executable: false,
            }
        }
    }
}

fn emit_event_locked(
    state: &mut FsState,
    components: &[String],
    kind: FsEventKind,
    target_path: Option<String>,
) {
    let event = FsEvent {
        path: components_to_path(components),
        kind,
        target_path,
    };
    state.watchers.retain(|entry| {
        let queue = match entry.queue.upgrade() {
            Some(queue) => queue,
            None => return false,
        };
        let mut queue = queue.lock().unwrap();
        if queue.closed {
            return false;
        }
        if path_matches(&entry.path, components, entry.recursive) {
            queue.events.push_back(event.clone());
        }
        true
    });
}

fn prune_watchers(state: &mut FsState) {
    state.watchers.retain(|entry| {
        if let Some(queue) = entry.queue.upgrade() {
            let queue = queue.lock().unwrap();
            !queue.closed
        } else {
            false
        }
    });
}

fn path_matches(watch: &[String], event: &[String], recursive: bool) -> bool {
    if watch.is_empty() {
        return true;
    }
    if recursive {
        return event.starts_with(watch);
    }
    if watch == event {
        return true;
    }
    event.starts_with(watch) && event.len() == watch.len() + 1
}

fn components_to_path(components: &[String]) -> String {
    if components.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", components.join("/"))
    }
}

fn ensure_parent<'a>(root: &'a mut Node, components: &[String]) -> Result<(&'a mut Node, String)> {
    if components.is_empty() {
        return Err(invalid_input("path is empty"));
    }
    let name = components.last().unwrap().clone();
    let parent_components = &components[..components.len() - 1];
    let parent = ensure_dir(root, parent_components)?;
    Ok((parent, name))
}

fn ensure_dir<'a>(node: &'a mut Node, components: &[String]) -> Result<&'a mut Node> {
    if components.is_empty() {
        return Ok(node);
    }
    let children = as_dir_mut(node)?;
    let entry = children
        .entry(components[0].clone())
        .or_insert_with(Node::directory);
    ensure_dir(entry, &components[1..])
}

fn get_parent<'a>(root: &'a Node, components: &[String]) -> Result<(&'a Node, String)> {
    if components.is_empty() {
        return Err(invalid_input("path is empty"));
    }
    let name = components.last().unwrap().clone();
    let parent_components = &components[..components.len() - 1];
    let parent = get_node(root, parent_components)?;
    Ok((parent, name))
}

fn get_parent_mut<'a>(
    root: &'a mut Node,
    components: &[String],
) -> Result<(&'a mut Node, String)> {
    if components.is_empty() {
        return Err(invalid_input("path is empty"));
    }
    let name = components.last().unwrap().clone();
    let parent_components = &components[..components.len() - 1];
    let parent = get_node_mut(root, parent_components)?;
    Ok((parent, name))
}

fn get_node<'a>(node: &'a Node, components: &[String]) -> Result<&'a Node> {
    if components.is_empty() {
        return Ok(node);
    }
    match &node.kind {
        NodeKind::Directory(children) => {
            let child = children
                .get(&components[0])
                .ok_or_else(|| not_found("path does not exist"))?;
            get_node(child, &components[1..])
        }
        _ => Err(invalid_input("path is not a directory")),
    }
}

fn get_node_mut<'a>(node: &'a mut Node, components: &[String]) -> Result<&'a mut Node> {
    if components.is_empty() {
        return Ok(node);
    }
    match &mut node.kind {
        NodeKind::Directory(children) => {
            let child = children
                .get_mut(&components[0])
                .ok_or_else(|| not_found("path does not exist"))?;
            get_node_mut(child, &components[1..])
        }
        _ => Err(invalid_input("path is not a directory")),
    }
}

fn as_dir_mut(node: &mut Node) -> Result<&mut BTreeMap<String, Node>> {
    match &mut node.kind {
        NodeKind::Directory(children) => Ok(children),
        _ => Err(invalid_input("path is not a directory")),
    }
}

fn path_components(path: &Path) -> Result<Vec<String>> {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(segment) => {
                let segment = segment
                    .to_str()
                    .ok_or_else(|| invalid_input("path must be valid utf-8"))?;
                if !segment.is_empty() {
                    components.push(segment.to_string());
                }
            }
            Component::RootDir | Component::CurDir => {}
            Component::ParentDir => {
                return Err(invalid_input("parent directory segments are not supported"))
            }
            Component::Prefix(_) => return Err(invalid_input("path prefix is not supported")),
        }
    }
    Ok(components)
}

fn invalid_input(message: &str) -> WosixError {
    WosixError::new(ErrorCode::InvalidInput, message)
}

fn not_found(message: &str) -> WosixError {
    WosixError::new(ErrorCode::NotFound, message)
}

#[cfg(test)]
mod tests {
    use super::InMemoryFileSystem;
    use crate::{
        ErrorCode, FileSystem, FsEventKind, MkdirOptions, MountFile, MountTree, RemoveOptions,
        WatchOptions, WriteOptions,
    };
    use std::path::Path;

    #[test]
    fn write_and_read_file() {
        let fs = InMemoryFileSystem::new();
        fs.mkdir(Path::new("/tmp"), MkdirOptions::default())
            .unwrap();
        fs.write_file(
            Path::new("/tmp/hello.txt"),
            b"hi",
            WriteOptions::default(),
        )
        .unwrap();
        let data = fs.read_file(Path::new("/tmp/hello.txt")).unwrap();
        assert_eq!(data, b"hi");
    }

    #[test]
    fn remove_directory_requires_recursive() {
        let fs = InMemoryFileSystem::new();
        fs.mkdir(Path::new("/root/child"), MkdirOptions { recursive: true, mode: None })
            .unwrap();
        let err = fs
            .remove(
                Path::new("/root"),
                RemoveOptions {
                    recursive: false,
                    force: false,
                },
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidInput);
        fs.remove(
            Path::new("/root"),
            RemoveOptions {
                recursive: true,
                force: false,
            },
        )
        .unwrap();
    }

    #[test]
    fn rename_rejects_self_parent() {
        let fs = InMemoryFileSystem::new();
        fs.mkdir(Path::new("/data/inner"), MkdirOptions { recursive: true, mode: None })
            .unwrap();
        let err = fs
            .rename(Path::new("/data"), Path::new("/data/inner/sub"))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidInput);
    }

    #[test]
    fn mount_tree_populates_entries() {
        let fs = InMemoryFileSystem::new();
        let tree = MountTree::Directory(
            [(
                "app".to_string(),
                MountTree::File(MountFile {
                    data: b"ok".to_vec(),
                    executable: false,
                }),
            )]
            .into_iter()
            .collect(),
        );
        fs.mount_tree(Path::new("/"), tree).unwrap();
        let data = fs.read_file(Path::new("/app")).unwrap();
        assert_eq!(data, b"ok");
    }

    #[test]
    fn watch_emits_events() {
        let fs = InMemoryFileSystem::new();
        let mut watcher = fs
            .watch(Path::new("/"), WatchOptions::default())
            .unwrap();
        fs.write_file(Path::new("/note.txt"), b"one", WriteOptions::default())
            .unwrap();
        let event = watcher.next_event().unwrap().unwrap();
        assert_eq!(event.kind, FsEventKind::Created);
        assert_eq!(event.path, "/note.txt");

        fs.rename(Path::new("/note.txt"), Path::new("/note-2.txt"))
            .unwrap();
        let event = watcher.next_event().unwrap().unwrap();
        assert_eq!(event.kind, FsEventKind::Renamed);
        assert_eq!(event.path, "/note.txt");
        assert_eq!(event.target_path.as_deref(), Some("/note-2.txt"));
    }
}
