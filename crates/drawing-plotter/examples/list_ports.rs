//! List available serial ports and check for AxiDraw devices

#[cfg(feature = "hardware")]
fn main() {
    use drawing_plotter::AxiDraw;

    println!("Available USB serial ports:");
    match AxiDraw::list_ports_detailed() {
        Ok(ports) => {
            for p in &ports {
                let axi = if p.is_axidraw { " [AXIDRAW]" } else { "" };
                println!(
                    "  - {}{} ({:?}, {:?})",
                    p.name, axi, p.product, p.manufacturer
                );
            }
            if ports.is_empty() {
                println!("  (none)");
            }
        }
        Err(e) => println!("  Error: {}", e),
    }

    println!("\nAxiDraw devices:");
    match AxiDraw::find_devices() {
        Ok(devices) => {
            for d in &devices {
                println!("  - {}", d);
            }
            if devices.is_empty() {
                println!("  (none found)");
            }
        }
        Err(e) => println!("  Error: {}", e),
    }

    println!("\nTrying auto-connect...");
    match AxiDraw::auto_connect() {
        Ok(_ax) => println!("  Connected!"),
        Err(e) => println!("  Failed: {}", e),
    }
}

#[cfg(not(feature = "hardware"))]
fn main() {
    println!("This example requires the 'hardware' feature.");
    println!("Run with: cargo run --example list_ports --features hardware");
}
