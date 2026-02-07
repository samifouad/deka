use deno_core::{error::CoreError, op2};
use serde::Serialize;
use std::time::Duration;
use tokio::sync::{Mutex, mpsc};

#[derive(Serialize)]
struct StdinRead {
    data: Vec<u8>,
    eof: bool,
}

enum StdinMessage {
    Data(Vec<u8>),
    Eof,
}

struct StdinChannel {
    receiver: Mutex<mpsc::UnboundedReceiver<StdinMessage>>,
}

static STDIN_CHANNEL: std::sync::OnceLock<StdinChannel> = std::sync::OnceLock::new();

fn stdin_channel() -> &'static StdinChannel {
    STDIN_CHANNEL.get_or_init(|| {
        let (tx, rx) = mpsc::unbounded_channel();
        std::thread::spawn(move || {
            let mut stdin = std::io::stdin();
            let mut buf = [0u8; 1024];
            loop {
                match std::io::Read::read(&mut stdin, &mut buf) {
                    Ok(0) => {
                        let _ = tx.send(StdinMessage::Eof);
                        break;
                    }
                    Ok(n) => {
                        let _ = tx.send(StdinMessage::Data(buf[..n].to_vec()));
                    }
                    Err(_) => {
                        let _ = tx.send(StdinMessage::Eof);
                        break;
                    }
                }
            }
        });
        StdinChannel {
            receiver: Mutex::new(rx),
        }
    })
}

#[op2(async)]
#[serde]
pub(crate) async fn op_stdin_read() -> Result<StdinRead, CoreError> {
    let channel = stdin_channel();
    let mut rx = channel.receiver.lock().await;
    let next = match tokio::time::timeout(Duration::from_millis(50), rx.recv()).await {
        Ok(value) => value,
        Err(_) => {
            return Ok(StdinRead { data: Vec::new(), eof: false });
        }
    };
    match next {
        Some(StdinMessage::Data(data)) => Ok(StdinRead { data, eof: false }),
        Some(StdinMessage::Eof) | None => Ok(StdinRead { data: Vec::new(), eof: true }),
    }
}

#[op2(fast)]
pub(crate) fn op_stdin_set_raw_mode(#[smi] enabled: u8) -> Result<(), CoreError> {
    set_raw_mode(enabled != 0)
}

#[cfg(unix)]
fn set_raw_mode(enabled: bool) -> Result<(), CoreError> {
    use std::os::unix::io::AsRawFd;
    use std::sync::{Mutex, OnceLock};

    #[derive(Default)]
    struct RawModeState {
        original: Option<libc::termios>,
        enabled: bool,
    }

    static RAW_MODE_STATE: OnceLock<Mutex<RawModeState>> = OnceLock::new();
    let state = RAW_MODE_STATE.get_or_init(|| Mutex::new(RawModeState::default()));
    let mut guard = state
        .lock()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "raw mode lock poisoned"))?;

    if enabled {
        if guard.enabled {
            return Ok(());
        }
        let fd = std::io::stdin().as_raw_fd();
        let mut term: libc::termios = unsafe { std::mem::zeroed() };
        let res = unsafe { libc::tcgetattr(fd, &mut term as *mut libc::termios) };
        if res != 0 {
            return Err(CoreError::from(std::io::Error::last_os_error()));
        }
        if guard.original.is_none() {
            guard.original = Some(term);
        }
        let mut raw = term;
        unsafe {
            libc::cfmakeraw(&mut raw as *mut libc::termios);
        }
        let res = unsafe { libc::tcsetattr(fd, libc::TCSANOW, &raw as *const libc::termios) };
        if res != 0 {
            return Err(CoreError::from(std::io::Error::last_os_error()));
        }
        guard.enabled = true;
        return Ok(());
    }

    if !guard.enabled {
        return Ok(());
    }
    let fd = std::io::stdin().as_raw_fd();
    if let Some(original) = guard.original {
        let res = unsafe { libc::tcsetattr(fd, libc::TCSANOW, &original as *const libc::termios) };
        if res != 0 {
            return Err(CoreError::from(std::io::Error::last_os_error()));
        }
    }
    guard.enabled = false;
    Ok(())
}

#[cfg(not(unix))]
fn set_raw_mode(_enabled: bool) -> Result<(), CoreError> {
    Ok(())
}
