use std::collections::HashMap;
use std::str::{CharIndices, FromStr};

#[derive(Clone, Debug, PartialEq)]
pub enum JsonTokenType {
    LeftBrace,
    RightBrace,
    Comma,
    String,
    Column,
    LeftBracket,
    RightBracket,
    Number,
    True,
    False,
    Null,
}

#[derive(Clone, Debug, PartialEq)]
pub struct JsonToken<'a> {
    pub slice: &'a str,
    pub token_type: JsonTokenType,
}

#[derive(Debug)]
pub enum JsonNumber {
    Integer(i64),
    Float(f64),
}

impl JsonNumber {
    pub fn parse(slice: &str) -> Self {
        if let Ok(n) = i64::from_str(slice) {
            Self::Integer(n)
        } else {
            Self::Float(f64::from_str(slice).unwrap())
        }
    }
}

#[derive(Debug)]
pub enum JsonValue<'a> {
    String(&'a str),
    Number(JsonNumber),
    Boolean(bool),
    Null,
    Array(Vec<JsonValue<'a>>),
    Object(HashMap<&'a str, JsonValue<'a>>),
}

fn forward(iter: &mut impl Iterator, skip: usize) {
    for _ in 0..skip {
        let _ = iter.next();
    }
}

pub fn lex(source: &str) -> Vec<JsonToken> {
    let mut tokens = Vec::new();
    let mut indices = source.char_indices();

    while let Some((idx, chr)) = indices.next() {
        // Skip whitespaces
        if chr.is_whitespace() {
            continue;
        }
        let token_type_single_char = match chr {
            '{' => Some(JsonTokenType::LeftBrace),
            '}' => Some(JsonTokenType::RightBrace),
            ',' => Some(JsonTokenType::Comma),
            ':' => Some(JsonTokenType::Column),
            '[' => Some(JsonTokenType::LeftBracket),
            ']' => Some(JsonTokenType::RightBracket),
            _ => None,
        };

        if let Some(token_type) = token_type_single_char {
            let next_idx = idx + 1;
            tokens.push(JsonToken {
                slice: &source[idx..next_idx],
                token_type,
            });
        } else {
            match chr {
                // Try to find a string
                '"' => {
                    let next_idx = loop {
                        match indices.next() {
                            // Some escaped char
                            Some((_, '\\')) => {
                                if let Some((_, escaped)) = indices.next() {
                                    match escaped {
                                        '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' => {}
                                        'u' => {
                                            // 4 hex digits
                                            forward(&mut indices, 4);
                                        }
                                        _ => panic!(
                                            "Unexpected escaped char '{}' while lexing a string",
                                            escaped
                                        ),
                                    }
                                } else {
                                    panic!("Unexpected end of file while lexing a string with escape chars")
                                }
                                continue;
                            }
                            // End of string
                            Some((idx, '"')) => break idx + 1,
                            // End of file
                            None => panic!("Unexpected end of file while lexing a string"),
                            _ => {}
                        }
                    };
                    tokens.push(JsonToken {
                        slice: &source[idx..next_idx],
                        token_type: JsonTokenType::String,
                    });
                }
                // Try to find a number
                '-' | '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                    let (next_idx, followed_by_comma) = lex_number(idx, chr, &mut indices);
                    tokens.push(JsonToken {
                        slice: &source[idx..next_idx],
                        token_type: JsonTokenType::Number,
                    });
                    if followed_by_comma {
                        tokens.push(JsonToken {
                            slice: &source[next_idx..(next_idx + 1)],
                            token_type: JsonTokenType::Comma,
                        });
                    }
                }
                // Try to find `true`
                't' => {
                    let next_idx = idx + 4;
                    match source.get(idx..next_idx) {
                        Some("true") => {
                            tokens.push(JsonToken {
                                slice: &source[idx..next_idx],
                                token_type: JsonTokenType::True,
                            });
                            forward(&mut indices, 3);
                        }
                        _ => panic!("Failed to lex boolean `true`..."),
                    }
                }
                // Try to find `false`
                'f' => {
                    let next_idx = idx + 5;
                    match source.get(idx..next_idx) {
                        Some("false") => {
                            tokens.push(JsonToken {
                                slice: &source[idx..next_idx],
                                token_type: JsonTokenType::False,
                            });
                            forward(&mut indices, 4);
                        }
                        _ => panic!("Failed to lex boolean `false`..."),
                    }
                }
                // Try to find `null`
                'n' => {
                    let next_idx = idx + 4;
                    match source.get(idx..next_idx) {
                        Some("null") => {
                            tokens.push(JsonToken {
                                slice: &source[idx..next_idx],
                                token_type: JsonTokenType::Null,
                            });
                            forward(&mut indices, 3);
                        }
                        _ => panic!("Failed to lex `null`..."),
                    }
                }
                invalid => panic!("Invalid char encountered: '{}'", invalid),
            }
        }
    }

    tokens
}

#[derive(PartialEq)]
enum NumberLexerState {
    Sign,
    FirstDigits,
    FirstZero,
    FractionDot,
    FractionDigits,
    Exponent,
    ExponentSign,
    ExponentDigits,
}
fn lex_number(start: usize, chr: char, indices: &mut CharIndices) -> (usize, bool) {
    use NumberLexerState::*;
    let mut state = match chr {
        '-' => Sign,
        '0' => FirstZero,
        _ => FirstDigits,
    };
    loop {
        let (idx, chr) = match indices.next() {
            Some(tuple) => tuple,
            None if state == Sign => panic!("Unexpected end of file while lexing a number"),
            None => break (start, false),
        };
        match state {
            Sign => match chr {
                '0' => state = FirstZero,
                '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => state = FirstDigits,
                other => panic!("Unexpected char '{}' while lexing a number", other),
            },
            FirstDigits => match chr {
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {}
                '.' => state = FractionDot,
                'e' | 'E' => state = Exponent,
                other => break (idx, other == ','),
            },
            FirstZero => match chr {
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '-' => {
                    panic!("Invalid start of number '0{}'", chr)
                }
                '.' => state = FractionDot,
                'e' | 'E' => state = Exponent,
                other => break (idx, other == ','),
            },
            FractionDot => match chr {
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                    state = FractionDigits;
                }
                other => panic!(
                    "Unexpected char '{}' after '.' while lexing a number",
                    other
                ),
            },
            FractionDigits => match chr {
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {}
                'e' | 'E' => state = Exponent,
                other => break (idx, other == ','),
            },
            Exponent => match chr {
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => state = ExponentDigits,
                '-' | '+' => state = ExponentSign,
                other => panic!(
                    "Unexpected char '{}' after '[eE]' while lexing a number",
                    other
                ),
            },
            ExponentSign => match chr {
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => state = ExponentDigits,
                other => panic!(
                    "Unexpected char '{}' after exponent sign while lexing a number",
                    other
                ),
            },
            ExponentDigits => match chr {
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {}
                other => break (idx, other == ','),
            },
        }
    }
}

pub fn parse(json: &str) -> JsonValue {
    parse_value(&lex(json))
}

fn parse_value<'a>(tokens: &[JsonToken<'a>]) -> JsonValue<'a> {
    match tokens.first() {
        Some(tok) => match (&tok.token_type, tokens.len()) {
            (JsonTokenType::LeftBracket, len) => {
                if len < 2 {
                    panic!("Incomplete array");
                }
                let last_idx = len - 1;
                if tokens[last_idx].token_type != JsonTokenType::RightBracket {
                    panic!(
                        "Invalid token at the end of document: '{}'",
                        tokens[last_idx].slice
                    );
                }
                parse_array(&tokens[1..last_idx])
            }
            (JsonTokenType::LeftBrace, len) => {
                if len < 2 {
                    panic!("Incomplete object");
                }
                let last_idx = len - 1;
                if tokens[last_idx].token_type != JsonTokenType::RightBrace {
                    panic!(
                        "Invalid token at the end of document: '{}'",
                        tokens[last_idx].slice
                    );
                }
                parse_object(&tokens[1..last_idx])
            }
            (JsonTokenType::String, 1) => JsonValue::String(&tok.slice[1..(tok.slice.len() - 1)]),
            (JsonTokenType::Number, 1) => JsonValue::Number(JsonNumber::parse(tok.slice)),
            (JsonTokenType::True, 1) => JsonValue::Boolean(true),
            (JsonTokenType::False, 1) => JsonValue::Boolean(false),
            (JsonTokenType::Null, 1) => JsonValue::Null,
            _ => panic!("Invalid JSON token stream: ({:?}, {})", tok, tokens.len()),
        },
        None => panic!("Empty JSON is invalid JSON"),
    }
}

fn parse_array<'a>(tokens: &[JsonToken<'a>]) -> JsonValue<'a> {
    let len = tokens.len();
    let mut array = Vec::new();
    let mut idx = 0;
    let mut start = idx;
    let mut n_bracket = 0;
    let mut n_brace = 0;
    loop {
        if idx == len {
            if idx > start {
                array.push(parse_value(&tokens[start..idx]));
            }
            break;
        }
        match &tokens[idx].token_type {
            JsonTokenType::LeftBracket => n_bracket += 1,
            JsonTokenType::LeftBrace => n_brace += 1,
            JsonTokenType::RightBracket => n_bracket -= 1,
            JsonTokenType::RightBrace => n_brace -= 1,
            JsonTokenType::Comma if n_bracket == 0 && n_brace == 0 => {
                array.push(parse_value(&tokens[start..idx]));
                start = idx;
            }
            _ => {}
        }
        idx += 1;
    }
    JsonValue::Array(array)
}

#[derive(PartialEq)]
enum ObjectParserState<'a> {
    BeforeKey,
    Key(JsonToken<'a>),
    Column(JsonToken<'a>, usize),
}
fn parse_object<'a>(tokens: &[JsonToken<'a>]) -> JsonValue<'a> {
    use ObjectParserState::*;

    let len = tokens.len();
    let mut obj = HashMap::new();
    let mut idx = 0;
    let mut n_bracket = 0;
    let mut n_brace = 0;
    let mut state = BeforeKey;
    loop {
        if idx == len {
            match state {
                BeforeKey => {}
                Column(ref key, start) if idx > start => {
                    let k = &key.slice[1..(key.slice.len() - 1)];
                    obj.insert(k, parse_value(&tokens[start..idx]));
                }
                _ => panic!("Incomplete object"),
            }
            break;
        }
        let tok = &tokens[idx];
        match state {
            BeforeKey => {
                if tok.token_type != JsonTokenType::String {
                    panic!(
                        "Unexpected token '{}' in place of string key in object",
                        tok.slice
                    );
                }
                state = Key(tok.clone());
            }
            Key(ref key) => {
                if tok.token_type != JsonTokenType::Column {
                    panic!("Expected ':' token, found '{}'", tok.slice);
                }
                state = Column(key.clone(), idx + 1);
            }
            Column(ref key, start) => match &tok.token_type {
                JsonTokenType::LeftBracket => n_bracket += 1,
                JsonTokenType::LeftBrace => n_brace += 1,
                JsonTokenType::RightBracket => n_bracket -= 1,
                JsonTokenType::RightBrace => n_brace -= 1,
                JsonTokenType::Comma if n_bracket == 0 && n_brace == 0 => {
                    let k = &key.slice[1..(key.slice.len() - 1)];
                    obj.insert(k, parse_value(&tokens[start..idx]));
                    state = BeforeKey;
                }
                _ => {}
            },
        }

        idx += 1;
    }
    JsonValue::Object(obj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_empty_str() {
        let tokens = lex("  ");
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn lex_random_seq_of_single_char_tokens() {
        let tokens = lex("\n{   ]\t{, :\t\t ,\r, \n");
        assert_eq!(
            tokens
                .into_iter()
                .map(|t| t.token_type)
                .collect::<Vec<JsonTokenType>>(),
            vec![
                JsonTokenType::LeftBrace,
                JsonTokenType::RightBracket,
                JsonTokenType::LeftBrace,
                JsonTokenType::Comma,
                JsonTokenType::Column,
                JsonTokenType::Comma,
                JsonTokenType::Comma
            ]
        );
    }
}
