use std::collections::BTreeMap;

/// Opcodes for the Smart Contract Virtual Machine.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    /// Halt execution.
    Stop = 0x00,
    /// Push an 8-byte value onto the stack.
    Push = 0x01,
    /// Pop a value from the stack.
    Pop = 0x02,
    /// Add the top two values on the stack.
    Add = 0x03,
    /// Subtract the top value from the second-top value on the stack.
    Sub = 0x04,
    /// Load a value from memory using the address on top of the stack.
    Load = 0x05,
    /// Store a value into memory using the address and value on top of the stack.
    Store = 0x06,
    /// Jump to the program counter specified on top of the stack.
    Jump = 0x07,
    /// Jump to the program counter specified by the first stack item, if the second is non-zero.
    Jumpi = 0x08,
}

impl TryFrom<u8> for Opcode {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Opcode::Stop),
            0x01 => Ok(Opcode::Push),
            0x02 => Ok(Opcode::Pop),
            0x03 => Ok(Opcode::Add),
            0x04 => Ok(Opcode::Sub),
            0x05 => Ok(Opcode::Load),
            0x06 => Ok(Opcode::Store),
            0x07 => Ok(Opcode::Jump),
            0x08 => Ok(Opcode::Jumpi),
            _ => Err(format!("Invalid opcode: {:#04x}", value)),
        }
    }
}

/// A deterministic, stack-based bytecode Virtual Machine.
#[derive(Debug, Clone, Default)]
pub struct Vm {
    stack: Vec<u64>,
    memory: BTreeMap<u64, u64>,
}

impl Vm {
    /// Creates a new, empty Virtual Machine.
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            memory: BTreeMap::new(),
        }
    }

    /// Retrieves the current stack state.
    pub fn stack(&self) -> &[u64] {
        &self.stack
    }

    /// Retrieves the current memory state.
    pub fn memory(&self) -> &BTreeMap<u64, u64> {
        &self.memory
    }

    /// Executes a given sequence of bytecode.
    pub fn execute(&mut self, code: &[u8]) -> Result<(), String> {
        let mut pc = 0;
        
        while pc < code.len() {
            let op_byte = code[pc];
            let opcode = Opcode::try_from(op_byte)?;

            match opcode {
                Opcode::Stop => {
                    break;
                }
                Opcode::Push => {
                    if pc + 8 >= code.len() {
                        return Err("Unexpected end of code for PUSH instruction".to_string());
                    }
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&code[pc + 1..pc + 9]);
                    let value = u64::from_be_bytes(bytes);
                    self.stack.push(value);
                    pc += 8;
                }
                Opcode::Pop => {
                    self.stack.pop().ok_or("Stack underflow on POP")?;
                }
                Opcode::Add => {
                    let b = self.stack.pop().ok_or("Stack underflow on ADD (value b)")?;
                    let a = self.stack.pop().ok_or("Stack underflow on ADD (value a)")?;
                    self.stack.push(a.wrapping_add(b));
                }
                Opcode::Sub => {
                    let b = self.stack.pop().ok_or("Stack underflow on SUB (value b)")?;
                    let a = self.stack.pop().ok_or("Stack underflow on SUB (value a)")?;
                    self.stack.push(a.wrapping_sub(b));
                }
                Opcode::Load => {
                    let addr = self.stack.pop().ok_or("Stack underflow on LOAD")?;
                    let val = self.memory.get(&addr).copied().unwrap_or(0);
                    self.stack.push(val);
                }
                Opcode::Store => {
                    let addr = self.stack.pop().ok_or("Stack underflow on STORE (address)")?;
                    let val = self.stack.pop().ok_or("Stack underflow on STORE (value)")?;
                    self.memory.insert(addr, val);
                }
                Opcode::Jump => {
                    let dest = self.stack.pop().ok_or("Stack underflow on JUMP")?;
                    if dest as usize >= code.len() {
                        return Err(format!("Invalid JUMP destination: {}", dest));
                    }
                    pc = dest as usize;
                    continue; // Skip the default pc += 1
                }
                Opcode::Jumpi => {
                    let dest = self.stack.pop().ok_or("Stack underflow on JUMPI (destination)")?;
                    let cond = self.stack.pop().ok_or("Stack underflow on JUMPI (condition)")?;
                    if cond != 0 {
                        if dest as usize >= code.len() {
                            return Err(format!("Invalid JUMPI destination: {}", dest));
                        }
                        pc = dest as usize;
                        continue; // Skip the default pc += 1
                    }
                }
            }
            pc += 1;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_pop() {
        let mut vm = Vm::new();
        // PUSH 42
        let mut code = vec![Opcode::Push as u8];
        code.extend_from_slice(&42u64.to_be_bytes());
        // POP
        code.push(Opcode::Pop as u8);

        vm.execute(&code).unwrap();
        assert_eq!(vm.stack().len(), 0);
    }

    #[test]
    fn test_add() {
        let mut vm = Vm::new();
        let mut code = vec![Opcode::Push as u8];
        code.extend_from_slice(&10u64.to_be_bytes());
        code.push(Opcode::Push as u8);
        code.extend_from_slice(&20u64.to_be_bytes());
        code.push(Opcode::Add as u8);

        vm.execute(&code).unwrap();
        assert_eq!(vm.stack().len(), 1);
        assert_eq!(vm.stack()[0], 30);
    }

    #[test]
    fn test_memory_load_store() {
        let mut vm = Vm::new();
        // Stack layout for STORE: [..., val, addr] (top is addr)
        // Let's store value 100 at address 50
        
        // PUSH 100 (value)
        let mut code = vec![Opcode::Push as u8];
        code.extend_from_slice(&100u64.to_be_bytes());
        // PUSH 50 (address)
        code.push(Opcode::Push as u8);
        code.extend_from_slice(&50u64.to_be_bytes());
        // STORE
        code.push(Opcode::Store as u8);

        // PUSH 50 (address to load from)
        code.push(Opcode::Push as u8);
        code.extend_from_slice(&50u64.to_be_bytes());
        // LOAD
        code.push(Opcode::Load as u8);

        vm.execute(&code).unwrap();
        assert_eq!(vm.stack().len(), 1);
        assert_eq!(vm.stack()[0], 100);
        assert_eq!(vm.memory().get(&50), Some(&100));
    }
}
