use core::{CommandSpec, Context, Registry};
use deno_task_shell::ExecutableCommand;
use deno_task_shell::KillSignal;
use deno_task_shell::ShellCommand;
use deno_task_shell::ShellPipeWriter;
use deno_task_shell::ShellState;
use deno_task_shell::execute_with_pipes;
use deno_task_shell::parser::parse;
use deno_task_shell::pipe;
use serde_json::Value;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use stdio::{raw, warn_simple};

const COMMAND: CommandSpec = CommandSpec {
    name: "task",
    category: "runtime",
    summary: "run a task from deka.json",
    aliases: &[],
    subcommands: &[],
    handler: cmd,
};

pub fn register(registry: &mut Registry) {
    registry.add_command(COMMAND);
}

pub fn cmd(context: &Context) {
    let cwd = std::env::current_dir().ok();
    let Some(cwd) = cwd else {
        stdio::error("task", "failed to resolve current directory");
        return;
    };
    let Some((project_root, json)) = load_deka_json(&cwd) else {
        stdio::error("task", "deka.json not found (searched parent directories)");
        return;
    };

    let (tasks, used_scripts) = extract_tasks(&json);
    if used_scripts {
        warn_simple("scripts is deprecated for tasks; prefer tasks in deka.json");
    }

    let task_name = requested_task_name(context);
    if task_name.is_none() {
        print_task_list(&tasks);
        return;
    }

    let task_name = task_name.unwrap();
    if !tasks.contains_key(task_name) {
        stdio::error("task", &format!("unknown task `{}`", task_name));
        print_task_list(&tasks);
        return;
    }

    let task = tasks.get(task_name).expect("task exists");
    if let Err(message) = run_task(task_name, task, &project_root, &cwd) {
        stdio::error("task", &message);
    }
}

fn requested_task_name<'a>(context: &'a Context) -> Option<&'a str> {
    if let Some(first) = context.args.positionals.first() {
        return Some(first.as_str());
    }
    if context.args.commands.len() > 1 {
        return Some(context.args.commands[1].as_str());
    }
    None
}

#[derive(Debug, Clone)]
struct TaskDef {
    command: String,
    description: Option<String>,
    dependencies: Vec<String>,
}

fn load_deka_json(start: &Path) -> Option<(PathBuf, Value)> {
    for dir in start.ancestors() {
        let path = dir.join("deka.json");
        if !path.is_file() {
            continue;
        }
        let content = std::fs::read_to_string(&path).ok()?;
        let json = serde_json::from_str(&content).ok()?;
        return Some((dir.to_path_buf(), json));
    }
    None
}

fn extract_tasks(json: &Value) -> (BTreeMap<String, TaskDef>, bool) {
    let tasks = json
        .get("tasks")
        .and_then(|value| value.as_object());
    let (source, used_scripts) = if tasks.is_some() {
        (tasks, false)
    } else {
        (
            json.get("scripts").and_then(|value| value.as_object()),
            true,
        )
    };

    let mut out = BTreeMap::new();
    let Some(source) = source else {
        return (out, used_scripts);
    };

    for (name, value) in source {
        if let Some(command) = value.as_str() {
            out.insert(
                name.to_string(),
                TaskDef {
                    command: command.to_string(),
                    description: None,
                    dependencies: Vec::new(),
                },
            );
            continue;
        }

        let Some(obj) = value.as_object() else {
            warn_simple(&format!(
                "task `{}` ignored: expected string or object",
                name
            ));
            continue;
        };

        let command = obj
            .get("command")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let Some(command) = command else {
            warn_simple(&format!(
                "task `{}` ignored: missing required `command`",
                name
            ));
            continue;
        };

        let description = obj
            .get("description")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let dependencies = obj
            .get("dependencies")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(|value| value.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        out.insert(
            name.to_string(),
            TaskDef {
                command,
                description,
                dependencies,
            },
        );
    }

    (out, used_scripts)
}

fn print_task_list(tasks: &BTreeMap<String, TaskDef>) {
    if tasks.is_empty() {
        stdio::error("task", "no tasks found in deka.json");
        return;
    }

    raw("Tasks:");
    for (name, task) in tasks {
        let mut line = String::new();
        line.push_str("  ");
        line.push_str(name);
        if let Some(desc) = task.description.as_deref() {
            line.push_str("  - ");
            line.push_str(desc);
        }
        if !task.dependencies.is_empty() {
            line.push_str("  (deps: ");
            line.push_str(&task.dependencies.join(", "));
            line.push(')');
        }
        if !task.command.is_empty() {
            line.push_str("  -> ");
            line.push_str(&task.command);
        }
        raw(&line);
    }
}

fn run_task(
    name: &str,
    task: &TaskDef,
    project_root: &Path,
    init_cwd: &Path,
) -> Result<(), String> {
    if task.command.trim().is_empty() {
        return Err(format!("task `{}` has empty command", name));
    }
    if !task.dependencies.is_empty() {
        return Err(format!(
            "task `{}` has dependencies which are not supported yet",
            name
        ));
    }

    let list = parse(&task.command).map_err(|err| err.to_string())?;
    let mut env_vars = HashMap::new();
    for (key, value) in std::env::vars_os() {
        env_vars.insert(key, value);
    }
    env_vars.insert(OsString::from("INIT_CWD"), OsString::from(init_cwd));

    let deka_exe = std::env::current_exe()
        .map_err(|err| format!("failed to resolve deka executable: {}", err))?;
    let mut custom_commands: HashMap<String, Rc<dyn ShellCommand>> = HashMap::new();
    custom_commands.insert(
        "deka".to_string(),
        Rc::new(ExecutableCommand::new("deka".to_string(), deka_exe)),
    );

    let kill_signal = KillSignal::default();
    let (stdin, stdin_writer) = pipe();
    drop(stdin_writer);
    let stdout = ShellPipeWriter::stdout();
    let stderr = ShellPipeWriter::stderr();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| format!("failed to start task runtime: {}", err))?;
    let exit_code = runtime.block_on(async move {
        let local = tokio::task::LocalSet::new();
        let state = ShellState::new(
            env_vars,
            project_root.to_path_buf(),
            custom_commands,
            kill_signal,
        );
        local
            .run_until(execute_with_pipes(list, state, stdin, stdout, stderr))
            .await
    });

    if exit_code == 0 {
        Ok(())
    } else {
        Err(format!("task `{}` failed with exit code {}", name, exit_code))
    }
}
