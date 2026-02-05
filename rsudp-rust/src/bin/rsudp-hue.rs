use clap::{Parser, Subcommand};
use rsudp_rust::hue::{client::HueClient, discovery::Discovery};
use std::time::Duration;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Setup,
    List {
        #[arg(long)]
        ip: String,
        #[arg(long)]
        key: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Setup) => {
            println!("Discovering Hue Bridges...");
            match Discovery::find_bridge(Duration::from_secs(5), None).await {
                Some((id, ip)) => {
                    println!("Found Bridge: {} at {}", id, ip);
                    println!("Please press the Link Button on the Hue Bridge NOW.");
                    println!("Waiting 30 seconds for link...");
                    
                    // Retry loop for pairing
                    let client = match HueClient::new(&ip.to_string(), None) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("Failed to create client: {}", e);
                            return;
                        }
                    };

                    let start = std::time::Instant::now();
                    while start.elapsed() < Duration::from_secs(30) {
                        match client.register_app().await {
                            Ok(key) => {
                                println!("\nSUCCESS! App Key generated.");
                                println!("App Key: {}", key);
                                println!("Bridge ID: {}", id);
                                println!("\nPlease add these to your rsudp.toml config.");
                                return;
                            }
                            Err(e) => {
                                if e.contains("link button not pressed") {
                                    print!(".");
                                    use std::io::Write;
                                    std::io::stdout().flush().unwrap();
                                    tokio::time::sleep(Duration::from_secs(1)).await;
                                } else {
                                    eprintln!("\nError: {}", e);
                                    return;
                                }
                            }
                        }
                    }
                    println!("\nTimeout: Link button was not pressed.");
                }
                None => {
                    println!("No Hue Bridge found via mDNS.");
                }
            }
        }
        Some(Commands::List { ip, key }) => {
            println!("Connecting to {}...", ip);
            let client = match HueClient::new(ip, Some(key.clone())) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to create client: {}", e);
                    return;
                }
            };

            println!("\nLights:");
            match client.get_lights().await {
                Ok(lights) => {
                    for l in lights {
                        let name = l.metadata.map(|m| m.name).unwrap_or("Unknown".to_string());
                        println!("- ID: {} | Name: {}", l.id, name);
                    }
                }
                Err(e) => eprintln!("Failed to get lights: {}", e),
            }

            println!("\nRooms:");
            match client.get_rooms().await {
                Ok(rooms) => {
                    for r in rooms {
                        let name = r.metadata.map(|m| m.name).unwrap_or("Unknown".to_string());
                        println!("- ID: {} | Name: {}", r.id, name);
                    }
                }
                Err(e) => eprintln!("Failed to get rooms: {}", e),
            }
            
            println!("\nZones:");
            match client.get_zones().await {
                Ok(zones) => {
                    for z in zones {
                        let name = z.metadata.map(|m| m.name).unwrap_or("Unknown".to_string());
                        println!("- ID: {} | Name: {}", z.id, name);
                    }
                }
                Err(e) => eprintln!("Failed to get zones: {}", e),
            }
        }
        None => {}
    }
}
