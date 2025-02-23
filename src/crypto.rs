pub fn encode_message(key: u64, message: &str) -> Vec<u8> {
    let key = key.to_le_bytes();

    let message_bytes = message.as_bytes();
    let mut encoded = Vec::with_capacity(message_bytes.len());

    for (i, &byte) in message_bytes.iter().enumerate() {
        let key_byte = key[i % key.len()];
        encoded.push(byte ^ key_byte);
    }

    encoded
}

pub fn decode_message(key: u64, encoded: &[u8]) -> String {
    let key = key.to_le_bytes();

    let mut decoded = Vec::with_capacity(encoded.len());

    for (i, &byte) in encoded.iter().enumerate() {
        let key_byte = key[i % key.len()];
        decoded.push(byte ^ key_byte);
    }

    String::from_utf8(decoded).expect("Failed decoding the message!")
}
