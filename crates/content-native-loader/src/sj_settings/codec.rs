//! SJ 设置的文本编解码：base64、URI 与 JS 数字解析（与上游浏览器行为对齐）。

use super::*;

pub(super) fn b64_val(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

pub(super) fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut index = 0;
    while index + 3 <= data.len() {
        let packed =
            ((data[index] as u32) << 16) | ((data[index + 1] as u32) << 8) | data[index + 2] as u32;
        out.push(TABLE[((packed >> 18) & 63) as usize] as char);
        out.push(TABLE[((packed >> 12) & 63) as usize] as char);
        out.push(TABLE[((packed >> 6) & 63) as usize] as char);
        out.push(TABLE[(packed & 63) as usize] as char);
        index += 3;
    }
    if index < data.len() {
        let rest = data.len() - index;
        let second = if rest == 2 { data[index + 1] as u32 } else { 0 };
        let packed = ((data[index] as u32) << 16) | (second << 8);
        out.push(TABLE[((packed >> 18) & 63) as usize] as char);
        out.push(TABLE[((packed >> 12) & 63) as usize] as char);
        if rest == 2 {
            out.push(TABLE[((packed >> 6) & 63) as usize] as char);
            out.push('=');
        } else {
            out.push('=');
            out.push('=');
        }
    }
    out
}

pub(super) fn base64_decode(text: &str) -> Result<Vec<u8>, SjError> {
    if text.is_empty() || text.len() % 4 != 0 {
        return Err(SjError::BadEncoding("base64 length".into()));
    }
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let chunk = &bytes[index..index + 4];
        let pad = chunk.iter().rev().take_while(|byte| **byte == b'=').count();
        if pad > 2 || (pad > 0 && index + 4 != bytes.len()) {
            return Err(SjError::BadEncoding("base64 pad".into()));
        }
        let mut vals = [0u8; 4];
        for (slot, byte) in chunk.iter().enumerate() {
            if *byte == b'=' {
                if slot < 4 - pad {
                    return Err(SjError::BadEncoding("base64 pad".into()));
                }
            } else if slot >= 4 - pad {
                return Err(SjError::BadEncoding("base64 pad".into()));
            } else {
                vals[slot] =
                    b64_val(*byte).ok_or_else(|| SjError::BadEncoding("base64 alphabet".into()))?;
            }
        }
        let packed = ((vals[0] as u32) << 18)
            | ((vals[1] as u32) << 12)
            | ((vals[2] as u32) << 6)
            | vals[3] as u32;
        out.push((packed >> 16) as u8);
        if pad < 2 {
            out.push((packed >> 8) as u8);
        }
        if pad < 1 {
            out.push(packed as u8);
        }
        index += 4;
    }
    Ok(out)
}

/// `lib.init.encode`。BMP 字符与上游的 UTF-8 展开一致。非 BMP 拒绝。
pub fn encode_text(text: &str) -> Result<String, SjError> {
    if text.chars().any(|ch| (ch as u32) > 0xFFFF) {
        return Err(SjError::BadEncoding("non-bmp".into()));
    }
    Ok(base64_encode(text.as_bytes()))
}

/// `lib.init.decode`。非法 Base64 或非法 UTF-8 是导入错误。
pub fn decode_text(text: &str) -> Result<String, SjError> {
    let bytes = base64_decode(text)?;
    String::from_utf8(bytes).map_err(|_| SjError::BadEncoding("utf8".into()))
}

pub(super) fn hex_val(ch: char) -> Option<u8> {
    match ch {
        '0'..='9' => Some((ch as u8) - b'0'),
        'a'..='f' => Some((ch as u8) - b'a' + 10),
        'A'..='F' => Some((ch as u8) - b'A' + 10),
        _ => None,
    }
}

pub(super) fn is_reserved(ch: char) -> bool {
    matches!(
        ch,
        ';' | '/' | '?' | ':' | '@' | '&' | '=' | '+' | '$' | ',' | '#'
    )
}

pub(super) fn utf8_width(lead: u8) -> Result<usize, SjError> {
    let mut width = 0;
    let mut bits = lead;
    while bits & 0x80 != 0 {
        width += 1;
        bits = bits.wrapping_shl(1);
        if width > 4 {
            break;
        }
    }
    if width < 2 || width > 4 {
        Err(SjError::BadEncoding("uri".into()))
    } else {
        Ok(width)
    }
}

/// `decodeURI`。保留字符的百分号序列原样留下。非法序列返回错误。
pub fn decode_uri(text: &str) -> Result<String, SjError> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] != '%' {
            out.push(chars[index]);
            index += 1;
            continue;
        }
        if index + 2 >= chars.len() {
            return Err(SjError::BadEncoding("uri".into()));
        }
        let lead = (hex_val(chars[index + 1]).ok_or_else(|| SjError::BadEncoding("uri".into()))?
            << 4)
            | hex_val(chars[index + 2]).ok_or_else(|| SjError::BadEncoding("uri".into()))?;
        if lead & 0x80 == 0 {
            let ch = char::from(lead);
            if is_reserved(ch) {
                out.push('%');
                out.push(chars[index + 1]);
                out.push(chars[index + 2]);
            } else {
                out.push(ch);
            }
            index += 3;
            continue;
        }
        let width = utf8_width(lead)?;
        let mut octets = vec![lead];
        let mut cursor = index + 3;
        for _ in 1..width {
            if cursor >= chars.len() || chars[cursor] != '%' || cursor + 2 >= chars.len() {
                return Err(SjError::BadEncoding("uri".into()));
            }
            let next = (hex_val(chars[cursor + 1])
                .ok_or_else(|| SjError::BadEncoding("uri".into()))?
                << 4)
                | hex_val(chars[cursor + 2]).ok_or_else(|| SjError::BadEncoding("uri".into()))?;
            if next & 0xC0 != 0x80 {
                return Err(SjError::BadEncoding("uri".into()));
            }
            octets.push(next);
            cursor += 3;
        }
        let decoded = String::from_utf8(octets).map_err(|_| SjError::BadEncoding("uri".into()))?;
        out.push_str(&decoded);
        index = cursor;
    }
    Ok(out)
}

pub(super) fn js_number(raw: &str) -> Option<f64> {
    let text = raw.trim();
    if text.is_empty() {
        return Some(0.0);
    }
    if text == "Infinity" || text == "+Infinity" {
        return Some(f64::INFINITY);
    }
    if text == "-Infinity" {
        return Some(f64::NEG_INFINITY);
    }
    let (sign, hex) = if let Some(rest) = text
        .strip_prefix("+0x")
        .or_else(|| text.strip_prefix("+0X"))
    {
        (1.0, Some(rest))
    } else if let Some(rest) = text
        .strip_prefix("-0x")
        .or_else(|| text.strip_prefix("-0X"))
    {
        (-1.0, Some(rest))
    } else if let Some(rest) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        (1.0, Some(rest))
    } else {
        (1.0, None)
    };
    if let Some(rest) = hex {
        if rest.is_empty() || !rest.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return None;
        }
        return u64::from_str_radix(rest, 16)
            .ok()
            .map(|value| sign * value as f64);
    }
    let mut body = text.to_string();
    if let Some(stripped) = body.strip_prefix('+') {
        body = stripped.to_string();
    }
    if body.ends_with('.') {
        body.push('0');
    }
    if body.starts_with('.') {
        body.insert(0, '0');
    }
    if body.contains("+.0") || body.contains("-.0") {
        return None;
    }
    let numeric = body
        .chars()
        .all(|ch| ch.is_ascii_digit() || matches!(ch, '.' | '-' | '+' | 'e' | 'E'))
        && body.matches('.').count() <= 1
        && body.matches('-').count() + body.matches('+').count() <= 2;
    if !numeric {
        return None;
    }
    body.parse::<f64>()
        .ok()
        .filter(|value| value.is_finite() || value.is_infinite())
}
