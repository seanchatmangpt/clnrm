use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Opaque identifier for a [`Packet`], backed by a UUID v4 string.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PacketId(pub String);

impl PacketId {
    fn new() -> Self {
        PacketId(uuid::Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for PacketId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The payload carried by a [`Packet`].
#[derive(Debug, Clone)]
pub enum PacketPayload {
    /// Raw binary data.
    Data(Vec<u8>),
    /// A reference to a settlement receipt by its ID.
    Receipt(String),
    /// An encoded instruction string.
    Instruction(String),
    /// Acknowledgment of a previously received packet.
    Acknowledgment { packet_id: String, accepted: bool },
}

impl PacketPayload {
    /// Serializes the payload to a canonical byte representation used for
    /// checksum computation.
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            PacketPayload::Data(bytes) => {
                let mut out = b"data:".to_vec();
                out.extend_from_slice(bytes);
                out
            }
            PacketPayload::Receipt(id) => {
                let mut out = b"receipt:".to_vec();
                out.extend_from_slice(id.as_bytes());
                out
            }
            PacketPayload::Instruction(instr) => {
                let mut out = b"instruction:".to_vec();
                out.extend_from_slice(instr.as_bytes());
                out
            }
            PacketPayload::Acknowledgment {
                packet_id,
                accepted,
            } => {
                let mut out = b"ack:".to_vec();
                out.extend_from_slice(packet_id.as_bytes());
                out.push(if *accepted { 1 } else { 0 });
                out
            }
        }
    }
}

/// An A2A (agent-to-agent) packet carrying a typed payload between agents.
#[derive(Debug, Clone)]
pub struct Packet {
    /// Unique identifier for this packet.
    pub id: PacketId,
    /// Identity of the sending agent.
    pub sender: String,
    /// Identity of the target agent.
    pub recipient: String,
    /// The packet's payload.
    pub payload: PacketPayload,
    /// Unix timestamp in milliseconds when the packet was created.
    pub created_at_ms: u64,
    /// Time-to-live in milliseconds; 0 means never expires.
    pub ttl_ms: u64,
    /// Monotonically increasing sequence number assigned by the router.
    pub sequence_num: u64,
    /// Arbitrary metadata headers.
    pub headers: HashMap<String, String>,
    /// SHA-256 checksum over `sender || recipient || payload_bytes || seq`.
    pub checksum: [u8; 32],
}

impl Packet {
    /// Creates a new `Packet`.  The checksum is computed immediately.
    /// `sequence_num` starts at 0 and is overwritten by [`PacketRouter::send`].
    pub fn new(sender: &str, recipient: &str, payload: PacketPayload, ttl_ms: u64) -> Self {
        let created_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let sequence_num = 0u64;
        let payload_bytes = payload.to_bytes();
        let checksum = Self::compute_checksum(sender, recipient, &payload_bytes, sequence_num);

        Packet {
            id: PacketId::new(),
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            payload,
            created_at_ms,
            ttl_ms,
            sequence_num,
            headers: HashMap::new(),
            checksum,
        }
    }

    /// Computes `SHA-256(sender || recipient || payload_bytes || seq.to_le_bytes())`.
    pub fn compute_checksum(
        sender: &str,
        recipient: &str,
        payload_bytes: &[u8],
        seq: u64,
    ) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(sender.as_bytes());
        hasher.update(recipient.as_bytes());
        hasher.update(payload_bytes);
        hasher.update(seq.to_le_bytes());
        hasher.finalize().into()
    }

    /// Returns `true` when `current_time_ms` is past `created_at_ms + ttl_ms`.
    /// A `ttl_ms` of 0 is treated as "never expires".
    pub fn is_expired(&self, current_time_ms: u64) -> bool {
        if self.ttl_ms == 0 {
            return false;
        }
        current_time_ms > self.created_at_ms.saturating_add(self.ttl_ms)
    }

    /// Returns `true` when the stored checksum matches a freshly computed one.
    pub fn is_valid(&self) -> bool {
        let payload_bytes = self.payload_bytes();
        let expected = Self::compute_checksum(
            &self.sender,
            &self.recipient,
            &payload_bytes,
            self.sequence_num,
        );
        self.checksum == expected
    }

    /// Builder-style method to attach a metadata header.
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// Returns the canonical byte representation of the payload.
    pub fn payload_bytes(&self) -> Vec<u8> {
        self.payload.to_bytes()
    }

    /// Recomputes the checksum for the current `sequence_num` and stores it.
    /// Called by [`PacketRouter::send`] after assigning the sequence number.
    fn refresh_checksum(&mut self) {
        let payload_bytes = self.payload_bytes();
        self.checksum = Self::compute_checksum(
            &self.sender,
            &self.recipient,
            &payload_bytes,
            self.sequence_num,
        );
    }
}

/// Routes packets between agents using per-recipient in-memory queues.
pub struct PacketRouter {
    /// Per-recipient queues.
    pub queues: HashMap<String, Vec<Packet>>,
    /// Count of packets successfully delivered (enqueued).
    pub delivered: u64,
    /// Count of packets dropped due to expiry or validation failure.
    pub dropped: u64,
    /// Monotonically increasing counter used to stamp sequence numbers.
    pub sequence_counter: u64,
}

impl PacketRouter {
    /// Creates a new, empty `PacketRouter`.
    pub fn new() -> Self {
        PacketRouter {
            queues: HashMap::new(),
            delivered: 0,
            dropped: 0,
            sequence_counter: 0,
        }
    }

    /// Assigns a sequence number to `packet`, validates it, and enqueues it.
    ///
    /// Returns the [`PacketId`] on success, or an error string when the packet
    /// is expired or fails checksum validation.
    pub fn send(&mut self, mut packet: Packet, current_time_ms: u64) -> Result<PacketId, String> {
        // Assign next sequence number and refresh the checksum.
        self.sequence_counter += 1;
        packet.sequence_num = self.sequence_counter;
        packet.refresh_checksum();

        if packet.is_expired(current_time_ms) {
            self.dropped += 1;
            return Err(format!(
                "Packet {} is already expired at send time",
                packet.id
            ));
        }

        if !packet.is_valid() {
            self.dropped += 1;
            return Err(format!("Packet {} failed checksum validation", packet.id));
        }

        let id = packet.id.clone();
        self.queues
            .entry(packet.recipient.clone())
            .or_default()
            .push(packet);
        self.delivered += 1;
        Ok(id)
    }

    /// Drains the queue for `recipient`, returning only non-expired packets.
    /// Expired packets are counted as dropped and discarded.
    pub fn receive(&mut self, recipient: &str, current_time_ms: u64) -> Vec<Packet> {
        let queue = match self.queues.get_mut(recipient) {
            Some(q) => q,
            None => return Vec::new(),
        };

        let mut live = Vec::new();
        let drained: Vec<Packet> = queue.drain(..).collect();

        for pkt in drained {
            if pkt.is_expired(current_time_ms) {
                self.dropped += 1;
            } else {
                live.push(pkt);
            }
        }

        live
    }

    /// Returns the number of packets currently queued for `recipient`.
    pub fn queue_depth(&self, recipient: &str) -> usize {
        self.queues.get(recipient).map_or(0, |q| q.len())
    }

    /// Returns the total number of packets that have been delivered (enqueued).
    pub fn total_delivered(&self) -> u64 {
        self.delivered
    }

    /// Returns the total number of packets that have been dropped.
    pub fn total_dropped(&self) -> u64 {
        self.dropped
    }
}

impl Default for PacketRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    #[test]
    fn test_packet_creation_and_validity() {
        let pkt = Packet::new(
            "agent-a",
            "agent-b",
            PacketPayload::Data(b"hello".to_vec()),
            5_000,
        );
        assert!(pkt.is_valid());
        assert!(!pkt.is_expired(now_ms()));
    }

    #[test]
    fn test_packet_expired() {
        let pkt = Packet::new(
            "agent-a",
            "agent-b",
            PacketPayload::Instruction("op:noop".to_string()),
            1, // 1 ms TTL
        );
        // Far in the future
        assert!(pkt.is_expired(pkt.created_at_ms + 1_000));
    }

    #[test]
    fn test_packet_never_expires_when_ttl_zero() {
        let pkt = Packet::new(
            "agent-a",
            "agent-b",
            PacketPayload::Receipt("rcpt-123".to_string()),
            0, // 0 = never expires
        );
        assert!(!pkt.is_expired(u64::MAX));
    }

    #[test]
    fn test_router_send_and_receive() {
        let mut router = PacketRouter::new();
        let t = now_ms();

        let pkt = Packet::new(
            "sender",
            "bob",
            PacketPayload::Data(b"payload".to_vec()),
            60_000,
        );
        let id = router.send(pkt, t).expect("send failed");
        assert_eq!(router.queue_depth("bob"), 1);
        assert_eq!(router.total_delivered(), 1);

        let received = router.receive("bob", t);
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].id, id);
        assert_eq!(router.queue_depth("bob"), 0);
    }

    #[test]
    fn test_router_drops_expired_at_receive() {
        let mut router = PacketRouter::new();
        let t = now_ms();

        let pkt = Packet::new(
            "sender",
            "carol",
            PacketPayload::Data(b"old".to_vec()),
            1, // 1 ms TTL
        );
        router.send(pkt, t).expect("send at creation time is fine");

        // Receive far in the future — packet should be dropped
        let received = router.receive("carol", t + 1_000);
        assert!(received.is_empty());
        assert_eq!(router.total_dropped(), 1);
    }

    #[test]
    fn test_router_drops_expired_at_send() {
        let mut router = PacketRouter::new();

        // Build a packet with a tiny TTL then send it "in the future"
        let pkt = Packet::new(
            "sender",
            "dave",
            PacketPayload::Acknowledgment {
                packet_id: "p-1".to_string(),
                accepted: true,
            },
            1,
        );
        let result = router.send(pkt, u64::MAX);
        assert!(result.is_err());
        assert_eq!(router.total_dropped(), 1);
    }

    #[test]
    fn test_with_header() {
        let pkt = Packet::new("a", "b", PacketPayload::Data(vec![]), 0)
            .with_header("content-type", "application/octet-stream");
        assert_eq!(pkt.headers["content-type"], "application/octet-stream");
    }

    #[test]
    fn test_sequence_numbers_increment() {
        let mut router = PacketRouter::new();
        let t = now_ms();

        for _ in 0..3 {
            let pkt = Packet::new("x", "y", PacketPayload::Data(vec![]), 60_000);
            router.send(pkt, t).unwrap();
        }

        let pkts = router.receive("y", t);
        assert_eq!(pkts[0].sequence_num, 1);
        assert_eq!(pkts[1].sequence_num, 2);
        assert_eq!(pkts[2].sequence_num, 3);
    }
}
