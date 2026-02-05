use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::time::Duration;
use std::net::IpAddr;

pub struct Discovery;

impl Discovery {
    pub async fn find_bridge(timeout: Duration, target_id: Option<String>) -> Option<(String, IpAddr)> {
        let mdns = ServiceDaemon::new().ok()?;
        let service_type = "_hue._tcp.local.";
        let receiver = mdns.browse(service_type).ok()?;

        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            while let Ok(event) = receiver.recv_timeout(Duration::from_millis(100)) {
                if let ServiceEvent::ServiceResolved(info) = event {
                    let addresses = info.get_addresses();
                    // Prefer IPv4
                    let target_ip = addresses.iter().find(|ip| ip.is_ipv4()).or_else(|| addresses.iter().next());
                    
                    if let Some(ip) = target_ip {
                        let raw_id = info.get_properties().get("bridgeid").map(|p| p.to_string()).unwrap_or_default();
                        // Clean up if it contains "bridgeid=" prefix
                        let id = if raw_id.starts_with("bridgeid=") {
                            raw_id.replace("bridgeid=", "")
                        } else {
                            raw_id
                        };
                        
                        let clean_id = id.to_lowercase();
                        
                        if let Some(target) = &target_id {
                            if !clean_id.eq_ignore_ascii_case(target) {
                                continue;
                            }
                        }
                        
                        return Some((clean_id, *ip));
                    }
                }
            }
        }
        None
    }
}
