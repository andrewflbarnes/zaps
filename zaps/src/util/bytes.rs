extern crate hex;

pub fn byte_to_string(raw: &[u8]) -> String {
    raw.iter().map(|b| *b as char).collect()
}

pub fn byte_to_hex_string(raw: &[u8]) -> String {
    format!("0x{}", hex::encode(raw))
}