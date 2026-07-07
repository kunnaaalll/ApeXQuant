fn main() {
    println!("Starting APEX V3 Market Data Engine...");
    
    // The engine is still under development, keep it alive so docker-compose doesn't crash
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}
