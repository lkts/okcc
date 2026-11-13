use std::{collections::HashMap, iter::Peekable, sync::LazyLock};

use snafu::Snafu;

use crate::compiler::{Keyword, Token};

#[derive(Debug, Snafu, PartialEq, Eq)]
pub struct LexerError {
    errors: Vec<InvalidToken>,
}

// TODO nicer errors, refer to input maybe?
#[derive(Debug, Snafu, PartialEq, Eq)]
struct InvalidToken {
    token: char,
}

pub fn lex(input: &str) -> Result<Vec<Token>, LexerError> {
    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    let mut chars = input.chars().peekable();
    while let Some(char) = chars.next() {
        let token_result = match char {
            '(' => Ok(Token::OpenParen),
            ')' => Ok(Token::CloseParen),
            '{' => Ok(Token::OpenBrace),
            '}' => Ok(Token::CloseBrace),
            ';' => Ok(Token::Semicolon),
            c if c.is_ascii_digit() => number(c, &mut chars),
            // Per standard has to start with "nondigit"
            c if c.is_ascii_alphabetic() || c == '_' => Ok(identifier_or_keyword(c, &mut chars)),
            c if c.is_whitespace() == false => Err(InvalidToken { token: c }),
            _ => continue,
        };

        match token_result {
            Ok(token) => {
                tokens.push(token);
            }
            Err(err) => {
                errors.push(err);
            }
        }
    }

    if errors.is_empty() {
        Ok(tokens)
    } else {
        Err(LexerError { errors })
    }
}

fn number(
    current: char,
    chars: &mut Peekable<impl Iterator<Item = char>>,
) -> Result<Token, InvalidToken> {
    // Only integers for now
    let mut source = String::new();
    source.push(current);

    while let Some(c) = chars.next_if(|c| c.is_ascii_digit()) {
        source.push(c);
    }

    // We are done with this number but the next token has to make sense as well.
    // `is_alphabetic` check protects from identifiers that start with a digit.
    if let Some(c) = chars.peek()
        && c.is_alphabetic()
    {
        Err(InvalidToken { token: *c })
    } else {
        Ok(Token::Constant {
            value: source.parse().unwrap(),
        })
    }
}

fn identifier_or_keyword(current: char, chars: &mut Peekable<impl Iterator<Item = char>>) -> Token {
    let mut source = String::new();
    source.push(current);

    // TODO should be less strict, there are specific allowed character ranges in "universal character names"
    while let Some(char) = chars.next_if(|c| c.is_alphanumeric()) {
        source.push(char);
    }

    if let Some(keyword) = KEYWORDS.get(source.as_str()) {
        Token::Keyword(*keyword)
    } else {
        Token::Identifier { value: source }
    }
}

static KEYWORDS: LazyLock<HashMap<&str, Keyword>> = LazyLock::new(|| {
    HashMap::from([
        ("int", Keyword::Int),
        ("return", Keyword::Return),
        ("void", Keyword::Void),
    ])
});

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use super::*;
    use crate::compiler::{Keyword, Token};
    use proptest::{collection::vec, prelude::*};

    #[test]
    fn return_two() {
        let program = r#"
            int main(void) {
                return 2;
            }"#;

        let tokens = vec![
            Token::Keyword(Keyword::Int),
            Token::Identifier {
                value: "main".to_string(),
            },
            Token::OpenParen,
            Token::Keyword(Keyword::Void),
            Token::CloseParen,
            Token::OpenBrace,
            Token::Keyword(Keyword::Return),
            Token::Constant { value: 2 },
            Token::Semicolon,
            Token::CloseBrace,
        ];
        assert_eq!(tokens, lex(program).unwrap());
    }

    #[test]
    fn invalid_token() {
        assert_eq!(
            Err(LexerError {
                errors: vec![InvalidToken { token: '@' }]
            }),
            lex("@")
        );
    }

    #[test]
    fn multiple_invalid_tokens() {
        assert_eq!(
            Err(LexerError {
                errors: vec![InvalidToken { token: '@' }, InvalidToken { token: '^' }]
            }),
            lex("@   \n ^")
        );
    }

    #[test]
    fn invalid_identifier() {
        let program = r#"
        int 1main(void) {
            return 451foo;
        }"#;

        // 'm' is invalid since we expect a constant after seeing '1', same logic for 'f'
        assert_eq!(
            Err(LexerError {
                errors: vec![InvalidToken { token: 'm' }, InvalidToken { token: 'f' }]
            }),
            lex(&program)
        );
    }

    proptest! {
        #[test]
        fn doesnt_crash(s in "\\PC*") {
            let _ = lex(&s);
        }

        #[test]
        fn lex_arbitrary_tokens(input in lexer_input()) {
            assert!(lex(&input).is_ok());
        }
    }

    fn lexer_input() -> impl Strategy<Value = String> {
        vec(any::<Token>(), 0..10)
            .prop_flat_map(|token_vec| {
                let whitespace = vec("\\s+", token_vec.len());

                (Just(token_vec), whitespace)
            })
            .prop_map(|(tokens, whitespaces)| {
                tokens.iter().zip(whitespaces.iter()).fold(
                    String::new(),
                    |mut acc, (token, whitespace)| {
                        write_token(&mut acc, token);
                        acc.push_str(whitespace);

                        acc
                    },
                )
            })
    }

    fn write_token(dest: &mut String, token: &Token) {
        match token {
            Token::Identifier { value } => {
                write!(dest, "{value}")
            }
            Token::Constant { value } => write!(dest, "{value}"),
            Token::Keyword(keyword) => match keyword {
                Keyword::Int => write!(dest, "int"),
                Keyword::Return => write!(dest, "return"),
                Keyword::Void => write!(dest, "void"),
            },
            Token::OpenParen => write!(dest, "("),
            Token::CloseParen => write!(dest, ")"),
            Token::OpenBrace => write!(dest, "{{"),
            Token::CloseBrace => write!(dest, "}}"),
            Token::Semicolon => write!(dest, ";"),
        }
        .expect("Append to string should always succeed");
    }
}
