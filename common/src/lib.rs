use bitvec::prelude::*;
use blake2b_simd::{Hash, Params};
use byteorder::{BigEndian, WriteBytesExt};
use ed25519_dalek::{PublicKey, SecretKey};
use once_cell::sync::Lazy;
use regex::Regex;

/// Derive private key from seed and index
pub fn derive_private_key(seed: [u8; 32], index: u32) -> Hash {
    let mut wtr = vec![];
    wtr.write_u32::<BigEndian>(index).unwrap();
    Params::new()
        .hash_length(32)
        .to_state()
        .update(&seed)
        .update(&wtr)
        .finalize()
}

/// Derive public key from private key
pub fn derive_public_key(private_key: Hash) -> PublicKey {
    PublicKey::from(&SecretKey::from_bytes(private_key.as_bytes()).unwrap())
}

/// Derive address from public key
pub fn derive_address(public_key: PublicKey) -> String {
    // Code based on Feeless project implementation
    let mut address = String::with_capacity(65);
    address.push_str("nano_");

    const PKP_LEN: usize = 4 + 8 * 32;
    const PKP_CAPACITY: usize = 4 + 8 * 32 + 4;
    let mut bits: BitVec<Msb0, u8> = BitVec::with_capacity(PKP_CAPACITY);
    let pad: BitVec<Msb0, u8> = bitvec![Msb0, u8; 0; 4];
    bits.extend_from_bitslice(&pad);
    bits.extend_from_raw_slice(public_key.as_bytes());
    debug_assert_eq!(bits.capacity(), PKP_CAPACITY);
    debug_assert_eq!(bits.len(), PKP_LEN);
    let public_key_part = encode_nano_base_32(&bits);
    address.push_str(&public_key_part);

    let result = Params::new()
        .hash_length(5)
        .to_state()
        .update(public_key.as_bytes())
        .finalize();
    let bits: BitVec<Msb0, u8> = BitVec::from_iter(result.as_bytes().iter().rev());
    let checksum = encode_nano_base_32(&bits);
    address.push_str(&checksum);
    address
}

// Function based on Feeless project implementation
const ALPHABET: &str = "13456789abcdefghijkmnopqrstuwxyz";
static ALPHABET_VEC: Lazy<Vec<char>> = Lazy::new(|| ALPHABET.chars().collect());
const ENCODING_BITS: usize = 5;

pub fn encode_nano_base_32(bits: &BitSlice<Msb0, u8>) -> String {
    debug_assert_eq!(
        bits.len() % ENCODING_BITS,
        0,
        "BitSlice must be divisible by 5"
    );
    let mut s = String::new(); // TODO: with_capacity
    for idx in (0..bits.len()).step_by(ENCODING_BITS) {
        let chunk: &BitSlice<Msb0, u8> = &bits[idx..idx + ENCODING_BITS];
        let value: u8 = chunk.load_be();
        let char = ALPHABET_VEC[value as usize];
        s.push(char);
    }
    s
}

/// Convert hex string to bytes array of size 32, where each byte contains 2 hex digits.
pub const HEX: [&str; 16] = [
"0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F",
];

pub fn hexstring_to_bytes(hexstring: &str) -> [u8; 32] {
    let mut buf: [u8; 32] = [0; 32];
    let mut i: usize = 0;
    let mut j_a: u8 = 0;
    let mut j_b: u8 = 0;

    for (a, b) in hexstring.chars().zip(hexstring.chars().skip(1)).step_by(2) {
        for (j, x) in HEX.iter().enumerate() {
            if a.to_string() == x.to_string() {
                j_a = j as u8;
            }
            if b.to_string() == x.to_string() {
                j_b = j as u8;
            }
        }
        buf[i] = j_a << 4 | j_b;
        i += 1;
    }
    buf
}

pub fn validate_seed(seed: &str) -> bool {
    seed.chars().count() == 64 && seed.chars().all(|x| HEX.contains(&x.to_string().as_str()))
}

pub fn validate_address(target: &str) -> bool {
    let re = Regex::new(r"^(nano|xrb)_[13]{1}[13456789abcdefghijkmnopqrstuwxyz]{59}$").unwrap();
    re.is_match(target)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
