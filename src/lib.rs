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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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
                    let (next_idx, next_char) = lex_number(idx, chr, &mut indices);
                    tokens.push(JsonToken {
                        slice: &source[idx..next_idx],
                        token_type: JsonTokenType::Number,
                    });
                    match next_char {
                        Some(',') => tokens.push(JsonToken {
                            slice: &source[next_idx..(next_idx + 1)],
                            token_type: JsonTokenType::Comma,
                        }),
                        Some('}') => tokens.push(JsonToken {
                            slice: &source[next_idx..(next_idx + 1)],
                            token_type: JsonTokenType::RightBrace,
                        }),
                        Some(']') => tokens.push(JsonToken {
                            slice: &source[next_idx..(next_idx + 1)],
                            token_type: JsonTokenType::RightBracket,
                        }),
                        Some(other) => {
                            if !other.is_whitespace() {
                                panic!("Number followed by '{}'", other);
                            }
                        },
                        None => {}
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

#[derive(Debug, PartialEq)]
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
fn lex_number(start: usize, chr: char, indices: &mut CharIndices) -> (usize, Option<char>) {
    use NumberLexerState::*;
    let mut state = match chr {
        '-' => Sign,
        '0' => FirstZero,
        _ => FirstDigits,
    };
    let mut current = start;
    loop {
        let (idx, chr) = match indices.next() {
            Some(tuple) => tuple,
            None if state == Sign => panic!("Unexpected end of file while lexing a number"),
            None => break (current + 1, None),
        };
        current = idx;

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
                other => break (current, Some(other)),
            },
            FirstZero => match chr {
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '-' => {
                    panic!("Invalid start of number '0{}'", chr)
                }
                '.' => state = FractionDot,
                'e' | 'E' => state = Exponent,
                other => break (current, Some(other)),
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
                other => break (current, Some(other)),
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
                other => break (current, Some(other)),
            },
        }
    }
}

pub struct ParseError<'a, 'b> {
    pub token: &'a JsonToken<'b>,
    pub view: &'a [JsonToken<'b>],
    pub msg: String
}
impl<'a, 'b> ParseError<'a, 'b> {
    fn new<M: AsRef<str>, T>(msg: M, token: &'a JsonToken<'b>, view: &'a [JsonToken<'b>]) -> Result<T, Self> {
        Err(Self { msg: msg.as_ref().to_string(), token, view })
    }
}
impl <'a,'b> std::fmt::Debug for ParseError<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let end = 5.min(self.view.len());
        f.debug_struct("ParseError")
            .field("msg", &self.msg)
            .field("token", &self.token)
            .field("view", &&self.view[..end])
            .finish()
    }
}

pub fn parse(json: &str) -> JsonValue {
    let tokens = lex(json);
    match parse_value(&tokens) {
        Ok(v) => v,
        Err(e) => panic!("{:#?}", e)
    }
}

fn parse_value<'a, 'b>(tokens: &'a [JsonToken<'b>]) -> Result<JsonValue<'b>, ParseError<'a, 'b>> {
    Ok(match tokens.first() {
        Some(tok) => match (&tok.token_type, tokens.len()) {
            (JsonTokenType::LeftBracket, len) => {
                if len < 2 {
                    ParseError::new("Incomplete array", tok, tokens)?;
                }
                let last_idx = len - 1;
                if tokens[last_idx].token_type != JsonTokenType::RightBracket {
                    ParseError::new("Invalid token at the end of document", &tokens[last_idx], &tokens[(last_idx - 3)..])?;
                }
                parse_array(&tokens[1..last_idx])?
            }
            (JsonTokenType::LeftBrace, len) => {
                if len < 2 {
                    ParseError::new("Incomplete object", tok, tokens)?;
                }
                let last_idx = len - 1;
                if tokens[last_idx].token_type != JsonTokenType::RightBrace {
                    ParseError::new("Invalid token at the end of document", &tokens[last_idx], &tokens[(last_idx - 3)..])?;
                }
                parse_object(&tokens[1..last_idx])?
            }
            (JsonTokenType::String, 1) => JsonValue::String(&tok.slice[1..(tok.slice.len() - 1)]),
            (JsonTokenType::Number, 1) => JsonValue::Number(JsonNumber::parse(tok.slice)),
            (JsonTokenType::True, 1) => JsonValue::Boolean(true),
            (JsonTokenType::False, 1) => JsonValue::Boolean(false),
            (JsonTokenType::Null, 1) => JsonValue::Null,
            _ => ParseError::new("Invalid JSON token stream", tok, tokens)?,
        },
        None => panic!("Empty JSON is invalid JSON"),
    })
}

fn parse_array<'a, 'b>(tokens: &'a [JsonToken<'b>]) -> Result<JsonValue<'b>, ParseError<'a, 'b>> {
    let len = tokens.len();
    let mut array = Vec::new();
    let mut idx = 0;
    let mut start = idx;
    let mut n_bracket = 0;
    let mut n_brace = 0;
    loop {
        if idx == len {
            if idx > start {
                array.push(parse_value(&tokens[start..idx])?);
            }
            break;
        }
        match &tokens[idx].token_type {
            JsonTokenType::LeftBracket => n_bracket += 1,
            JsonTokenType::LeftBrace => n_brace += 1,
            JsonTokenType::RightBracket => n_bracket -= 1,
            JsonTokenType::RightBrace => n_brace -= 1,
            JsonTokenType::Comma if n_bracket == 0 && n_brace == 0 => {
                array.push(parse_value(&tokens[start..idx])?);
                start = idx + 1;
            }
            _ => {}
        }
        idx += 1;
    }
    Ok(JsonValue::Array(array))
}

#[derive(PartialEq)]
enum ObjectParserState<'a> {
    BeforeKey,
    Key(JsonToken<'a>),
    Column(JsonToken<'a>, usize),
}
fn parse_object<'a, 'b>(tokens: &'a [JsonToken<'b>]) -> Result<JsonValue<'b>, ParseError<'a, 'b>> {
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
                    obj.insert(k, parse_value(&tokens[start..idx])?);
                }
                Column(_, start) => {
                    ParseError::new(format!("start({}) >= idx({})", start, idx), &tokens[idx - 1], tokens)?
                },
                Key(_) => ParseError::new("Incomplete object", &tokens[idx - 1], tokens)?,
            }
            break;
        }
        let tok = &tokens[idx];
        match state {
            BeforeKey => {
                if tok.token_type != JsonTokenType::String {
                    ParseError::new(
                        "Unexpected token in place of string key in object",
                        tok,
                        &tokens[(idx - 1)..]
                    )?;
                }
                state = Key(tok.clone());
            }
            Key(ref key) => {
                if tok.token_type != JsonTokenType::Column {
                    ParseError::new("Expected ':' token, found '{}'", tok, tokens)?;
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
                    obj.insert(k, parse_value(&tokens[start..idx])?);
                    state = BeforeKey;
                }
                _ => {}
            },
        }

        idx += 1;
    }
    Ok(JsonValue::Object(obj))
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

    #[test]
    fn simple_values()  {
        assert_eq!(JsonValue::Number(JsonNumber::Integer(5)), parse("5"));
        assert_eq!(JsonValue::Number(JsonNumber::Float(6.626E-34)), parse("6.626e-34"));
        assert_eq!(JsonValue::Boolean(true), parse("true"));
        assert_eq!(JsonValue::Null, parse("null"));
        assert_eq!(JsonValue::String("Hello"), parse("\"Hello\""));
    }
}
