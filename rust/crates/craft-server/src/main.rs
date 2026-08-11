use craft_server::serve;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:4080".into());
    let db_path = std::env::var("CRAFT_DB").unwrap_or_else(|_| "craft.db".into());
    let listener = TcpListener::bind(&addr).await?;
    serve(listener, &db_path).await
}
