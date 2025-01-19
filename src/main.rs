use btleplug::api::{Central, Manager as _, Peripheral as _, WriteType, ScanFilter};
use btleplug::platform::Manager;
use std::error::Error;
use std::time::{Duration, Instant};
use tokio::time;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await?;

    // Get the first Bluetooth adapter
    let adapters = manager.adapters().await?;
    if adapters.is_empty() {
        eprintln!("No Bluetooth adapters found.");
        return Ok(());
    }    
    let adapter = adapters.into_iter().next().unwrap();

    // Start scanning for devices
    adapter.start_scan(ScanFilter::default()).await?;
    time::sleep(Duration::from_secs(2)).await;

    // Find all available devices
    let devices = adapter.peripherals().await?;    

    // List all available devices and their properties
    for device in &devices {
        if let Ok(Some(props)) = device.properties().await {
            println!("Device found: {:?}", props.local_name);
        }
    }

    // Optionally, you can select a device based on a specific name, UUID, etc.
    let target_device_name = "Jakub’s iPhone";  // Replace with your desired device name
    let mut target_device = None;

    for device in devices {
        if let Ok(Some(props)) = device.properties().await {
            if props.local_name == Some(target_device_name.to_string()) {
                target_device = Some(device);
                break;
            }
        }
    }

    // If no target device is found, exit
    let Some(device) = target_device else {
        eprintln!("Target device not found.");
        return Ok(());
    };

    // Connect to the device
    device.connect().await?;
    println!("DEVICE CONNECTED");
    device.discover_services().await?;

    let services = device.services();

    // List all available characteristics (UUIDs) from the services
    for service in &services {
        println!("Service UUID: {}", service.uuid);
        for characteristic in &service.characteristics {
            println!("  Characteristic UUID: {}", characteristic.uuid);
        }
    }

    // Optionally, choose the characteristic by name, UUID, or other criteria
    // For simplicity, let's assume we pick the first available characteristic
    let characteristic = services
        .iter()
        .flat_map(|service| &service.characteristics)
        .next()
        .ok_or_else(|| "No characteristic found")?;

    // Set a timeout duration
    let timeout_duration = Duration::from_secs(10);
    let start_time = Instant::now();

    // Wait for a message or timeout
    println!("READING");
    loop {
        // If the timeout has been reached, break the loop
        if start_time.elapsed() > timeout_duration {
            println!("Timeout reached, no message received.");
            break;
        }

        // Try to read data from the characteristic
        match device.read(&characteristic).await {
            Ok(data) => {
                
                println!("Data read: {:?}", data);
                break; // Exit loop if data is received
            }
            Err(_) => {
                // Continue if no data received yet
                time::sleep(Duration::from_millis(100)).await;
            }
        }
    }

    // Send data to the device after the message or timeout
    let write_data = vec![0x01, 0x02, 0x03];
    println!("WRITING");
    match device
        .write(&characteristic, &write_data, WriteType::WithoutResponse)
        .await
    {
        Ok(_) => println!("Data written: {:?}", write_data),
        Err(e) => eprintln!("Error writing data: {:?}", e),
    }

    // Disconnect from the device
    println!("DISCONNECTING...");
    device.disconnect().await?;

    Ok(())
}
