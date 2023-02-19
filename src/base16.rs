use hex;

pub fn decode_string(input: &str) -> Vec<u8> {
    return hex::decode(input).expect("decode-hex");
}

pub fn encode_bytes(input: &[u8]) -> String {
    return hex::encode(&input);
}