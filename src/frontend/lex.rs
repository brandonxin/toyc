use super::char::Decode;
use super::token::Token;

#[derive(Debug)]
pub struct Lexer<D: Decode<R>, R: std::io::Read> {
    input: D,
    last: char,
    row: isize,
    col: isize,
    _r: std::marker::PhantomData<R>,
}

impl<D: Decode<R>, R: std::io::Read> Lexer<D, R> {
    pub fn new(input: D) -> Lexer<D, R> {
        Lexer {
            input,
            last: ' ',
            row: 1,
            col: 0,
            _r: std::marker::PhantomData,
        }
    }

    pub fn gettok(&mut self) -> Token {
        while self.last.is_whitespace() {
            self.last = match self.getchar() {
                Some(ch) => ch,
                None => return Token::EOF,
            }
        }

        if self.last.is_alphabetic() {
            let mut word = String::new();

            while self.last.is_alphanumeric() || self.last == '_' {
                word.push(self.last);
                self.last = self.getchar().unwrap_or(' ');
            }

            match word.as_str() {
                "func" => return Token::Func,
                "extern" => return Token::Extern,
                "if" => return Token::If,
                "else" => return Token::Else,
                "for" => return Token::For,
                "while" => return Token::While,
                "return" => return Token::Return,
                "var" => return Token::Var,
                _ => return Token::Identifier(word),
            }
        }

        if self.last.is_ascii_digit() {
            let mut number = String::new();

            while self.last.is_ascii_digit() {
                number.push(self.last);
                self.last = self.getchar().unwrap_or(' ');
            }

            let number = number.parse::<u64>().unwrap();
            return Token::Integer(number);
        }

        if self.last == '#' {
            while self.last != '\n' && self.last != '\r' {
                self.last = match self.getchar() {
                    Some(ch) => ch,
                    None => return Token::EOF,
                };
            }
            return self.gettok();
        }

        let mut should_get_next = true;
        let token = match self.last {
            '=' => {
                self.last = self.getchar().unwrap_or(' ');
                if self.last == '=' {
                    Token::Eq
                } else {
                    should_get_next = false;
                    Token::Assign
                }
            }
            '>' => {
                self.last = self.getchar().unwrap_or(' ');
                if self.last == '=' {
                    Token::Ge
                } else if self.last == '>' {
                    Token::RShift
                } else {
                    should_get_next = false;
                    Token::Gt
                }
            }
            '<' => {
                self.last = self.getchar().unwrap_or(' ');
                if self.last == '=' {
                    Token::Le
                } else if self.last == '<' {
                    Token::LShift
                } else {
                    should_get_next = false;
                    Token::Lt
                }
            }
            '|' => {
                self.last = self.getchar().unwrap_or(' ');
                if self.last == '|' {
                    Token::LogicalOr
                } else {
                    should_get_next = false;
                    Token::BitwiseOr
                }
            }
            '^' => Token::BitwiseXor,
            '&' => {
                self.last = self.getchar().unwrap_or(' ');
                if self.last == '&' {
                    Token::LogicalAnd
                } else {
                    should_get_next = false;
                    Token::BitwiseAnd
                }
            }
            '+' => Token::Add,
            '-' => Token::Sub,
            '*' => Token::Mul,
            '/' => Token::Div,
            '%' => Token::Mod,
            '~' => Token::BitwiseNot,
            '!' => {
                self.last = self.getchar().unwrap_or(' ');
                if self.last == '=' {
                    Token::Ne
                } else {
                    should_get_next = false;
                    Token::LogicalNot
                }
            }
            '(' => Token::LParen,
            ')' => Token::RParen,
            '[' => Token::LBrack,
            ']' => Token::RBrack,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            ':' => Token::Colon,
            ';' => Token::SemiColon,
            ',' => Token::Comma,
            _ => panic!(""),
        };

        if should_get_next {
            self.last = self.getchar().unwrap_or(' ');
        }

        token
    }

    fn getchar(&mut self) -> Option<char> {
        let ch = self.input.get_char()?;

        if ch == '\n' {
            self.row += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }

        Some(ch)
    }
}

#[cfg(test)]
mod tests {
    use super::super::utf8::Utf8Decoder;
    use super::*;

    #[test]
    fn fib() {
        let src = String::from(
            r"func fib(n: Int64) : Int64 {
    if n {
        if n - 1 {
            return fib(n - 1) + fib(n - 2);
        } else {
            return 1;
        }
    } else {
        return 1;
    }
}

",
        );

        let mut lexer = Lexer::new(Utf8Decoder::new(src.as_bytes()));
        let tokens = [
            Token::Func,
            Token::Identifier(String::from("fib")),
            Token::LParen,
            Token::Identifier(String::from("n")),
            Token::Colon,
            Token::Identifier(String::from("Int64")),
            Token::RParen,
            Token::Colon,
            Token::Identifier(String::from("Int64")),
            Token::LBrace,
            Token::If,
            Token::Identifier(String::from("n")),
            Token::LBrace,
            Token::If,
            Token::Identifier(String::from("n")),
            Token::Sub,
            Token::Integer(1),
            Token::LBrace,
            Token::Return,
            Token::Identifier(String::from("fib")),
            Token::LParen,
            Token::Identifier(String::from("n")),
            Token::Sub,
            Token::Integer(1),
            Token::RParen,
            Token::Add,
            Token::Identifier(String::from("fib")),
            Token::LParen,
            Token::Identifier(String::from("n")),
            Token::Sub,
            Token::Integer(2),
            Token::RParen,
            Token::SemiColon,
            Token::RBrace,
            Token::Else,
            Token::LBrace,
            Token::Return,
            Token::Integer(1),
            Token::SemiColon,
            Token::RBrace,
            Token::RBrace,
            Token::Else,
            Token::LBrace,
            Token::Return,
            Token::Integer(1),
            Token::SemiColon,
            Token::RBrace,
            Token::RBrace,
        ];

        for (i, answer) in tokens.iter().enumerate() {
            let token = lexer.gettok();
            assert_eq!(&{ token }, answer);
        }
    }
}
