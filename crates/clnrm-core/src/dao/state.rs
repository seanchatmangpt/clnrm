use crate::pqc::hash::custom_hash;

/// Converts a byte slice into a vector of nibbles (half-bytes).
fn bytes_to_nibbles(bytes: &[u8]) -> Vec<u8> {
    let mut nibbles = Vec::with_capacity(bytes.len() * 2);
    for &b in bytes {
        nibbles.push(b >> 4);
        nibbles.push(b & 0x0f);
    }
    nibbles
}

/// Returns the length of the common prefix of two slices.
fn common_prefix(a: &[u8], b: &[u8]) -> usize {
    a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count()
}

#[derive(Clone, Debug, Default)]
pub enum Node {
    #[default]
    Empty,
    Leaf {
        path: Vec<u8>,
        value: Vec<u8>,
    },
    Extension {
        path: Vec<u8>,
        child: Box<Node>,
    },
    Branch {
        children: Box<[Option<Node>; 16]>,
        value: Option<Vec<u8>>,
    },
}

impl Node {
    fn insert(self, path: &[u8], value: &[u8]) -> Self {
        match self {
            Node::Empty => Node::Leaf {
                path: path.to_vec(),
                value: value.to_vec(),
            },
            Node::Leaf {
                path: leaf_path,
                value: leaf_value,
            } => {
                let cp = common_prefix(&leaf_path, path);
                if cp == leaf_path.len() && cp == path.len() {
                    // Update exact match
                    Node::Leaf {
                        path: leaf_path,
                        value: value.to_vec(),
                    }
                } else {
                    let mut branch_children: Box<[Option<Node>; 16]> =
                        Box::new(std::array::from_fn(|_| None));
                    let mut branch_value = None;

                    if cp == leaf_path.len() {
                        branch_value = Some(leaf_value);
                    } else {
                        let leaf_idx = leaf_path[cp] as usize;
                        let new_leaf_path = leaf_path[cp + 1..].to_vec();
                        branch_children[leaf_idx] = Some(Node::Leaf {
                            path: new_leaf_path,
                            value: leaf_value,
                        });
                    }

                    if cp == path.len() {
                        branch_value = Some(value.to_vec());
                    } else {
                        let path_idx = path[cp] as usize;
                        let new_path = path[cp + 1..].to_vec();
                        branch_children[path_idx] = Some(Node::Leaf {
                            path: new_path,
                            value: value.to_vec(),
                        });
                    }

                    let branch = Node::Branch {
                        children: branch_children,
                        value: branch_value,
                    };

                    if cp > 0 {
                        Node::Extension {
                            path: leaf_path[..cp].to_vec(),
                            child: Box::new(branch),
                        }
                    } else {
                        branch
                    }
                }
            }
            Node::Extension {
                path: ext_path,
                child,
            } => {
                let cp = common_prefix(&ext_path, path);
                if cp == ext_path.len() {
                    let new_child = child.insert(&path[cp..], value);
                    Node::Extension {
                        path: ext_path,
                        child: Box::new(new_child),
                    }
                } else {
                    let mut branch_children: Box<[Option<Node>; 16]> =
                        Box::new(std::array::from_fn(|_| None));
                    let mut branch_value = None;

                    let ext_idx = ext_path[cp] as usize;
                    let new_ext_path = ext_path[cp + 1..].to_vec();
                    if new_ext_path.is_empty() {
                        branch_children[ext_idx] = Some(*child);
                    } else {
                        branch_children[ext_idx] = Some(Node::Extension {
                            path: new_ext_path,
                            child,
                        });
                    }

                    if cp == path.len() {
                        branch_value = Some(value.to_vec());
                    } else {
                        let path_idx = path[cp] as usize;
                        let new_path = path[cp + 1..].to_vec();
                        branch_children[path_idx] = Some(Node::Leaf {
                            path: new_path,
                            value: value.to_vec(),
                        });
                    }

                    let branch = Node::Branch {
                        children: branch_children,
                        value: branch_value,
                    };

                    if cp > 0 {
                        Node::Extension {
                            path: ext_path[..cp].to_vec(),
                            child: Box::new(branch),
                        }
                    } else {
                        branch
                    }
                }
            }
            Node::Branch {
                mut children,
                value: mut branch_value,
            } => {
                if path.is_empty() {
                    branch_value = Some(value.to_vec());
                } else {
                    let idx = path[0] as usize;
                    let child_node = children[idx].take().unwrap_or(Node::Empty);
                    children[idx] = Some(child_node.insert(&path[1..], value));
                }
                Node::Branch {
                    children,
                    value: branch_value,
                }
            }
        }
    }

    fn get(&self, path: &[u8]) -> Option<Vec<u8>> {
        match self {
            Node::Empty => None,
            Node::Leaf {
                path: leaf_path,
                value,
            } => {
                if leaf_path == path {
                    Some(value.clone())
                } else {
                    None
                }
            }
            Node::Extension {
                path: ext_path,
                child,
            } => {
                if path.starts_with(ext_path) {
                    child.get(&path[ext_path.len()..])
                } else {
                    None
                }
            }
            Node::Branch { children, value } => {
                if path.is_empty() {
                    value.clone()
                } else {
                    let idx = path[0] as usize;
                    if let Some(child) = &children[idx] {
                        child.get(&path[1..])
                    } else {
                        None
                    }
                }
            }
        }
    }

    fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();
        match self {
            Node::Empty => {
                out.push(0);
            }
            Node::Leaf { path, value } => {
                out.push(1);
                out.extend_from_slice(&(path.len() as u32).to_le_bytes());
                out.extend_from_slice(path);
                out.extend_from_slice(&(value.len() as u32).to_le_bytes());
                out.extend_from_slice(value);
            }
            Node::Extension { path, child } => {
                out.push(2);
                out.extend_from_slice(&(path.len() as u32).to_le_bytes());
                out.extend_from_slice(path);
                out.extend_from_slice(&child.hash());
            }
            Node::Branch { children, value } => {
                out.push(3);
                for child_opt in children.iter() {
                    if let Some(child) = child_opt {
                        out.extend_from_slice(&child.hash());
                    } else {
                        out.extend_from_slice(&[0u8; 32]);
                    }
                }
                if let Some(v) = value {
                    out.push(1);
                    out.extend_from_slice(&(v.len() as u32).to_le_bytes());
                    out.extend_from_slice(v);
                } else {
                    out.push(0);
                }
            }
        }
        out
    }

    fn hash(&self) -> [u8; 32] {
        let serialized = self.serialize();
        custom_hash(&serialized)
    }
}

pub struct Trie {
    root: Node,
}

impl Default for Trie {
    fn default() -> Self {
        Self::new()
    }
}

impl Trie {
    pub fn new() -> Self {
        Self { root: Node::Empty }
    }

    pub fn insert(&mut self, key: &[u8], value: &[u8]) {
        let nibbles = bytes_to_nibbles(key);
        let root = std::mem::take(&mut self.root);
        self.root = root.insert(&nibbles, value);
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let nibbles = bytes_to_nibbles(key);
        self.root.get(&nibbles)
    }

    pub fn state_root(&self) -> [u8; 32] {
        self.root.hash()
    }
}
