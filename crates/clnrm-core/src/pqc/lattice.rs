//! A math-complete Post-Quantum Lattice-based signature scheme from scratch.
//! This implements a "miniature Dilithium" (Fiat-Shamir with Aborts) over
//! the polynomial ring Z_q[X] / (X^N + 1).

pub const N: usize = 64;
pub const Q: i64 = 8388609; // A modulus large enough to avoid hints
pub const D: i64 = 131072;
pub const GAMMA: i64 = 524288;
pub const TAU: usize = 10; // Number of non-zero coefficients in challenge

/// Reduces an integer modulo Q into the canonical range [-Q/2, Q/2].
#[inline]
fn reduce(x: i64) -> i64 {
    let mut r = x % Q;
    if r > Q / 2 {
        r -= Q;
    } else if r < -Q / 2 {
        r += Q;
    }
    r
}

/// A polynomial in the ring Z_q[X] / (X^N + 1).
#[derive(Clone, Debug, PartialEq)]
pub struct Poly {
    pub coeffs: [i64; N],
}

impl Poly {
    /// Creates a zero polynomial.
    pub fn zero() -> Self {
        Poly { coeffs: [0; N] }
    }

    /// Adds two polynomials.
    pub fn add(&self, other: &Poly) -> Self {
        let mut r = Poly::zero();
        for i in 0..N {
            r.coeffs[i] = reduce(self.coeffs[i] + other.coeffs[i]);
        }
        r
    }

    /// Subtracts `other` from `self`.
    pub fn sub(&self, other: &Poly) -> Self {
        let mut r = Poly::zero();
        for i in 0..N {
            r.coeffs[i] = reduce(self.coeffs[i] - other.coeffs[i]);
        }
        r
    }

    /// Multiplies two polynomials modulo X^N + 1.
    pub fn mul(&self, other: &Poly) -> Self {
        let mut r = [0i64; N];
        for i in 0..N {
            for j in 0..N {
                let prod = (self.coeffs[i] * other.coeffs[j]) % Q;
                if i + j < N {
                    r[i + j] = (r[i + j] + prod) % Q;
                } else {
                    r[i + j - N] = (r[i + j - N] - prod) % Q;
                }
            }
        }
        let mut poly = Poly::zero();
        for (i, &val) in r.iter().enumerate() {
            poly.coeffs[i] = reduce(val);
        }
        poly
    }

    /// Computes the infinity norm (max absolute coefficient).
    pub fn norm_infty(&self) -> i64 {
        let mut max = 0;
        for &x in &self.coeffs {
            let abs_x = x.abs();
            if abs_x > max {
                max = abs_x;
            }
        }
        max
    }

    /// Extracts the high bits of each coefficient.
    pub fn high_bits(&self) -> [i64; N] {
        let mut out = [0i64; N];
        for (out_val, &coeff) in out.iter_mut().zip(self.coeffs.iter()) {
            let mut pos_x = coeff % Q;
            if pos_x < 0 {
                pos_x += Q;
            }
            *out_val = pos_x / D;
        }
        out
    }

    /// Checks if a polynomial is a valid ternary challenge of expected weight.
    pub fn is_valid_challenge(&self) -> bool {
        let mut weight = 0;
        for &x in &self.coeffs {
            if x == 1 || x == -1 {
                weight += 1;
            } else if x != 0 {
                return false;
            }
        }
        weight == TAU
    }
}

// --- SHA-256 Implementation (No external crates!) ---

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

struct Sha256 {
    state: [u32; 8],
    data: [u8; 64],
    len: usize,
    bitlen: u64,
}

impl Sha256 {
    fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            data: [0; 64],
            len: 0,
            bitlen: 0,
        }
    }

    fn transform(&mut self) {
        let mut m = [0u32; 64];
        for (i, chunk) in self.data.chunks_exact(4).enumerate().take(16) {
            m[i] = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        for i in 16..64 {
            let s0 = m[i - 15].rotate_right(7) ^ m[i - 15].rotate_right(18) ^ (m[i - 15] >> 3);
            let s1 = m[i - 2].rotate_right(17) ^ m[i - 2].rotate_right(19) ^ (m[i - 2] >> 10);
            m[i] = m[i - 16]
                .wrapping_add(s0)
                .wrapping_add(m[i - 7])
                .wrapping_add(s1);
        }

        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];
        let mut e = self.state[4];
        let mut f = self.state[5];
        let mut g = self.state[6];
        let mut h = self.state[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[i])
                .wrapping_add(m[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }

    fn update(&mut self, data: &[u8]) {
        for &byte in data {
            self.data[self.len] = byte;
            self.len += 1;
            if self.len == 64 {
                self.transform();
                self.bitlen += 512;
                self.len = 0;
            }
        }
    }

    fn finalize(mut self) -> [u8; 32] {
        let i = self.len;
        self.data[i] = 0x80;
        self.len += 1;
        if self.len > 56 {
            while self.len < 64 {
                self.data[self.len] = 0;
                self.len += 1;
            }
            self.transform();
            self.len = 0;
        }
        while self.len < 56 {
            self.data[self.len] = 0;
            self.len += 1;
        }
        self.bitlen += (i as u64) * 8;
        self.data[56..64].copy_from_slice(&self.bitlen.to_be_bytes());
        self.transform();

        let mut out = [0u8; 32];
        for i in 0..8 {
            out[i * 4..i * 4 + 4].copy_from_slice(&self.state[i].to_be_bytes());
        }
        out
    }
}

// --- Pseudo Random Number Generator ---

pub struct Prng {
    seed: [u8; 32],
    counter: u64,
    buf: [u8; 32],
    buf_pos: usize,
}

impl Prng {
    pub fn new(seed: [u8; 32]) -> Self {
        Prng {
            seed,
            counter: 0,
            buf: [0; 32],
            buf_pos: 32,
        }
    }

    pub fn next_u8(&mut self) -> u8 {
        if self.buf_pos == 32 {
            let mut hasher = Sha256::new();
            hasher.update(&self.seed);
            hasher.update(&self.counter.to_le_bytes());
            self.buf = hasher.finalize();
            self.counter += 1;
            self.buf_pos = 0;
        }
        let res = self.buf[self.buf_pos];
        self.buf_pos += 1;
        res
    }

    pub fn next_poly_bounded(&mut self, bound: i64) -> Poly {
        let mut p = Poly::zero();
        let range = (2 * bound + 1) as u64;
        for i in 0..N {
            let mut val = 0u64;
            for _ in 0..8 {
                val = (val << 8) | (self.next_u8() as u64);
            }
            p.coeffs[i] = (val % range) as i64 - bound;
        }
        p
    }

    pub fn next_poly_q(&mut self) -> Poly {
        self.next_poly_bounded(Q / 2)
    }

    pub fn next_poly_ternary(&mut self) -> Poly {
        self.next_poly_bounded(1)
    }
}

// --- Signature Scheme Algorithms ---

#[derive(Clone, Debug)]
pub struct PublicKey {
    pub a: Poly,
    pub t: Poly,
}

#[derive(Clone, Debug)]
pub struct PrivateKey {
    pub pub_key: PublicKey,
    pub s1: Poly,
    pub s2: Poly,
}

#[derive(Clone, Debug)]
pub struct Signature {
    pub z: Poly,
    pub c: Poly,
}

pub struct KeyPair {
    pub public: PublicKey,
    pub secret: PrivateKey,
}

/// Generates a public/private keypair from a seed.
pub fn generate_keypair(seed: [u8; 32]) -> KeyPair {
    let mut prng = Prng::new(seed);
    let a = prng.next_poly_q();
    let s1 = prng.next_poly_ternary();
    let s2 = prng.next_poly_ternary();

    let as1 = a.mul(&s1);
    let t = as1.add(&s2);

    KeyPair {
        public: PublicKey {
            a: a.clone(),
            t: t.clone(),
        },
        secret: PrivateKey {
            pub_key: PublicKey { a, t },
            s1,
            s2,
        },
    }
}

/// Hashes high-bits of 'w' and a message to derive a challenge polynomial.
pub fn hash_to_challenge(w1: &[i64; N], msg: &[u8]) -> Poly {
    let mut hasher = Sha256::new();
    let mut w1_bytes = [0u8; N];
    for i in 0..N {
        w1_bytes[i] = w1[i] as u8;
    }
    hasher.update(&w1_bytes);
    hasher.update(msg);
    let h = hasher.finalize();

    let mut prng = Prng::new(h);
    let mut c = Poly::zero();
    let mut count = 0;
    while count < TAU {
        let pos = prng.next_u8() as usize % N;
        if c.coeffs[pos] == 0 {
            let sign_bit = prng.next_u8() & 1;
            c.coeffs[pos] = if sign_bit == 1 { 1 } else { -1 };
            count += 1;
        }
    }
    c
}

/// Signs a message using the private key.
pub fn sign(sk: &PrivateKey, msg: &[u8], seed: [u8; 32]) -> Signature {
    let mut prng = Prng::new(seed);
    loop {
        let y = prng.next_poly_bounded(GAMMA);
        let w = sk.pub_key.a.mul(&y);
        let w1 = w.high_bits();

        let c = hash_to_challenge(&w1, msg);
        let cs1 = c.mul(&sk.s1);
        let z = y.add(&cs1);

        if z.norm_infty() > GAMMA - TAU as i64 {
            continue;
        }

        let az = sk.pub_key.a.mul(&z);
        let tc = sk.pub_key.t.mul(&c);
        let w_prime = az.sub(&tc);

        if w_prime.high_bits() != w1 {
            continue;
        }

        return Signature { z, c };
    }
}

/// Verifies a signature using the public key.
pub fn verify(pk: &PublicKey, msg: &[u8], sig: &Signature) -> bool {
    if sig.z.norm_infty() > GAMMA - TAU as i64 {
        return false;
    }
    if !sig.c.is_valid_challenge() {
        return false;
    }

    let az = pk.a.mul(&sig.z);
    let tc = pk.t.mul(&sig.c);
    let w_prime = az.sub(&tc);

    let w1 = w_prime.high_bits();
    let expected_c = hash_to_challenge(&w1, msg);

    sig.c == expected_c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lattice_signature() {
        let seed = [1u8; 32];
        let kp = generate_keypair(seed);

        let msg = b"Hello, post-quantum world!";
        let sig_seed = [2u8; 32];

        let sig = sign(&kp.secret, msg, sig_seed);
        let ok = verify(&kp.public, msg, &sig);
        assert!(ok, "Signature verification failed");

        // Test malleability/failure
        let mut bad_sig = sig.clone();
        bad_sig.z.coeffs[0] = reduce(bad_sig.z.coeffs[0] + 1);
        assert!(
            !verify(&kp.public, msg, &bad_sig),
            "Tampered signature should fail"
        );
    }
}
