# AI Agent Swarm Communication & Consensus (v1.8.0+)

**Feature Version**: v1.8.0 - v2.0.0+
**Status**: Architecture & Protocol Design
**Last Updated**: 2025-11-18

---

## Communication Architecture

### Message Types

```rust
// crates/clnrm-agents/src/communication/message.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    // Discovery
    PeerAnnouncement {
        agent_id: AgentId,
        address: SocketAddr,
        role: AgentRole,
        capabilities: Vec<Capability>,
    },
    PeerRequest(AgentRole),

    // Work Distribution
    TaskAssignment {
        task_id: TaskId,
        task: Task,
        deadline: SystemTime,
    },
    TaskResult {
        task_id: TaskId,
        result: TaskResult,
    },
    TaskFailed {
        task_id: TaskId,
        reason: String,
    },

    // Consensus
    VoteRequest {
        proposal_id: ProposalId,
        proposal: Proposal,
    },
    Vote {
        proposal_id: ProposalId,
        vote: bool,
    },
    Committed {
        proposal_id: ProposalId,
    },

    // Health
    Heartbeat {
        agent_id: AgentId,
        timestamp: SystemTime,
        state: AgentState,
        metrics: AgentMetrics,
    },
    HeartbeatAck(AgentId),

    // Logging
    Log {
        level: LogLevel,
        message: String,
        timestamp: SystemTime,
    },

    // Generic
    Request {
        id: RequestId,
        payload: Vec<u8>,
    },
    Response {
        id: RequestId,
        payload: Vec<u8>,
    },
}
```

### gRPC Service Definition

```protobuf
// proto/agent_service.proto

package clnrm.agent;

syntax = "proto3";

service AgentService {
    rpc SendMessage(MessageRequest) returns (MessageResponse);
    rpc Heartbeat(HeartbeatRequest) returns (HeartbeatResponse);
    rpc GetState(GetStateRequest) returns (AgentState);
    rpc ExecuteTask(TaskRequest) returns (TaskResult);
    rpc DiscoverPeers(DiscoverRequest) returns (PeerList);
}

message MessageRequest {
    bytes message = 1;  // Serialized Message enum
    string sender_id = 2;
}

message MessageResponse {
    bool received = 1;
    string error = 2;
}

message HeartbeatRequest {
    string agent_id = 1;
    int64 timestamp_ms = 2;
}

message HeartbeatResponse {
    bool alive = 1;
    AgentState state = 2;
    AgentMetrics metrics = 3;
}

message TaskRequest {
    string task_id = 1;
    bytes task_payload = 2;  // Serialized Task
}

message TaskResult {
    string task_id = 1;
    bytes result_payload = 2;  // Serialized TaskResult
    int64 duration_ms = 3;
    bool success = 4;
    string error = 5;
}
```

### Communication Channels

```rust
pub struct CommunicationLayer {
    // One-to-one messaging
    unicast: Arc<UnicastChannel>,

    // Broadcasting
    broadcast: Arc<BroadcastChannel>,

    // Pub-Sub
    pubsub: Arc<PubSubChannel>,

    // RPC
    rpc_client: Arc<RpcClient>,
    rpc_server: Arc<RpcServer>,
}

pub struct UnicastChannel {
    peers: Arc<DashMap<AgentId, PeerConnection>>,
    timeout: Duration,
    retries: u32,
}

pub struct PeerConnection {
    address: SocketAddr,
    channel: grpc::Channel,
    client: AgentServiceClient,
}

impl UnicastChannel {
    pub async fn send_message(
        &self,
        to: AgentId,
        message: Message,
    ) -> Result<()> {
        let peer = self.peers.get(&to)
            .ok_or_else(|| CleanroomError::not_found("Peer not found"))?;

        // Serialize message
        let bytes = serde_json::to_vec(&message)?;

        // Send with retries
        let mut last_err = None;
        for attempt in 0..self.retries {
            match peer.client.send_message(
                MessageRequest {
                    message: bytes.clone(),
                    sender_id: "self".to_string(),
                }
            ).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    last_err = Some(e);
                    if attempt < self.retries - 1 {
                        tokio::time::sleep(Duration::from_millis(2u64.pow(attempt))).await;
                    }
                }
            }
        }

        Err(last_err.unwrap().into())
    }
}

pub struct BroadcastChannel {
    subscribers: Arc<DashMap<Topic, Vec<AgentId>>>,
}

impl BroadcastChannel {
    pub async fn broadcast(
        &self,
        topic: Topic,
        message: Message,
    ) -> Result<BroadcastStats> {
        let subscribers = self.subscribers
            .get(&topic)
            .map(|v| v.clone())
            .unwrap_or_default();

        let mut success = 0;
        let mut failed = 0;

        for subscriber in subscribers {
            match self.send_to(&subscriber, &message).await {
                Ok(()) => success += 1,
                Err(_) => failed += 1,
            }
        }

        Ok(BroadcastStats { success, failed })
    }
}

pub struct PubSubChannel {
    topics: Arc<DashMap<Topic, Arc<broadcast::Sender<Message>>>>,
}

impl PubSubChannel {
    pub async fn publish(&self, topic: Topic, message: Message) -> Result<()> {
        if let Some(sender) = self.topics.get(&topic) {
            sender.send(message).ok();
        }

        Ok(())
    }

    pub async fn subscribe(&self, topic: Topic) -> Result<broadcast::Receiver<Message>> {
        let sender = self.topics
            .entry(topic)
            .or_insert_with(|| Arc::new(broadcast::channel(1000).0))
            .clone();

        Ok(sender.subscribe())
    }
}
```

---

## Consensus Protocol

### Raft-Based Consensus

```rust
// crates/clnrm-agents/src/consensus/raft.rs

pub struct RaftCluster {
    // Node state
    self_id: NodeId,
    nodes: Vec<NodeId>,

    // Persistent state (on disk)
    current_term: Arc<RwLock<u64>>,
    voted_for: Arc<RwLock<Option<NodeId>>>,
    log: Arc<RwLock<Vec<LogEntry>>>,

    // Volatile state
    commit_index: Arc<AtomicUsize>,
    last_applied: Arc<AtomicUsize>,

    // Leader state
    next_index: Arc<DashMap<NodeId, usize>>,
    match_index: Arc<DashMap<NodeId, usize>>,

    // Communication
    comm: Arc<CommunicationLayer>,
    state_machine: Arc<RwLock<StateMachine>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    term: u64,
    index: usize,
    command: Command,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    AgentJoined { agent_id: AgentId, address: SocketAddr },
    AgentFailed { agent_id: AgentId },
    TestCompleted { result: TestResult },
    PolicyUpdated { policy: Policy },
    ResourceAllocated { grant: ResourceGrant },
}

pub struct StateMachine {
    // Agents in cluster
    agents: HashMap<AgentId, AgentState>,

    // Test results
    completed_tests: HashMap<TestId, TestResult>,

    // Resource allocations
    allocations: HashMap<ResourceId, ResourceGrant>,

    // Policies
    policies: Vec<Policy>,
}

impl RaftCluster {
    pub async fn propose(&self, command: Command) -> Result<ProposalId> {
        // Only leader can propose
        if !self.is_leader().await {
            return Err(CleanroomError::invalid_operation("Not the leader"));
        }

        let term = self.current_term.read().await.clone();
        let index = {
            let log = self.log.read().await;
            log.len() + 1
        };

        // Append to log
        let entry = LogEntry { term, index, command };
        {
            let mut log = self.log.write().await;
            log.push(entry);
        }

        // Replicate to followers
        self.replicate_log().await?;

        // Wait for majority quorum
        self.wait_for_quorum(index).await?;

        Ok(ProposalId::new())
    }

    async fn replicate_log(&self) -> Result<()> {
        let log = self.log.read().await;
        let last_log_index = log.len();
        let last_log_term = log.last().map(|e| e.term).unwrap_or(0);

        for node in &self.nodes {
            if *node == self.self_id {
                continue;
            }

            let next_idx = *self.next_index.get(node).unwrap_or(&1);
            let prev_log_term = if next_idx > 1 {
                log[next_idx - 2].term
            } else {
                0
            };

            let entries = log[next_idx - 1..].to_vec();

            // Send AppendEntries RPC
            let request = AppendEntriesRequest {
                term: *self.current_term.read().await,
                leader_id: self.self_id.clone(),
                prev_log_index: next_idx - 1,
                prev_log_term,
                entries,
                leader_commit: *self.commit_index.load(Ordering::Relaxed),
            };

            // Async send (fire-and-forget for now)
            let node = node.clone();
            tokio::spawn(async move {
                // Send request to node
                // Handle response...
            });
        }

        Ok(())
    }

    async fn wait_for_quorum(&self, index: usize) -> Result<()> {
        let quorum = self.nodes.len() / 2 + 1;
        let mut votes = 1; // Vote for self

        // Wait until majority has replicated
        loop {
            let mut agreed = votes;

            for entry in self.match_index.iter() {
                if entry.value() >= &index {
                    agreed += 1;
                }
            }

            if agreed >= quorum {
                // Commit
                self.commit_index.store(index, Ordering::Relaxed);
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    async fn is_leader(&self) -> bool {
        // Check if we won the election
        // For now, simplified version
        true
    }

    pub async fn apply_state_machine(&self) -> Result<()> {
        loop {
            let last_applied = self.last_applied.load(Ordering::Relaxed);
            let commit_index = self.commit_index.load(Ordering::Relaxed);

            if last_applied >= commit_index {
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }

            let log = self.log.read().await;
            if last_applied >= log.len() {
                continue;
            }

            let entry = &log[last_applied];

            // Apply command to state machine
            {
                let mut sm = self.state_machine.write().await;
                self.apply_command(&mut sm, &entry.command).await?;
            }

            self.last_applied.store(last_applied + 1, Ordering::Relaxed);
        }
    }

    async fn apply_command(&self, sm: &mut StateMachine, command: &Command) -> Result<()> {
        match command {
            Command::AgentJoined { agent_id, address } => {
                sm.agents.insert(
                    agent_id.clone(),
                    AgentState {
                        address: address.clone(),
                        ..Default::default()
                    }
                );
            }
            Command::AgentFailed { agent_id } => {
                sm.agents.remove(agent_id);
            }
            Command::TestCompleted { result } => {
                sm.completed_tests.insert(result.id.clone(), result.clone());
            }
            Command::PolicyUpdated { policy } => {
                sm.policies.push(policy.clone());
            }
            Command::ResourceAllocated { grant } => {
                sm.allocations.insert(grant.id.clone(), grant.clone());
            }
        }

        Ok(())
    }
}
```

---

## Gossip Protocol

```rust
// crates/clnrm-agents/src/consensus/gossip.rs

pub struct GossipProtocol {
    local_version: Arc<AtomicU64>,
    peers: Arc<DashMap<AgentId, PeerVersion>>,
    state_updates: Arc<DashMap<String, VersionedValue>>,
    comm: Arc<CommunicationLayer>,
}

#[derive(Debug, Clone)]
pub struct PeerVersion {
    agent_id: AgentId,
    version: u64,
    last_seen: SystemTime,
}

#[derive(Debug, Clone)]
pub struct VersionedValue {
    key: String,
    value: Vec<u8>,
    version: u64,
    timestamp: SystemTime,
}

impl GossipProtocol {
    pub async fn disseminate(&self, key: String, value: Vec<u8>) -> Result<()> {
        let version = self.local_version.fetch_add(1, Ordering::Relaxed);

        let update = VersionedValue {
            key: key.clone(),
            value,
            version,
            timestamp: SystemTime::now(),
        };

        self.state_updates.insert(key, update.clone());

        // Gossip to random peers
        let peers: Vec<_> = self.peers.iter()
            .map(|entry| entry.key().clone())
            .collect();

        for _ in 0..3 {  // Fanout: 3 random peers
            if let Some(peer) = peers.choose(&mut rand::thread_rng()) {
                self.gossip_to(peer, &update).await.ok();
            }
        }

        Ok(())
    }

    async fn gossip_to(&self, peer: &AgentId, update: &VersionedValue) -> Result<()> {
        // Send update to peer
        let message = Message::GossipUpdate {
            key: update.key.clone(),
            value: update.value.clone(),
            version: update.version,
        };

        self.comm.send_message(peer, message).await
    }

    pub async fn receive_gossip(&self, key: String, value: Vec<u8>, version: u64) -> Result<()> {
        // Apply update if newer than local version
        let should_apply = {
            let existing = self.state_updates.get(&key);
            existing.is_none() || existing.unwrap().version < version
        };

        if should_apply {
            let update = VersionedValue {
                key: key.clone(),
                value: value.clone(),
                version,
                timestamp: SystemTime::now(),
            };

            self.state_updates.insert(key.clone(), update);

            // Gossip to others
            self.disseminate(key, value).await?;
        }

        Ok(())
    }

    pub async fn anti_entropy(&self) {
        // Periodic sync to ensure eventual consistency
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;

            for peer_ref in self.peers.iter() {
                let peer = peer_ref.key().clone();
                let peer_version = peer_ref.value().version;

                // Send all updates newer than peer's version
                for entry in self.state_updates.iter() {
                    if entry.value().version > peer_version {
                        self.gossip_to(&peer, entry.value()).await.ok();
                    }
                }
            }
        }
    }
}
```

---

## Message Ordering & Delivery

```rust
pub struct ReliableMessaging {
    sent_messages: Arc<DashMap<MessageId, SentMessage>>,
    received_messages: Arc<DashSet<MessageId>>,
    retransmission_timeout: Duration,
    max_retries: u32,
}

#[derive(Debug, Clone)]
pub struct SentMessage {
    id: MessageId,
    recipient: AgentId,
    payload: Vec<u8>,
    sent_at: SystemTime,
    retry_count: u32,
}

impl ReliableMessaging {
    pub async fn send_reliable(
        &self,
        recipient: AgentId,
        message: Message,
    ) -> Result<()> {
        let id = MessageId::new();
        let payload = serde_json::to_vec(&message)?;

        let msg = SentMessage {
            id: id.clone(),
            recipient: recipient.clone(),
            payload: payload.clone(),
            sent_at: SystemTime::now(),
            retry_count: 0,
        };

        self.sent_messages.insert(id, msg);

        // Send message
        self.send_with_acks(&recipient, &payload).await?;

        Ok(())
    }

    async fn resend_unacked(&self) {
        loop {
            tokio::time::sleep(self.retransmission_timeout).await;

            let now = SystemTime::now();
            let mut to_resend = Vec::new();

            for entry in self.sent_messages.iter() {
                let msg = entry.value();
                if now.duration_since(msg.sent_at).ok()? > self.retransmission_timeout
                    && msg.retry_count < self.max_retries
                {
                    to_resend.push(msg.clone());
                }
            }

            for msg in to_resend {
                self.send_with_acks(&msg.recipient, &msg.payload).await.ok();

                if let Some(mut entry) = self.sent_messages.get_mut(&msg.id) {
                    entry.retry_count += 1;
                }
            }
        }
    }

    async fn send_with_acks(&self, recipient: &AgentId, payload: &[u8]) -> Result<()> {
        // Send and wait for ACK
        // (implementation details)
        Ok(())
    }

    pub async fn mark_received(&self, message_id: MessageId) {
        self.received_messages.insert(message_id.clone());
        self.sent_messages.remove(&message_id);
    }
}
```

---

## Network Partition Handling

```rust
pub struct PartitionDetector {
    peer_heartbeats: Arc<DashMap<AgentId, Instant>>,
    heartbeat_timeout: Duration,
    partition_grace_period: Duration,
}

impl PartitionDetector {
    pub async fn detect_partitions(&self) -> Result<Vec<Partition>> {
        let now = Instant::now();
        let mut unreachable = Vec::new();

        for entry in self.peer_heartbeats.iter() {
            let last_seen = entry.value().clone();
            if now.duration_since(last_seen) > self.heartbeat_timeout {
                unreachable.push(entry.key().clone());
            }
        }

        // Wait grace period before declaring partition
        if !unreachable.is_empty() {
            tokio::time::sleep(self.partition_grace_period).await;

            // Re-check
            let mut still_unreachable = Vec::new();
            for agent in unreachable {
                let last_seen = self.peer_heartbeats.get(&agent)
                    .map(|e| e.clone())
                    .unwrap_or_else(|| Instant::now());

                if now.duration_since(last_seen) > self.heartbeat_timeout {
                    still_unreachable.push(agent);
                }
            }

            if !still_unreachable.is_empty() {
                return Ok(vec![Partition {
                    unreachable_agents: still_unreachable,
                    partition_time: now,
                }]);
            }
        }

        Ok(Vec::new())
    }

    pub async fn handle_partition(&self, partition: Partition) -> Result<()> {
        // Strategy: Minority partition goes read-only
        // Majority partition continues serving

        let quorum_size = self.get_total_agents() / 2 + 1;
        let reachable = self.get_reachable_agents().len();

        if reachable < quorum_size {
            // We're in minority, go read-only
            self.set_read_only(true).await;
        } else {
            // We're in majority, continue normally
            // Remove unreachable agents from membership
            for agent in &partition.unreachable_agents {
                self.remove_agent(agent).await?;
            }
        }

        Ok(())
    }
}
```

---

## Performance & Observability

```rust
pub struct CommunicationMetrics {
    messages_sent: Arc<AtomicU64>,
    messages_received: Arc<AtomicU64>,
    messages_dropped: Arc<AtomicU64>,
    latency_ms: Arc<Histogram>,
    consensus_commits: Arc<AtomicU64>,
    consensus_latency_ms: Arc<Histogram>,
}

impl CommunicationMetrics {
    pub fn report(&self) -> CommunicationReport {
        CommunicationReport {
            messages_sent: self.messages_sent.load(Ordering::Relaxed),
            messages_received: self.messages_received.load(Ordering::Relaxed),
            messages_dropped: self.messages_dropped.load(Ordering::Relaxed),
            avg_latency_ms: self.latency_ms.mean(),
            p99_latency_ms: self.latency_ms.percentile(99),
            consensus_commits: self.consensus_commits.load(Ordering::Relaxed),
            avg_consensus_latency_ms: self.consensus_latency_ms.mean(),
        }
    }
}
```

---

## References

- Raft Consensus: https://raft.github.io/
- Gossip Protocols: Epidemic Information Dissemination
- Byzantine Fault Tolerance: PBFT Protocol
- gRPC: https://grpc.io/

---

**Version**: 1.0
**Last Updated**: 2025-11-18
