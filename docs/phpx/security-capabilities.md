# PHPX Security Capabilities (MVP)

`deka` supports a Deno-inspired capability model for runtime guard rails.

## Config Key

Project policy lives in `deka.json` under `deka.security`:

```json
{
  "deka.security": {
    "allow": {
      "read": ["./src"],
      "write": ["./.cache"],
      "net": ["api.linkhash.dev:443"],
      "env": ["DATABASE_URL"],
      "run": ["git"],
      "db": ["postgres"],
      "wasm": true,
      "dynamic": false
    },
    "deny": {
      "net": ["169.254.169.254"],
      "run": ["bash", "sh"],
      "dynamic": true
    },
    "prompt": true
  }
}
```

## Capability Groups

- `read`: filesystem reads
- `write`: filesystem writes
- `net`: network access
- `env`: environment variable access
- `run`: subprocess execution
- `db`: database operations
- `wasm`: wasm load/instantiate/host interop
- `dynamic`: dynamic execution paths (for example module eval fallback)

## CLI Flags

Allow flags:
- `--allow-read`
- `--allow-write`
- `--allow-net`
- `--allow-env`
- `--allow-run`
- `--allow-db`
- `--allow-wasm`
- `--allow-dynamic`
- `--allow-all`

Deny flags:
- `--deny-read`
- `--deny-write`
- `--deny-net`
- `--deny-env`
- `--deny-run`
- `--deny-db`
- `--deny-wasm`
- `--deny-dynamic`

Prompt control:
- `--no-prompt`

## Precedence

1. defaults
2. `deka.security` config
3. CLI allow flags
4. CLI deny flags

`deny` always wins when both allow and deny apply.

## Prompt Behavior

- If prompts are enabled and the process has a TTY, runtime may prompt for undeclared operations.
- `--no-prompt` disables prompts and forces deterministic deny.
- Non-TTY execution is treated as non-interactive deny.

## Subprocesses (`run`) and Privilege Escalation

Subprocess execution can bypass most in-process sandbox assumptions.

- `run` is blocked unless explicitly allowed by policy or CLI flag.
- Broad `--allow-run` emits a warning because child processes run with host privileges.
- Prefer allowlisting executables in `deka.security.allow.run` for safer operation.

## Dynamic Execution

Dynamic execution is treated as high risk and should remain disabled unless required.

- Dynamic module eval fallback is gated by `dynamic`.
- If disabled, runtime raises `SECURITY_DYNAMIC_EXEC_DENIED`.

## Registry + Install Guard Rails

- Linkhash publish preflight detects `run` and `dynamic` usage and requires explicit declaration.
- Install checks local `deka.security` policy and blocks packages that require denied capabilities.
