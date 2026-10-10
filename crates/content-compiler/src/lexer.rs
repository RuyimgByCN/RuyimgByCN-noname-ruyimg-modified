//! Hand-rolled lexer for the skill DSL.

use crate::CompileError;

/// Token kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /// Identifier or keyword.
    Ident(String),
    /// Integer.
    Int(i64),
    /// (
    LParen,
    /// )
    RParen,
    /// {
    LBrace,
    /// }
    RBrace,
    /// ,
    Comma,
    /// ;
    Semi,
    /// +
    Plus,
    /// -
    Minus,
    /// *
    Star,
    /// ==
    EqEq,
    /// <
    Lt,
    /// >
    Gt,
    /// &&
    AmpAmp,
    /// =
    Assign,
    /// End of input.
    Eof,
}

/// Token with position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// Kind.
    pub kind: TokenKind,
    /// Byte offset.
    pub pos: usize,
}

/// Lex full source.
pub fn lex(source: &str) -> Result<Vec<Token>, CompileError> {
    let bytes = source.as_bytes();
    let mut i = 0usize;
    let mut out = Vec::new();
    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        let pos = i;
        match c {
            b'(' => {
                out.push(Token {
                    kind: TokenKind::LParen,
                    pos,
                });
                i += 1;
            }
            b')' => {
                out.push(Token {
                    kind: TokenKind::RParen,
                    pos,
                });
                i += 1;
            }
            b'{' => {
                out.push(Token {
                    kind: TokenKind::LBrace,
                    pos,
                });
                i += 1;
            }
            b'}' => {
                out.push(Token {
                    kind: TokenKind::RBrace,
                    pos,
                });
                i += 1;
            }
            b',' => {
                out.push(Token {
                    kind: TokenKind::Comma,
                    pos,
                });
                i += 1;
            }
            b';' => {
                out.push(Token {
                    kind: TokenKind::Semi,
                    pos,
                });
                i += 1;
            }
            b'+' => {
                out.push(Token {
                    kind: TokenKind::Plus,
                    pos,
                });
                i += 1;
            }
            b'*' => {
                out.push(Token {
                    kind: TokenKind::Star,
                    pos,
                });
                i += 1;
            }
            b'<' => {
                out.push(Token {
                    kind: TokenKind::Lt,
                    pos,
                });
                i += 1;
            }
            b'>' => {
                out.push(Token {
                    kind: TokenKind::Gt,
                    pos,
                });
                i += 1;
            }
            b'&' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'&' {
                    out.push(Token {
                        kind: TokenKind::AmpAmp,
                        pos,
                    });
                    i += 2;
                } else {
                    return Err(CompileError::Lex {
                        pos,
                        message: "unexpected byte 38".into(),
                    });
                }
            }
            b'=' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    out.push(Token {
                        kind: TokenKind::EqEq,
                        pos,
                    });
                    i += 2;
                } else {
                    out.push(Token {
                        kind: TokenKind::Assign,
                        pos,
                    });
                    i += 1;
                }
            }
            b'-' => {
                if i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit() {
                    i += 1;
                    let (n, ni) = lex_int(bytes, i)?;
                    out.push(Token {
                        kind: TokenKind::Int(-n),
                        pos,
                    });
                    i = ni;
                } else {
                    out.push(Token {
                        kind: TokenKind::Minus,
                        pos,
                    });
                    i += 1;
                }
            }
            b'0'..=b'9' => {
                let (n, ni) = lex_int(bytes, i)?;
                out.push(Token {
                    kind: TokenKind::Int(n),
                    pos,
                });
                i = ni;
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let start = i;
                i += 1;
                while i < bytes.len() {
                    let b = bytes[i];
                    if b.is_ascii_alphanumeric() || b == b'_' {
                        i += 1;
                    } else if b >= 0x80 {
                        // Multi-byte UTF-8 continuation: extend while the decoded `char` is
                        // alphanumeric. This lets localized identifiers (e.g. CJK clan names
                        // embedded in trigger-filter keywords like
                        // `owner_is_event_player_no_太原王氏_lost_hand_this_turn`) lex as one ident.
                        let rest = &source[i..];
                        let mut chars = rest.char_indices();
                        match chars.next() {
                            Some((_, ch)) if ch.is_alphanumeric() => {
                                i += ch.len_utf8();
                            }
                            _ => break,
                        }
                    } else {
                        break;
                    }
                }
                let s = source[start..i].to_string();
                out.push(Token {
                    kind: TokenKind::Ident(s),
                    pos,
                });
            }
            _ => {
                return Err(CompileError::Lex {
                    pos,
                    message: format!("unexpected byte {c:?}"),
                });
            }
        }
    }
    out.push(Token {
        kind: TokenKind::Eof,
        pos: bytes.len(),
    });
    Ok(out)
}

fn lex_int(bytes: &[u8], mut i: usize) -> Result<(i64, usize), CompileError> {
    let start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let s = std::str::from_utf8(&bytes[start..i]).unwrap();
    let n: i64 = s.parse().map_err(|_| CompileError::Lex {
        pos: start,
        message: "invalid integer".into(),
    })?;
    Ok((n, i))
}
