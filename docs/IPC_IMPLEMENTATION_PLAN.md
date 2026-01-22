# IPC (Inter-Process Communication) Implementation Plan

## Overview

To make deka a true drop-in replacement for Node.js, we need to implement bidirectional IPC between parent and child processes. This is currently blocking Next.js from running natively in the deka runtime.

## Current State

### What Works ✓
- ✓ Synchronous process spawning (`op_process_spawn_immediate`)
- ✓ PID available immediately after fork
- ✓ `stdio: 'inherit'` support
- ✓ Environment variable passing
- ✓ Fork delegates to Node.js (temporary workaround)

### What's Missing ✗
- ✗ IPC channel setup (stdio[3])
- ✗ `child.send(message)` - Parent to child messaging
- ✗ `child.on('message', callback)` - Child to parent messaging
- ✗ Message serialization/deserialization
- ✗ Structured clone algorithm for message passing
- ✗ Handle transfer (for passing sockets, servers, etc.)

## How Node.js IPC Works

### IPC Channel
When Node.js forks a child with `child_process.fork()`, it:
1. Creates an extra stdio channel at **file descriptor 3** (stdio[3])
2. stdio[0] = stdin, stdio[1] = stdout, stdio[2] = stderr, **stdio[3] = ipc**
3. Uses this channel for bidirectional JSON message passing
4. Both parent and child can send/receive on this channel

### Message Format
Messages are JSON-serialized with a specific protocol:
```javascript
{
  cmd: 'NODE_...',  // Internal command type (optional)
  // ... message data ...
}
```

User messages are sent as:
```javascript
{
  cmd: 'NODE_HANDLE',  // or no cmd field for simple messages
  msg: <actual user data>
}
```

### API Surface

**Parent Process:**
```javascript
const child = fork('./worker.js');

// Send message to child
child.send({ hello: 'world' });

// Receive message from child
child.on('message', (msg) => {
  console.log('Parent received:', msg);
});
```

**Child Process:**
```javascript
// Send message to parent
process.send({ hello: 'parent' });

// Receive message from parent
process.on('message', (msg) => {
  console.log('Child received:', msg);
});
```

## Architecture Design

### Rust Side (process.rs)

#### New Structures
```rust
// IPC channel handle
struct IpcChannel {
    reader: AsyncMutex<ChildStdout>,  // stdio[3] for reading
    writer: AsyncMutex<ChildStdin>,   // stdio[3] for writing
}

// Updated ChildProcessEntry
struct ChildProcessEntry {
    child: Option<Child>,
    stdout: Option<ChildStdout>,
    stderr: Option<ChildStderr>,
    stdin: Option<ChildStdin>,
    ipc: Option<IpcChannel>,  // NEW
}
```

#### New Operations

**1. op_process_spawn_immediate with IPC**
```rust
#[op2]
#[bigint]
pub(super) fn op_process_spawn_immediate(
    // ... existing params ...
    #[serde] enable_ipc: bool,  // NEW
) -> Result<u64, CoreError> {
    let mut cmd = Command::new(command);

    if enable_ipc {
        // Set stdio to [inherit/pipe, inherit/pipe, inherit/pipe, pipe]
        // stdio[3] = pipe for IPC channel
        use std::process::Stdio;
        use std::os::unix::process::CommandExt;

        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        // TODO: Add 4th pipe for IPC (fd 3)
    }

    // ... spawn and store with IPC channel ...
}
```

**2. op_process_send_message**
```rust
#[op2(async)]
pub(super) async fn op_process_send_message(
    #[bigint] id: u64,
    #[string] message: String,  // JSON-serialized message
) -> Result<(), CoreError> {
    // Get the IPC channel for this process
    // Write message + newline delimiter
    // Flush the writer
}
```

**3. op_process_read_message**
```rust
#[op2(async)]
#[string]
pub(super) async fn op_process_read_message(
    #[bigint] id: u64,
) -> Result<Option<String>, CoreError> {
    // Read from IPC channel until newline
    // Return the JSON message string
    // Return None on EOF
}
```

### JavaScript Side (deka.js)

#### ChildProcess Updates

```javascript
class ChildProcess extends EventEmitter {
    pid;
    stdout;
    stderr;
    stdin;
    stdio;
    channel;      // NEW: IPC channel (if enabled)
    connected;    // NEW: IPC connection state

    constructor(pidPromise, options) {
        super();
        this.connected = options?.ipc || false;
        // ...
    }

    send(message, sendHandle, options, callback) {
        if (!this.connected) {
            throw new Error('channel closed');
        }

        // Normalize arguments (Node.js supports multiple signatures)
        if (typeof sendHandle === 'function') {
            callback = sendHandle;
            sendHandle = undefined;
            options = undefined;
        }

        // Serialize message
        const serialized = JSON.stringify({
            cmd: 'NODE_HANDLE',
            msg: message
        });

        // Send via IPC channel
        op_process_send_message(this.pid, serialized)
            .then(() => {
                if (callback) callback(null);
            })
            .catch((err) => {
                if (callback) callback(err);
                else this.emit('error', err);
            });

        return true;
    }

    disconnect() {
        if (!this.connected) return;
        this.connected = false;
        this.emit('disconnect');
        // Close IPC channel
    }

    attachIpcReader() {
        if (!this.connected || !this.pid) return;

        const readLoop = async () => {
            while (this.connected) {
                try {
                    const msg = await op_process_read_message(this.pid);
                    if (msg === null) {
                        // EOF - channel closed
                        this.disconnect();
                        break;
                    }

                    // Deserialize and emit
                    const parsed = JSON.parse(msg);
                    const userMsg = parsed.msg !== undefined ? parsed.msg : parsed;
                    this.emit('message', userMsg);
                } catch (err) {
                    this.emit('error', err);
                    break;
                }
            }
        };

        readLoop();
    }
}
```

#### fork() Updates

```javascript
function fork(modulePath, args = [], options) {
    // ... existing setup ...

    // Enable IPC by default for fork (like Node.js)
    const ipcOptions = {
        ...options,
        ipc: true,
        stdio: options?.stdio || ['pipe', 'pipe', 'pipe', 'ipc']
    };

    const child = new ChildProcess(undefined, ipcOptions);

    // ... spawn with IPC enabled ...

    return child;
}
```

#### process.send() for Child Process

```javascript
// In the child process (running in isolate)
globalThis.process.send = function(message, sendHandle, options, callback) {
    if (!globalThis.process.connected) {
        throw new Error('channel closed');
    }

    // Same implementation as ChildProcess.send
    // but uses a special "to parent" IPC channel
};

globalThis.process.connected = true;  // If forked with IPC
globalThis.process.channel = { ... };  // IPC channel reference

globalThis.process.disconnect = function() {
    globalThis.process.connected = false;
    globalThis.process.emit('disconnect');
};
```

## Implementation Phases

### Phase 1: Basic IPC Channel Setup ✓ (Start Here)
**Goal:** Get stdio[3] pipe working for one-way messaging

**Tasks:**
1. ✓ Research how to add a 4th stdio pipe in Rust tokio::process::Command
2. ✓ Update `op_process_spawn_immediate` to create stdio[3] when `enable_ipc: true`
3. ✓ Store IPC reader/writer in ChildProcessEntry
4. ✓ Implement `op_process_send_message` (parent → child)
5. ✓ Implement `op_process_read_message` (parent ← child)
6. ✓ Test one-way messaging with a simple worker script

**Success Criteria:**
- Parent can send JSON message to child
- Child can read message from stdin/fd 3
- No crashes, clean error handling

### Phase 2: Bidirectional Messaging
**Goal:** Enable full two-way communication

**Tasks:**
1. ✓ Implement child-side IPC channel detection (check if fd 3 exists)
2. ✓ Add `process.send()` to child process global scope
3. ✓ Add `process.on('message')` handler in child
4. ✓ Implement message read loop in parent (ChildProcess.attachIpcReader)
5. ✓ Test round-trip messaging (parent ↔ child)

**Success Criteria:**
- Parent sends → Child receives and responds → Parent receives
- Event emitters working correctly
- Message ordering preserved

### Phase 3: ChildProcess.send() API
**Goal:** Full Node.js-compatible API

**Tasks:**
1. ✓ Implement `child.send(message, callback)`
2. ✓ Handle multiple callback signatures (Node.js compatibility)
3. ✓ Implement `child.disconnect()` / `child.connected`
4. ✓ Emit 'disconnect' events
5. ✓ Error handling and edge cases

**Success Criteria:**
- All send() signatures work
- Disconnect properly closes channels
- Errors propagated correctly

### Phase 4: Message Serialization & Advanced Features
**Goal:** Handle complex data types

**Tasks:**
1. ⬜ Implement structured clone algorithm (or use V8's built-in)
2. ⬜ Support circular references
3. ⬜ Handle special types (Buffer, typed arrays, Error objects)
4. ⬜ Implement handle transfer (optional - advanced)

**Success Criteria:**
- Can send complex objects
- Buffers serialized correctly
- Handles match Node.js behavior

### Phase 5: Next.js Integration
**Goal:** Remove the Node.js delegation workaround

**Tasks:**
1. ⬜ Remove Next.js detection from `run.rs`
2. ⬜ Update fork() to enable IPC by default
3. ⬜ Test Next.js dev server with native deka runtime
4. ⬜ Verify all Next.js IPC messages work (worker ready, server ready, etc.)

**Success Criteria:**
- `deka run --deka dev` starts Next.js natively
- No delegation to Node.js
- Full feature parity with `next dev`

## Platform-Specific Considerations

### Unix/Linux/macOS
- Use `std::os::unix::process::CommandExt` to set up fd 3
- Pipes work natively

### Windows
- Windows doesn't use file descriptors the same way
- May need `std::os::windows::process::CommandExt`
- Consider named pipes or other IPC mechanisms
- Defer Windows support to later phase if needed

## Test Cases

### Unit Tests
```javascript
// test/ipc/basic-send.js
const { fork } = require('child_process');
const child = fork('./worker.js');

child.on('message', (msg) => {
  console.assert(msg.result === 42);
  child.kill();
});

child.send({ compute: 'meaning of life' });
```

```javascript
// test/ipc/worker.js
process.on('message', (msg) => {
  if (msg.compute) {
    process.send({ result: 42 });
  }
});
```

### Integration Tests
- Test with actual Next.js dev server
- Test with PM2-style process manager
- Test message ordering under load
- Test disconnect/reconnect scenarios

## Success Metrics

**Phase 1 Complete:** One-way IPC working (1-2 days)
**Phase 2 Complete:** Bidirectional IPC working (1-2 days)
**Phase 3 Complete:** Full API parity (1 day)
**Phase 4 Complete:** Advanced features (2-3 days)
**Phase 5 Complete:** Next.js working natively (1 day testing)

**Total Estimate:** 6-10 days of focused development

## Open Questions

1. **Structured Clone:** Use V8's built-in or implement our own?
2. **Handle Transfer:** Do we need this for Next.js? (Probably not initially)
3. **Windows Support:** Defer or implement alongside Unix?
4. **Performance:** Should messages be buffered? What's the optimal buffer size?
5. **Child Process ID:** How to detect if running as forked child? (Check fd 3 existence?)

## References

- [Node.js child_process.fork() docs](https://nodejs.org/api/child_process.html#child_processforkmodulepath-args-options)
- [Node.js IPC channel implementation (C++)](https://github.com/nodejs/node/blob/main/src/process_wrap.cc)
- [IPC message protocol](https://github.com/nodejs/node/blob/main/lib/internal/child_process.js)
- [Structured Clone Algorithm](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Structured_clone_algorithm)

## Next Steps

1. ✅ Commit current work with this plan
2. **Start Phase 1:** Research and implement stdio[3] pipe in Rust
3. Create a minimal test case (parent sends message, child logs it)
4. Iterate on implementation phases
5. Remove Next.js delegation once IPC is working

---

**Status:** Planning Complete - Ready to Begin Implementation
**Priority:** High (blocks Next.js native support)
**Complexity:** Medium-High (new subsystem but well-defined)
