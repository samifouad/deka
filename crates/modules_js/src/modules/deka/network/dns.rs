use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use deno_core::{error::CoreError, op2};
use dns_lookup::lookup_addr;
use tokio::net::lookup_host;

#[op2(async)]
#[serde]
pub(crate) async fn op_dns_lookup(
    #[string] hostname: String,
    #[number] family: u64,
) -> Result<Vec<String>, CoreError> {
    let host = hostname.as_str();
    if let Ok(ip) = host.parse::<Ipv4Addr>() {
        return Ok(vec![ip.to_string()]);
    }
    if let Ok(ip) = host.parse::<Ipv6Addr>() {
        return Ok(vec![ip.to_string()]);
    }

    let results = lookup_host((host, 0)).await.map_err(CoreError::from)?;
    let mut addrs = Vec::new();
    for addr in results {
        match (family, addr.ip()) {
            (4, IpAddr::V4(ip)) => addrs.push(ip.to_string()),
            (6, IpAddr::V6(ip)) => addrs.push(ip.to_string()),
            (0, ip) => addrs.push(ip.to_string()),
            _ => {}
        }
    }
    if addrs.is_empty() {
        return Err(CoreError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "DNS lookup failed",
        )));
    }
    Ok(addrs)
}

#[op2(async)]
#[serde]
pub(crate) async fn op_dns_reverse(#[string] address: String) -> Result<Vec<String>, CoreError> {
    let ip: IpAddr = address.parse().map_err(|err| {
        CoreError::from(std::io::Error::new(std::io::ErrorKind::InvalidInput, err))
    })?;
    let host = tokio::task::spawn_blocking(move || lookup_addr(&ip))
        .await
        .map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                err.to_string(),
            ))
        })?
        .map_err(CoreError::from)?;
    Ok(vec![host.to_string()])
}
