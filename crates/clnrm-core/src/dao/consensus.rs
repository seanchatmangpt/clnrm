use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub enum PbftError {
    InvalidSignature,
    ViewMismatch,
    SequenceMismatch,
    DigestMismatch,
    InvalidStateTransition,
    UnauthorizedNode,
    InsufficientVotes,
}

impl std::fmt::Display for PbftError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PbftError::InvalidSignature => write!(f, "Invalid signature"),
            PbftError::ViewMismatch => write!(f, "View mismatch"),
            PbftError::SequenceMismatch => write!(f, "Sequence number mismatch"),
            PbftError::DigestMismatch => write!(f, "Digest mismatch"),
            PbftError::InvalidStateTransition => write!(f, "Invalid state transition"),
            PbftError::UnauthorizedNode => write!(f, "Message from unauthorized node"),
            PbftError::InsufficientVotes => write!(f, "Insufficient votes"),
        }
    }
}

impl std::error::Error for PbftError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Digest(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature(pub Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrePrepare {
    pub view: u64,
    pub sequence: u64,
    pub digest: Digest,
    pub signature: Signature,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prepare {
    pub view: u64,
    pub sequence: u64,
    pub digest: Digest,
    pub node_id: NodeId,
    pub signature: Signature,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commit {
    pub view: u64,
    pub sequence: u64,
    pub digest: Digest,
    pub node_id: NodeId,
    pub signature: Signature,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PbftMessage {
    PrePrepare(PrePrepare),
    Prepare(Prepare),
    Commit(Commit),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PbftState {
    Init,
    PrePrepared {
        pre_prepare: PrePrepare,
    },
    Prepared {
        pre_prepare: PrePrepare,
        prepares: HashMap<NodeId, Prepare>,
    },
    Committed {
        pre_prepare: PrePrepare,
        prepares: HashMap<NodeId, Prepare>,
        commits: HashMap<NodeId, Commit>,
    },
    Executed {
        digest: Digest,
    },
}

pub trait CryptoProvider {
    fn verify_signature(&self, node_id: &NodeId, data: &[u8], signature: &Signature) -> bool;
    fn sign(&self, data: &[u8]) -> Signature;
    fn node_id(&self) -> NodeId;
}

pub struct PbftStateMachine<C: CryptoProvider> {
    state: PbftState,
    view: u64,
    sequence: u64,
    validators: HashSet<NodeId>,
    crypto: C,
}

impl<C: CryptoProvider> PbftStateMachine<C> {
    pub fn new(view: u64, sequence: u64, validators: HashSet<NodeId>, crypto: C) -> Self {
        Self {
            state: PbftState::Init,
            view,
            sequence,
            validators,
            crypto,
        }
    }

    pub fn fault_tolerance(&self) -> usize {
        let n = self.validators.len();
        if n < 4 {
            0 // Requires at least 4 nodes to tolerate 1 Byzantine fault
        } else {
            (n - 1) / 3
        }
    }

    pub fn required_votes(&self) -> usize {
        2 * self.fault_tolerance() + 1
    }

    pub fn state(&self) -> &PbftState {
        &self.state
    }

    fn verify_message_signature(
        &self,
        node_id: &NodeId,
        msg: &PbftMessage,
        signature: &Signature,
    ) -> bool {
        let data = match msg {
            PbftMessage::PrePrepare(pp) => {
                let mut data = Vec::new();
                data.extend_from_slice(&pp.view.to_le_bytes());
                data.extend_from_slice(&pp.sequence.to_le_bytes());
                data.extend_from_slice(&pp.digest.0);
                data
            }
            PbftMessage::Prepare(p) => {
                let mut data = Vec::new();
                data.extend_from_slice(&p.view.to_le_bytes());
                data.extend_from_slice(&p.sequence.to_le_bytes());
                data.extend_from_slice(&p.digest.0);
                data
            }
            PbftMessage::Commit(c) => {
                let mut data = Vec::new();
                data.extend_from_slice(&c.view.to_le_bytes());
                data.extend_from_slice(&c.sequence.to_le_bytes());
                data.extend_from_slice(&c.digest.0);
                data
            }
        };
        self.crypto.verify_signature(node_id, &data, signature)
    }

    pub fn process_message(&mut self, msg: PbftMessage) -> Result<Option<PbftMessage>, PbftError> {
        match msg {
            PbftMessage::PrePrepare(pp) => self.process_pre_prepare(pp),
            PbftMessage::Prepare(p) => self.process_prepare(p),
            PbftMessage::Commit(c) => self.process_commit(c),
        }
    }

    fn process_pre_prepare(&mut self, pp: PrePrepare) -> Result<Option<PbftMessage>, PbftError> {
        if self.state != PbftState::Init {
            return Err(PbftError::InvalidStateTransition);
        }

        if pp.view != self.view {
            return Err(PbftError::ViewMismatch);
        }

        if pp.sequence != self.sequence {
            return Err(PbftError::SequenceMismatch);
        }

        self.state = PbftState::PrePrepared {
            pre_prepare: pp.clone(),
        };

        let mut data = Vec::new();
        data.extend_from_slice(&self.view.to_le_bytes());
        data.extend_from_slice(&self.sequence.to_le_bytes());
        data.extend_from_slice(&pp.digest.0);

        let signature = self.crypto.sign(&data);

        let prepare_msg = Prepare {
            view: self.view,
            sequence: self.sequence,
            digest: pp.digest,
            node_id: self.crypto.node_id(),
            signature,
        };

        Ok(Some(PbftMessage::Prepare(prepare_msg)))
    }

    fn process_prepare(&mut self, p: Prepare) -> Result<Option<PbftMessage>, PbftError> {
        if !self.validators.contains(&p.node_id) {
            return Err(PbftError::UnauthorizedNode);
        }

        if p.view != self.view {
            return Err(PbftError::ViewMismatch);
        }

        if p.sequence != self.sequence {
            return Err(PbftError::SequenceMismatch);
        }

        if !self.verify_message_signature(
            &p.node_id,
            &PbftMessage::Prepare(p.clone()),
            &p.signature,
        ) {
            return Err(PbftError::InvalidSignature);
        }

        let new_state = match &mut self.state {
            PbftState::PrePrepared { pre_prepare } => {
                if pre_prepare.digest != p.digest {
                    return Err(PbftError::DigestMismatch);
                }
                let mut prepares = HashMap::new();
                prepares.insert(p.node_id.clone(), p.clone());
                Some(PbftState::Prepared {
                    pre_prepare: pre_prepare.clone(),
                    prepares,
                })
            }
            PbftState::Prepared {
                pre_prepare,
                prepares,
            } => {
                if pre_prepare.digest != p.digest {
                    return Err(PbftError::DigestMismatch);
                }
                prepares.insert(p.node_id.clone(), p.clone());
                None
            }
            _ => return Err(PbftError::InvalidStateTransition),
        };

        if let Some(state) = new_state {
            self.state = state;
        }

        if let PbftState::Prepared {
            pre_prepare,
            prepares,
        } = &self.state
        {
            if prepares.len() >= self.required_votes() {
                let mut data = Vec::new();
                data.extend_from_slice(&self.view.to_le_bytes());
                data.extend_from_slice(&self.sequence.to_le_bytes());
                data.extend_from_slice(&pre_prepare.digest.0);

                let signature = self.crypto.sign(&data);

                let commit_msg = Commit {
                    view: self.view,
                    sequence: self.sequence,
                    digest: pre_prepare.digest.clone(),
                    node_id: self.crypto.node_id(),
                    signature,
                };

                self.state = PbftState::Committed {
                    pre_prepare: pre_prepare.clone(),
                    prepares: prepares.clone(),
                    commits: HashMap::new(),
                };

                return Ok(Some(PbftMessage::Commit(commit_msg)));
            }
        }

        Ok(None)
    }

    fn process_commit(&mut self, c: Commit) -> Result<Option<PbftMessage>, PbftError> {
        if !self.validators.contains(&c.node_id) {
            return Err(PbftError::UnauthorizedNode);
        }

        if c.view != self.view {
            return Err(PbftError::ViewMismatch);
        }

        if c.sequence != self.sequence {
            return Err(PbftError::SequenceMismatch);
        }

        if !self.verify_message_signature(&c.node_id, &PbftMessage::Commit(c.clone()), &c.signature)
        {
            return Err(PbftError::InvalidSignature);
        }

        let new_state = match &mut self.state {
            PbftState::Committed {
                pre_prepare,
                prepares,
                commits,
            } => {
                if pre_prepare.digest != c.digest {
                    return Err(PbftError::DigestMismatch);
                }
                commits.insert(c.node_id.clone(), c.clone());
                None
            }
            PbftState::Prepared {
                pre_prepare,
                prepares,
            } => {
                if pre_prepare.digest != c.digest {
                    return Err(PbftError::DigestMismatch);
                }
                let mut commits = HashMap::new();
                commits.insert(c.node_id.clone(), c.clone());
                Some(PbftState::Committed {
                    pre_prepare: pre_prepare.clone(),
                    prepares: prepares.clone(),
                    commits,
                })
            }
            _ => return Err(PbftError::InvalidStateTransition),
        };

        if let Some(state) = new_state {
            self.state = state;
        }

        if let PbftState::Committed {
            pre_prepare,
            prepares: _,
            commits,
        } = &self.state
        {
            if commits.len() >= self.required_votes() {
                let digest = pre_prepare.digest.clone();
                self.state = PbftState::Executed { digest };
                return Ok(None);
            }
        }

        Ok(None)
    }
}
