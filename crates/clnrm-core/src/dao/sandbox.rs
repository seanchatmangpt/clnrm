use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxError {
    MemoryLimitExceeded { attempted: usize, limit: usize },
    IllegalMemoryAccess { address: usize, limit: usize },
    CapabilityExceeded(&'static str),
}

impl fmt::Display for SandboxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SandboxError::MemoryLimitExceeded { attempted, limit } => write!(
                f,
                "Memory limit exceeded: attempted to allocate {} bytes, limit is {}",
                attempted, limit
            ),
            SandboxError::IllegalMemoryAccess { address, limit } => write!(
                f,
                "Illegal memory access at address {}, sandbox memory size is {}",
                address, limit
            ),
            SandboxError::CapabilityExceeded(cap) => write!(f, "Capability exceeded: {}", cap),
        }
    }
}

impl std::error::Error for SandboxError {}

#[derive(Debug, Clone)]
pub struct CapabilityLimits {
    pub max_memory_bytes: usize,
    pub allow_network: bool,
    pub allow_file_io: bool,
    pub max_execution_steps: usize,
}

impl Default for CapabilityLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 1024 * 1024, // 1MB default
            allow_network: false,
            allow_file_io: false,
            max_execution_steps: 10_000,
        }
    }
}

pub struct VmSandbox {
    memory: Vec<u8>,
    limits: CapabilityLimits,
    execution_steps: usize,
}

impl VmSandbox {
    pub fn new(limits: CapabilityLimits) -> Result<Self, SandboxError> {
        Ok(Self {
            memory: Vec::new(),
            limits,
            execution_steps: 0,
        })
    }

    pub fn allocate_memory(&mut self, size: usize) -> Result<(), SandboxError> {
        let new_size = self.memory.len().saturating_add(size);
        if new_size > self.limits.max_memory_bytes {
            return Err(SandboxError::MemoryLimitExceeded {
                attempted: new_size,
                limit: self.limits.max_memory_bytes,
            });
        }
        self.memory.resize(new_size, 0);
        Ok(())
    }

    pub fn read_memory(&self, address: usize, size: usize) -> Result<&[u8], SandboxError> {
        let end_address = address.saturating_add(size);
        if end_address > self.memory.len() {
            return Err(SandboxError::IllegalMemoryAccess {
                address: end_address.saturating_sub(1),
                limit: self.memory.len(),
            });
        }
        Ok(&self.memory[address..end_address])
    }

    pub fn write_memory(&mut self, address: usize, data: &[u8]) -> Result<(), SandboxError> {
        let end_address = address.saturating_add(data.len());
        if end_address > self.memory.len() {
            return Err(SandboxError::IllegalMemoryAccess {
                address: end_address.saturating_sub(1),
                limit: self.memory.len(),
            });
        }
        self.memory[address..end_address].copy_from_slice(data);
        Ok(())
    }

    pub fn execute_step(&mut self) -> Result<(), SandboxError> {
        self.execution_steps = self.execution_steps.saturating_add(1);
        if self.execution_steps > self.limits.max_execution_steps {
            return Err(SandboxError::CapabilityExceeded(
                "Max execution steps reached",
            ));
        }
        Ok(())
    }

    pub fn request_network(&self) -> Result<(), SandboxError> {
        if !self.limits.allow_network {
            return Err(SandboxError::CapabilityExceeded(
                "Network access is not allowed",
            ));
        }
        Ok(())
    }

    pub fn request_file_io(&self) -> Result<(), SandboxError> {
        if !self.limits.allow_file_io {
            return Err(SandboxError::CapabilityExceeded("File I/O is not allowed"));
        }
        Ok(())
    }
}
