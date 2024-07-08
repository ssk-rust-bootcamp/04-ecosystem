use std::{
    fmt::{self, Display},
    net::SocketAddr,
    sync::Arc,
};

use anyhow::Result;
use dashmap::DashMap;
use futures::{stream::SplitStream, SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[derive(Debug, Default)]
struct State {
    peers: DashMap<SocketAddr, mpsc::Sender<Arc<Message>>>,
}
struct Peer {
    username: String,
    stream: SplitStream<Framed<TcpStream, LinesCodec>>,
}
#[derive(Debug)]
enum Message {
    UserJoined(String),
    UserLeft(String),
    Chat { sender: String, content: String },
}

const MAX_MESSAGES: usize = 128;
#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    // console_subscriber::init();
    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;
    info!("listening on {}", addr);
    let state = Arc::new(State::default());

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Accepted connection from {}", addr);
        let state_clone = state.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(state_clone, addr, stream).await {
                warn!("Failed to handle client {}:{}", addr, e);
            }
        });
    }
}

async fn handle_client(state: Arc<State>, addr: SocketAddr, stream: TcpStream) -> Result<()> {
    let mut stream = Framed::new(stream, LinesCodec::new());

    stream.send("Enter your username:").await?;

    let username = match stream.next().await {
        Some(Ok(username)) => username,
        Some(Err(e)) => return Err(e.into()),
        None => return Ok(()),
    };

    let mut peer = state.add(addr, username, stream).await;

    let message = Arc::new(Message::user_joined(&peer.username));

    info!("{message}");

    state.broadcast(addr, message).await;

    while let Some(line) = peer.stream.next().await {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                warn!("Failed to read line from {}:{}", addr, e);
                break;
            }
        };
        let message = Arc::new(Message::chat(&peer.username, line));
        state.broadcast(addr, message).await;
    }

    // when while loop exit, peer has left the chat or line reading failed
    // remove peer from state
    state.peers.remove(&addr);

    // notify others that a user has left
    let message = Arc::new(Message::user_left(&peer.username));
    info!("{}", message);

    state.broadcast(addr, message).await;

    Ok(())
}

impl Message {
    fn user_joined(username: &str) -> Self {
        let content = format!("{} has joined the chat", username);
        Self::UserJoined(content)
    }

    fn chat(sender: impl Into<String>, content: impl Into<String>) -> Self {
        let sender = sender.into();
        let content = content.into();
        Self::Chat { sender, content }
    }
    fn user_left(username: &str) -> Self {
        let content = format!("{} has left the chat", username);
        Self::UserLeft(content)
    }
}

impl State {
    async fn broadcast(&self, addr: SocketAddr, message: Arc<Message>) {
        for peer in self.peers.iter() {
            if peer.key() == &addr {
                continue;
            }
            if let Err(e) = peer.value().send(message.clone()).await {
                warn!("Failed to seng message to {} :{} ", peer.key(), e);
                self.peers.remove(peer.key());
            }
        }
    }

    async fn add(&self, addr: SocketAddr, username: String, stream: Framed<TcpStream, LinesCodec>) -> Peer {
        let (tx, mut rx) = mpsc::channel(MAX_MESSAGES);
        self.peers.insert(addr, tx);
        let (mut stream_sender, stream_receiver) = stream.split();

        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = stream_sender.send(message.to_string()).await {
                    warn!("Failed to send message to {} :{} ", addr, e);
                    break;
                }
            }
        });
        Peer {
            username,
            stream: stream_receiver,
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::UserJoined(content) => write!(f, "[{}]", content),
            Message::UserLeft(content) => write!(f, "[{} :(]", content),
            Message::Chat { sender, content } => write!(f, "{}:{}", sender, content),
        }
    }
}
