use anyhow::Result;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;

const SSDP_MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(239, 255, 255, 250);
const SSDP_PORT: u16 = 1900;

// Try multiple search targets for better compatibility
const SEARCH_TARGETS: &[&str] = &[
    "urn:schemas-denon-com:device:ACT-Denon:1",
    "urn:schemas-upnp-org:device:MediaRenderer:1",
    "ssdp:all",
];

#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub ip: String,
    pub location: String,
    pub friendly_name: Option<String>,
}

pub async fn discover_devices(timeout_secs: u64) -> Result<Vec<DiscoveredDevice>> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true)?;

    let multicast_addr = SocketAddrV4::new(SSDP_MULTICAST_ADDR, SSDP_PORT);

    // Send search requests for each target
    for search_target in SEARCH_TARGETS {
        let search_msg = format!(
            "M-SEARCH * HTTP/1.1\r\n\
             HOST: {}:{}\r\n\
             MAN: \"ssdp:discover\"\r\n\
             MX: 3\r\n\
             ST: {}\r\n\
             \r\n",
            SSDP_MULTICAST_ADDR, SSDP_PORT, search_target
        );

        let _ = socket.send_to(search_msg.as_bytes(), multicast_addr).await;
    }

    let mut devices = Vec::new();
    let mut buf = [0u8; 2048];

    let discovery_timeout = Duration::from_secs(timeout_secs);
    let deadline = tokio::time::Instant::now() + discovery_timeout;

    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }

        match timeout(remaining, socket.recv_from(&mut buf)).await {
            Ok(Ok((len, addr))) => {
                let response = String::from_utf8_lossy(&buf[..len]);

                // Check if this looks like a HEOS/Denon device
                let is_heos = response.to_lowercase().contains("heos")
                    || response.to_lowercase().contains("denon")
                    || response.to_lowercase().contains("marantz")
                    || response.contains("ACT-Denon");

                if is_heos {
                    let ip = addr.ip().to_string();
                    let location = parse_header(&response, "LOCATION");

                    // Avoid duplicates
                    if !devices.iter().any(|d: &DiscoveredDevice| d.ip == ip) {
                        devices.push(DiscoveredDevice {
                            ip,
                            location: location.unwrap_or_default(),
                            friendly_name: None,
                        });
                    }
                }
            }
            Ok(Err(_)) => break,
            Err(_) => break, // Timeout
        }
    }

    Ok(devices)
}

fn parse_header(response: &str, header: &str) -> Option<String> {
    for line in response.lines() {
        let line_upper = line.to_uppercase();
        if line_upper.starts_with(&format!("{}:", header.to_uppercase())) {
            return Some(line.splitn(2, ':').nth(1)?.trim().to_string());
        }
    }
    None
}

pub async fn discover_first_device(timeout_secs: u64) -> Result<Option<String>> {
    let devices = discover_devices(timeout_secs).await?;
    Ok(devices.into_iter().next().map(|d| d.ip))
}
