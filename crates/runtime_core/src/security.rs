#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    Read,
    Write,
    Net,
    Env,
    Run,
    Db,
    Dynamic,
    Wasm,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationCapability {
    pub op_id: &'static str,
    pub capability: Capability,
    pub notes: &'static str,
}

// Single source of truth for operation -> capability classification.
// New runtime operations must be added here first.
pub const OPERATION_CAPABILITY_MATRIX: &[OperationCapability] = &[
    // Filesystem bridge ops.
    OperationCapability {
        op_id: "bridge.fs.open",
        capability: Capability::Unknown,
        notes: "Mode-sensitive; classify as read/write at runtime from FsOpenRequest.mode",
    },
    OperationCapability {
        op_id: "bridge.fs.read",
        capability: Capability::Read,
        notes: "Read bytes from opened handle",
    },
    OperationCapability {
        op_id: "bridge.fs.write",
        capability: Capability::Write,
        notes: "Write bytes to opened handle",
    },
    OperationCapability {
        op_id: "bridge.fs.close",
        capability: Capability::Read,
        notes: "Handle cleanup only",
    },
    OperationCapability {
        op_id: "bridge.fs.read_file",
        capability: Capability::Read,
        notes: "Read full file path",
    },
    OperationCapability {
        op_id: "bridge.fs.write_file",
        capability: Capability::Write,
        notes: "Write full file path",
    },
    // Network bridge ops.
    OperationCapability {
        op_id: "bridge.net.connect",
        capability: Capability::Net,
        notes: "TCP outbound connect",
    },
    OperationCapability {
        op_id: "bridge.net.set_deadline",
        capability: Capability::Net,
        notes: "Socket deadline update",
    },
    OperationCapability {
        op_id: "bridge.net.read",
        capability: Capability::Net,
        notes: "Read from socket handle",
    },
    OperationCapability {
        op_id: "bridge.net.write",
        capability: Capability::Net,
        notes: "Write to socket handle",
    },
    OperationCapability {
        op_id: "bridge.net.tls_upgrade",
        capability: Capability::Net,
        notes: "Promote TCP socket to TLS",
    },
    OperationCapability {
        op_id: "bridge.net.close",
        capability: Capability::Net,
        notes: "Socket close",
    },
    // Database bridge ops. Engine-level allowlisting is a qualifier on this capability.
    OperationCapability {
        op_id: "bridge.db.open",
        capability: Capability::Db,
        notes: "Open DB handle; enforce engine qualifier postgres/mysql/sqlite",
    },
    OperationCapability {
        op_id: "bridge.db.query",
        capability: Capability::Db,
        notes: "Run query on DB handle",
    },
    OperationCapability {
        op_id: "bridge.db.query_one",
        capability: Capability::Db,
        notes: "Run single-row query on DB handle",
    },
    OperationCapability {
        op_id: "bridge.db.exec",
        capability: Capability::Db,
        notes: "Run exec statement on DB handle",
    },
    OperationCapability {
        op_id: "bridge.db.begin",
        capability: Capability::Db,
        notes: "Begin transaction",
    },
    OperationCapability {
        op_id: "bridge.db.commit",
        capability: Capability::Db,
        notes: "Commit transaction",
    },
    OperationCapability {
        op_id: "bridge.db.rollback",
        capability: Capability::Db,
        notes: "Rollback transaction",
    },
    OperationCapability {
        op_id: "bridge.db.close",
        capability: Capability::Db,
        notes: "Close DB handle",
    },
    OperationCapability {
        op_id: "bridge.db.stats",
        capability: Capability::Db,
        notes: "Read DB stats for active handles and metrics",
    },
    // Host/env operations used by php wasm bridge.
    OperationCapability {
        op_id: "php.op_php_read_env",
        capability: Capability::Env,
        notes: "Read process environment map",
    },
    OperationCapability {
        op_id: "php.op_php_read_file_sync",
        capability: Capability::Read,
        notes: "Read file from host FS",
    },
    OperationCapability {
        op_id: "php.op_php_cwd",
        capability: Capability::Read,
        notes: "Read current working directory",
    },
    OperationCapability {
        op_id: "php.op_php_file_exists",
        capability: Capability::Read,
        notes: "Check file existence",
    },
    // Dynamic execution.
    OperationCapability {
        op_id: "runtime.dynamic.eval",
        capability: Capability::Dynamic,
        notes: "String/code eval path",
    },
    OperationCapability {
        op_id: "runtime.dynamic.import",
        capability: Capability::Dynamic,
        notes: "Dynamic module import from runtime expression",
    },
    OperationCapability {
        op_id: "runtime.dynamic.fetch_exec",
        capability: Capability::Dynamic,
        notes: "Fetch and execute remote code",
    },
    // WASM loading/execution.
    OperationCapability {
        op_id: "runtime.wasm.import",
        capability: Capability::Wasm,
        notes: "import ... as wasm",
    },
    OperationCapability {
        op_id: "runtime.wasm.compile",
        capability: Capability::Wasm,
        notes: "WebAssembly.compile / module compilation",
    },
    OperationCapability {
        op_id: "runtime.wasm.instantiate",
        capability: Capability::Wasm,
        notes: "WebAssembly.instantiate",
    },
    OperationCapability {
        op_id: "runtime.wasm.host_import",
        capability: Capability::Wasm,
        notes: "Host capability surface exposed into wasm import object",
    },
    // Subprocess execution.
    OperationCapability {
        op_id: "runtime.process.spawn",
        capability: Capability::Run,
        notes: "Spawn subprocess",
    },
    OperationCapability {
        op_id: "runtime.process.exec",
        capability: Capability::Run,
        notes: "Exec subprocess",
    },
];

pub fn capability_for_operation(op_id: &str) -> Capability {
    OPERATION_CAPABILITY_MATRIX
        .iter()
        .find(|entry| entry.op_id == op_id)
        .map(|entry| entry.capability)
        .unwrap_or(Capability::Unknown)
}

#[cfg(test)]
mod tests {
    use super::{Capability, OPERATION_CAPABILITY_MATRIX, capability_for_operation};
    use std::collections::HashSet;

    #[test]
    fn unknown_when_not_registered() {
        assert_eq!(
            capability_for_operation("runtime.unmapped.operation"),
            Capability::Unknown
        );
    }

    #[test]
    fn matrix_has_unique_operation_ids() {
        let mut seen = HashSet::new();
        for entry in OPERATION_CAPABILITY_MATRIX {
            assert!(
                seen.insert(entry.op_id),
                "duplicate operation id in capability matrix: {}",
                entry.op_id
            );
        }
    }
}
