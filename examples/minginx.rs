use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::{
    io,
    net::{TcpListener, TcpStream},
};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};
#[derive(Debug, Serialize, Deserialize)]
struct Config {
    upstream_str: String,
    listen_str: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let config = resolve_config();
    let config = Arc::new(config);
    info!("Upstream is {}", config.upstream_str);
    info!("Listening on {}", config.upstream_str);

    let listener = TcpListener::bind(&config.listen_str).await?;
    loop {
        let (client, addr) = listener.accept().await?;
        info!("accepted connection from {}", addr);
        let cloned_config = config.clone();
        tokio::spawn(async move {
            let upstream = TcpStream::connect(&cloned_config.upstream_str).await?;
            proxy(client, upstream).await?;
            Ok::<(), anyhow::Error>(())
        });
    }

    #[allow(unreachable_code)]
    Ok::<(), anyhow::Error>(())
}

async fn proxy(mut client: TcpStream, mut upstream: TcpStream) -> Result<()> {
    let (mut client_read, mut client_write) = client.split();
    let (mut upstream_read, mut upstream_write) = upstream.split();
    let client_to_upstream = io::copy(&mut client_read, &mut upstream_write);
    let upstream_to_client = io::copy(&mut upstream_read, &mut client_write);
    match tokio::try_join!(client_to_upstream, upstream_to_client) {
        Ok((n, m)) => info!(
            "proxyed {} bytes from client to upstream ,{} bytes from upstream to client",
            n, m
        ),
        Err(e) => warn!("error proxying: {:?}", e),
    }
    Ok(())
}
fn resolve_config() -> Config {
    Config {
        upstream_str: "0.0.0.0:8080".to_string(),
        listen_str: "0.0.0.0:8081".to_string(),
    }
}
