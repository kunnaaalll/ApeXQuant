#[tokio::main]
async fn main() {
    println!("Validator starting...");
    println!("Validator running...");
    std::future::pending::<()>().await;
}
