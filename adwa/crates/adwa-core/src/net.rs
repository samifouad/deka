use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortProtocol {
    Http,
    Https,
    Tcp,
    Udp,
}

#[derive(Debug, Clone)]
pub struct PortPublishOptions {
    pub protocol: PortProtocol,
    pub host: Option<String>,
}

impl Default for PortPublishOptions {
    fn default() -> Self {
        Self {
            protocol: PortProtocol::Http,
            host: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PortInfo {
    pub port: u16,
    pub protocol: PortProtocol,
    pub url: String,
}

#[derive(Debug, Clone)]
pub enum PortEvent {
    ServerReady(PortInfo),
    PortClosed(u16),
}

pub trait NetHost: Send + Sync {
    fn publish_port(&self, port: u16, options: PortPublishOptions) -> Result<PortInfo>;
    fn unpublish_port(&self, port: u16) -> Result<()>;
    fn next_event(&self) -> Result<Option<PortEvent>>;
}
