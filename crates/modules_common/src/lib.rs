use std::path::{Path, PathBuf};
use std::sync::Arc;

use deno_core::Extension;
use deno_permissions::{
    AllowRunDescriptor, AllowRunDescriptorParseResult, DenyRunDescriptor, EnvDescriptor,
    FfiDescriptor, ImportDescriptor, NetDescriptor, PathQueryDescriptor, PathResolveError,
    PermissionDescriptorParser, PermissionsContainer, ReadDescriptor, RunDescriptorParseError,
    RunQueryDescriptor, SysDescriptor, WriteDescriptor,
};

#[derive(Debug, Clone)]
struct DekaPermissionDescriptorParser {
    cwd: PathBuf,
}

impl DekaPermissionDescriptorParser {
    fn new() -> Result<Self, PathResolveError> {
        let cwd = std::env::current_dir().map_err(PathResolveError::CwdResolve)?;
        Ok(Self { cwd })
    }

    fn resolve_path(&self, text: &str) -> Result<PathBuf, PathResolveError> {
        if text.is_empty() {
            return Err(PathResolveError::EmptyPath);
        }
        let path = PathBuf::from(text);
        if path.is_absolute() {
            Ok(path)
        } else {
            Ok(self.cwd.join(path))
        }
    }
}

impl PermissionDescriptorParser for DekaPermissionDescriptorParser {
    fn parse_read_descriptor(&self, text: &str) -> Result<ReadDescriptor, PathResolveError> {
        Ok(ReadDescriptor(self.resolve_path(text)?))
    }

    fn parse_write_descriptor(&self, text: &str) -> Result<WriteDescriptor, PathResolveError> {
        Ok(WriteDescriptor(self.resolve_path(text)?))
    }

    fn parse_net_descriptor(
        &self,
        text: &str,
    ) -> Result<NetDescriptor, deno_permissions::NetDescriptorParseError> {
        NetDescriptor::parse(text)
    }

    fn parse_import_descriptor(
        &self,
        text: &str,
    ) -> Result<ImportDescriptor, deno_permissions::NetDescriptorParseError> {
        ImportDescriptor::parse(text)
    }

    fn parse_env_descriptor(
        &self,
        text: &str,
    ) -> Result<EnvDescriptor, deno_permissions::EnvDescriptorParseError> {
        Ok(EnvDescriptor::new(text))
    }

    fn parse_sys_descriptor(
        &self,
        text: &str,
    ) -> Result<SysDescriptor, deno_permissions::SysDescriptorParseError> {
        SysDescriptor::parse(text.to_string())
    }

    fn parse_allow_run_descriptor(
        &self,
        text: &str,
    ) -> Result<AllowRunDescriptorParseResult, RunDescriptorParseError> {
        Ok(AllowRunDescriptorParseResult::Descriptor(
            AllowRunDescriptor(self.resolve_path(text)?),
        ))
    }

    fn parse_deny_run_descriptor(&self, text: &str) -> Result<DenyRunDescriptor, PathResolveError> {
        if text.contains(std::path::MAIN_SEPARATOR) || Path::new(text).is_absolute() {
            Ok(DenyRunDescriptor::Path(self.resolve_path(text)?))
        } else {
            Ok(DenyRunDescriptor::Name(text.to_string()))
        }
    }

    fn parse_ffi_descriptor(&self, text: &str) -> Result<FfiDescriptor, PathResolveError> {
        Ok(FfiDescriptor(self.resolve_path(text)?))
    }

    fn parse_path_query(&self, path: &str) -> Result<PathQueryDescriptor, PathResolveError> {
        Ok(PathQueryDescriptor {
            resolved: self.resolve_path(path)?,
            requested: path.to_string(),
        })
    }

    fn parse_run_query(
        &self,
        requested: &str,
    ) -> Result<RunQueryDescriptor, RunDescriptorParseError> {
        RunQueryDescriptor::parse(requested).map_err(Into::into)
    }
}

pub fn permissions_extension() -> Extension {
    Extension {
        name: "deka_permissions",
        op_state_fn: Some(Box::new(|state| {
            let parser = DekaPermissionDescriptorParser::new().unwrap_or_else(|_| {
                DekaPermissionDescriptorParser {
                    cwd: PathBuf::from("/"),
                }
            });
            let container = PermissionsContainer::allow_all(Arc::new(parser));
            state.put(container);
        })),
        ..Default::default()
    }
}
