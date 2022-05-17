use ipdis_api::server::IpdisServer;
use ipis::{env::Infer, tokio};

#[tokio::main]
async fn main() {
    IpdisServer::infer().await.run().await
}
