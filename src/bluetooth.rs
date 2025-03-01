// src/bluetooth.rs
use btleplug::platform::Manager;
use btleplug::api::{Central, ScanFilter};
use futures::executor::block_on;
use std::time::Duration;
use std::thread;

/// Scans for Bluetooth Low Energy devices and prints any found.
pub fn scan_for_devices() {
    // Run the async scan in a separate thread to avoid blocking the main UI thread.
    thread::spawn(|| {
        // Use a lightweight executor to run async code.
        let result = block_on(async {
            let manager = Manager::new().await.map_err(|e| format!("Manager init error: {}", e))?;
            let adapters = manager.adapters().await.map_err(|e| format!("Adapter list error: {}", e))?;
            if adapters.is_empty() {
                println!("No Bluetooth adapters found.");
                return Ok(());
            }
            let central = adapters.into_iter().next().unwrap();
            // Start BLE scan
            println!("Starting Bluetooth LE scan...");
            central.start_scan(ScanFilter::default()).await
                .map_err(|e| format!("Failed to start scan: {}", e))?;
            // Scan for a fixed duration
            tokio::time::sleep(Duration::from_secs(5)).await;
            // Stop scanning (optional)
            let _ = central.stop_scan().await;
            // Retrieve discovered peripherals
            let peripherals = central.peripherals().await
                .map_err(|e| format!("Error getting peripherals: {}", e))?;
            if peripherals.is_empty() {
                println!("No BLE devices found.");
            } else {
                println!("BLE devices found:");
                for p in peripherals {
                    let props = p.properties().await.map_err(|e| format!("Props error: {}", e))?;
                    let name = props.and_then(|pr| pr.local_name).unwrap_or("(unknown)".to_string());
                    let addr = p.address();
                    println!("  - {} [{:?}]", name, addr);
                }
            }
            Ok::<(), String>(())
        });
        if let Err(err) = result {
            eprintln!("Bluetooth scan failed: {}", err);
        }
    });
}
