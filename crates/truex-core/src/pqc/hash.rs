//! Custom Cryptographic Hash function based on a Sponge construction.

/// A 32-byte digest result.
pub type Digest = [u8; 32];

/// A custom ARX-based permutation over 1024 bits (16 `u64` words).
fn custom_permute(state: &mut [u64; 16]) {
    macro_rules! mix {
        ($a:expr, $b:expr, $c:expr, $d:expr) => {
            $a = $a.wrapping_add($b);
            $d = ($d ^ $a).rotate_left(17);
            $c = $c.wrapping_add($d);
            $b = ($b ^ $c).rotate_left(29);
            $a = $a.wrapping_add($b);
            $d = ($d ^ $a).rotate_left(39);
            $c = $c.wrapping_add($d);
            $b = ($b ^ $c).rotate_left(43);
        };
    }

    const ROUNDS: u64 = 24;
    const CONSTANT: u64 = 0x9E3779B97F4A7C15;

    for r in 0..ROUNDS {
        // Mix columns
        mix!(state[0], state[4], state[8], state[12]);
        mix!(state[1], state[5], state[9], state[13]);
        mix!(state[2], state[6], state[10], state[14]);
        mix!(state[3], state[7], state[11], state[15]);

        // Mix diagonals
        mix!(state[0], state[5], state[10], state[15]);
        mix!(state[1], state[6], state[11], state[12]);
        mix!(state[2], state[7], state[8], state[13]);
        mix!(state[3], state[4], state[9], state[14]);

        // Add round constant to disrupt symmetry
        state[0] ^= CONSTANT.wrapping_add(r);
    }
}

/// Computes a 32-byte hash of the given arbitrary byte slice using a custom sponge construction.
pub fn custom_hash(input: &[u8]) -> Digest {
    let mut state = [0u64; 16];
    let rate_bytes = 64; // 512 bits = 8 u64s
    
    // Absorb phase
    let mut offset = 0;
    while offset < input.len() {
        let mut block = [0u8; 64];
        let remaining = input.len() - offset;
        let take = remaining.min(rate_bytes);
        
        block[..take].copy_from_slice(&input[offset..offset + take]);
        
        // Pad if it's the last block
        if take < rate_bytes {
            block[take] = 0x80; // simple padding
            // We'll leave the rest as zeros
        }
        
        // XOR block into the rate part of the state
        for i in 0..8 {
            let word_bytes = [
                block[i * 8], block[i * 8 + 1], block[i * 8 + 2], block[i * 8 + 3],
                block[i * 8 + 4], block[i * 8 + 5], block[i * 8 + 6], block[i * 8 + 7],
            ];
            state[i] ^= u64::from_le_bytes(word_bytes);
        }
        
        custom_permute(&mut state);
        
        offset += rate_bytes;
    }
    
    // If the input length was exactly a multiple of the rate,
    // we need to absorb an extra block containing just the padding.
    if input.len() % rate_bytes == 0 {
        let mut block = [0u8; 64];
        block[0] = 0x80;
        for i in 0..8 {
            let word_bytes = [
                block[i * 8], block[i * 8 + 1], block[i * 8 + 2], block[i * 8 + 3],
                block[i * 8 + 4], block[i * 8 + 5], block[i * 8 + 6], block[i * 8 + 7],
            ];
            state[i] ^= u64::from_le_bytes(word_bytes);
        }
        custom_permute(&mut state);
    }
    
    // Squeeze phase
    // We need 32 bytes, which is 4 u64s. Our rate is 8 u64s, so we have enough in the first squeeze.
    let mut out = [0u8; 32];
    for i in 0..4 {
        let bytes = state[i].to_le_bytes();
        out[i * 8..(i + 1) * 8].copy_from_slice(&bytes);
    }
    
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_hash_basic() {
        let input1 = b"hello world";
        let input2 = b"hello worle";
        
        let hash1 = custom_hash(input1);
        let hash2 = custom_hash(input2);
        
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_custom_hash_empty() {
        let hash_empty = custom_hash(b"");
        let hash_zero = custom_hash(&[0u8]);
        
        assert_ne!(hash_empty, hash_zero);
    }
    
    #[test]
    fn test_custom_hash_long() {
        let input = [0xABu8; 150];
        let hash = custom_hash(&input);
        // Just verify it doesn't panic and produces something.
        assert_ne!(hash, [0u8; 32]);
    }
}
