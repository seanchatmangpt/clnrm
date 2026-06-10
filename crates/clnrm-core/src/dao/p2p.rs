use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipMessage {
    pub message_id: String,
    pub sender: SocketAddr,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PeerConnection {
    pub address: SocketAddr,
    pub sender_channel: mpsc::Sender<GossipMessage>,
}

#[derive(Debug, Clone)]
pub struct NodeDiscovery {
    pub active_peers: Arc<RwLock<HashMap<SocketAddr, PeerConnection>>>,
    pub seen_messages: Arc<RwLock<HashSet<String>>>,
    pub local_address: SocketAddr,
}

impl NodeDiscovery {
    pub fn new(local_address: SocketAddr) -> Self {
        Self {
            active_peers: Arc::new(RwLock::new(HashMap::new())),
            seen_messages: Arc::new(RwLock::new(HashSet::new())),
            local_address,
        }
    }

    pub async fn add_peer(&self, address: SocketAddr, sender_channel: mpsc::Sender<GossipMessage>) {
        let mut peers = self.active_peers.write().await;
        peers.insert(
            address,
            PeerConnection {
                address,
                sender_channel,
            },
        );
    }

    pub async fn remove_peer(&self, address: &SocketAddr) {
        let mut peers = self.active_peers.write().await;
        peers.remove(address);
    }

    pub async fn get_peer_addresses(&self) -> Vec<SocketAddr> {
        let peers = self.active_peers.read().await;
        peers.keys().cloned().collect()
    }

    pub async fn record_message_seen(&self, message_id: &str) -> bool {
        let mut seen = self.seen_messages.write().await;
        seen.insert(message_id.to_string())
    }
}

pub struct P2pNetwork {
    pub discovery: NodeDiscovery,
    pub listener_port: u16,
    incoming_message_sender: mpsc::Sender<GossipMessage>,
    pub incoming_message_receiver: tokio::sync::Mutex<mpsc::Receiver<GossipMessage>>,
}

impl P2pNetwork {
    pub fn new(listener_port: u16) -> Self {
        let local_address = format!("0.0.0.0:{}", listener_port)
            .parse::<SocketAddr>()
            .expect("Invalid local address format");
        let (tx, rx) = mpsc::channel(1024);

        Self {
            discovery: NodeDiscovery::new(local_address),
            listener_port,
            incoming_message_sender: tx,
            incoming_message_receiver: tokio::sync::Mutex::new(rx),
        }
    }

    pub async fn start(&self) -> std::io::Result<()> {
        let address = format!("0.0.0.0:{}", self.listener_port);
        let listener = TcpListener::bind(&address).await?;

        let discovery_clone = self.discovery.clone();
        let internal_tx = self.incoming_message_sender.clone();

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let tx = internal_tx.clone();
                        let discovery = discovery_clone.clone();
                        tokio::spawn(async move {
                            if let Err(e) =
                                Self::handle_connection(stream, addr, discovery, tx).await
                            {
                                tracing::error!("Connection error with {}: {}", addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to accept incoming connection: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn connect_to_peer(&self, peer_address: SocketAddr) -> std::io::Result<()> {
        let stream = TcpStream::connect(peer_address).await?;
        let discovery_clone = self.discovery.clone();
        let internal_tx = self.incoming_message_sender.clone();

        tokio::spawn(async move {
            if let Err(e) =
                Self::handle_connection(stream, peer_address, discovery_clone, internal_tx).await
            {
                tracing::error!("Connection error with peer {}: {}", peer_address, e);
            }
        });

        Ok(())
    }

    async fn handle_connection(
        stream: TcpStream,
        peer_addr: SocketAddr,
        discovery: NodeDiscovery,
        internal_tx: mpsc::Sender<GossipMessage>,
    ) -> std::io::Result<()> {
        let (tx, mut rx) = mpsc::channel::<GossipMessage>(100);
        discovery.add_peer(peer_addr, tx).await;

        let (mut read_half, mut write_half) = stream.into_split();

        let writer_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let serialized = match serde_json::to_vec(&msg) {
                    Ok(bytes) => bytes,
                    Err(_) => break,
                };

                let len = serialized.len() as u32;
                if write_half.write_all(&len.to_be_bytes()).await.is_err() {
                    break;
                }
                if write_half.write_all(&serialized).await.is_err() {
                    break;
                }
            }
        });

        let discovery_read_clone = discovery.clone();
        let reader_task = tokio::spawn(async move {
            loop {
                let mut len_buf = [0u8; 4];
                if read_half.read_exact(&mut len_buf).await.is_err() {
                    break;
                }

                let len = u32::from_be_bytes(len_buf) as usize;

                if len > 10 * 1024 * 1024 {
                    break;
                }

                let mut msg_buf = vec![0u8; len];
                if read_half.read_exact(&mut msg_buf).await.is_err() {
                    break;
                }

                if let Ok(msg) = serde_json::from_slice::<GossipMessage>(&msg_buf) {
                    if discovery_read_clone
                        .record_message_seen(&msg.message_id)
                        .await
                    {
                        let _ = internal_tx.send(msg).await;
                    }
                } else {
                    break;
                }
            }
        });

        tokio::select! {
            _ = writer_task => {},
            _ = reader_task => {},
        };

        discovery.remove_peer(&peer_addr).await;
        Ok(())
    }

    pub async fn broadcast(&self, message: GossipMessage) {
        self.discovery
            .record_message_seen(&message.message_id)
            .await;

        let peers = self.discovery.active_peers.read().await;
        for peer in peers.values() {
            let _ = peer.sender_channel.send(message.clone()).await;
        }
    }
}
