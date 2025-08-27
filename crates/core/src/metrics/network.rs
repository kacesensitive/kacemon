use crate::{error::Result, model::NetworkInfo};
use std::collections::HashMap;
use sysinfo::Networks;

pub struct NetworkCollector {
    networks: Networks,
    previous_stats: HashMap<String, (u64, u64, u64, u64)>, // (rx_bytes, tx_bytes, rx_packets, tx_packets)
}

impl NetworkCollector {
    pub fn new() -> Result<Self> {
        let networks = Networks::new_with_refreshed_list();
        
        Ok(Self {
            networks,
            previous_stats: HashMap::new(),
        })
    }

    pub fn init(&mut self) -> Result<()> {
        self.networks.refresh_list();
        self.networks.refresh();
        
        // Store initial network stats for delta calculation
        for (interface_name, data) in &self.networks {
            let rx_bytes = data.total_received();
            let tx_bytes = data.total_transmitted();
            let rx_packets = data.total_packets_received();
            let tx_packets = data.total_packets_transmitted();
            
            self.previous_stats.insert(
                interface_name.clone(),
                (rx_bytes, tx_bytes, rx_packets, tx_packets),
            );
        }
        
        Ok(())
    }

    pub fn collect(&mut self) -> Result<Vec<NetworkInfo>> {
        self.networks.refresh();
        
        let mut networks = Vec::new();
        
        for (interface_name, data) in &self.networks {
            let rx_bytes = data.total_received();
            let tx_bytes = data.total_transmitted();
            let rx_packets = data.total_packets_received();
            let tx_packets = data.total_packets_transmitted();
            let rx_errors = data.total_errors_on_received();
            let tx_errors = data.total_errors_on_transmitted();
            
            // Calculate deltas since last measurement
            let (rx_bytes_delta, tx_bytes_delta) = if let Some((prev_rx, prev_tx, _prev_rx_packets, _prev_tx_packets)) = 
                self.previous_stats.get(interface_name.as_str()) {
                (
                    rx_bytes.saturating_sub(*prev_rx),
                    tx_bytes.saturating_sub(*prev_tx),
                )
            } else {
                (0, 0)
            };
            
            // Update previous stats
            self.previous_stats.insert(
                interface_name.clone(),
                (rx_bytes, tx_bytes, rx_packets, tx_packets),
            );
            
            // Skip loopback and inactive interfaces for cleaner display
            if interface_name == "lo" || interface_name.starts_with("lo") {
                continue;
            }
            
            // Skip interfaces with no activity (optional filter)
            if rx_bytes == 0 && tx_bytes == 0 && rx_packets == 0 && tx_packets == 0 {
                continue;
            }
            
            networks.push(NetworkInfo {
                interface_name: interface_name.clone(),
                rx_bytes,
                tx_bytes,
                rx_bytes_delta,
                tx_bytes_delta,
                rx_packets,
                tx_packets,
                rx_errors,
                tx_errors,
            });
        }
        
        Ok(networks)
    }

    /// Get aggregate network statistics across all interfaces
    pub fn get_aggregate_stats(&mut self) -> Result<NetworkInfo> {
        let networks = self.collect()?;
        
        let mut aggregate = NetworkInfo {
            interface_name: "total".to_string(),
            rx_bytes: 0,
            tx_bytes: 0,
            rx_bytes_delta: 0,
            tx_bytes_delta: 0,
            rx_packets: 0,
            tx_packets: 0,
            rx_errors: 0,
            tx_errors: 0,
        };
        
        for net in networks {
            aggregate.rx_bytes += net.rx_bytes;
            aggregate.tx_bytes += net.tx_bytes;
            aggregate.rx_bytes_delta += net.rx_bytes_delta;
            aggregate.tx_bytes_delta += net.tx_bytes_delta;
            aggregate.rx_packets += net.rx_packets;
            aggregate.tx_packets += net.tx_packets;
            aggregate.rx_errors += net.rx_errors;
            aggregate.tx_errors += net.tx_errors;
        }
        
        Ok(aggregate)
    }
}
