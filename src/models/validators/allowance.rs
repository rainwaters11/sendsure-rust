pub(crate) fn is_uint256_max(value: &str) -> bool {
    let trimmed = value.trim();
    let bytes = if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        parse_hex_to_le_bytes(hex)
    } else {
        parse_decimal_to_le_bytes(trimmed)
    };
    bytes.map(|value| value == vec![0xff; 32]).unwrap_or(false)
}

fn parse_decimal_to_le_bytes(value: &str) -> Option<Vec<u8>> {
    let mut bytes = vec![0_u8];
    for ch in value.chars() {
        let digit = ch.to_digit(10)? as u16;
        let mut carry = digit;
        for byte in &mut bytes {
            let sum = (*byte as u16) * 10 + carry;
            *byte = (sum & 0xff) as u8;
            carry = sum >> 8;
        }
        while carry > 0 {
            bytes.push((carry & 0xff) as u8);
            carry >>= 8;
        }
    }
    Some(normalize_le_bytes(bytes))
}

fn parse_hex_to_le_bytes(value: &str) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut current = 0_u8;
    let mut nibble_index = 0_u8;
    for ch in value.chars() {
        let nibble = ch.to_digit(16)? as u8;
        if nibble_index.is_multiple_of(2) {
            current = nibble << 4;
        } else {
            current |= nibble;
            bytes.push(current);
        }
        nibble_index += 1;
    }
    if nibble_index % 2 == 1 {
        bytes.push(current);
    }
    Some(normalize_le_bytes(bytes))
}

fn normalize_le_bytes(mut bytes: Vec<u8>) -> Vec<u8> {
    while bytes.len() > 1 && bytes.last() == Some(&0) {
        bytes.pop();
    }
    bytes
}
