use std::collections::VecDeque;
use std::sync::Mutex;

use crate::{NetHost, PortEvent, PortInfo, PortProtocol, PortPublishOptions, Result};

#[derive(Debug, Default)]
pub struct InMemoryNetHost {
    state: Mutex<NetState>,
}

impl InMemoryNetHost {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(NetState::default()),
        }
    }
}

#[derive(Debug, Default)]
struct NetState {
    events: VecDeque<PortEvent>,
}

impl NetHost for InMemoryNetHost {
    fn publish_port(&self, port: u16, options: PortPublishOptions) -> Result<PortInfo> {
        let url = build_url(port, options.protocol, options.host.as_deref());
        let info = PortInfo {
            port,
            protocol: options.protocol,
            url,
        };
        let mut state = self.state.lock().unwrap();
        state.events.push_back(PortEvent::ServerReady(info.clone()));
        Ok(info)
    }

    fn unpublish_port(&self, port: u16) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.events.push_back(PortEvent::PortClosed(port));
        Ok(())
    }

    fn next_event(&self) -> Result<Option<PortEvent>> {
        let mut state = self.state.lock().unwrap();
        Ok(state.events.pop_front())
    }
}

fn build_url(port: u16, protocol: PortProtocol, host: Option<&str>) -> String {
    let scheme = match protocol {
        PortProtocol::Http => "http",
        PortProtocol::Https => "https",
        PortProtocol::Tcp => "tcp",
        PortProtocol::Udp => "udp",
    };
    match host {
        Some(host) => {
            if host.contains("://") {
                format!("{}:{}", host.trim_end_matches('/'), port)
            } else {
                format!("{}://{}:{}", scheme, host, port)
            }
        }
        None => format!("{}://localhost:{}", scheme, port),
    }
}
