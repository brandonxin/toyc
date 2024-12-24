use std::io::Read;

use crate::ast::{self, Function};

struct Reader<R: Read> {
    input: R,
}

impl<R: Read> Reader<R> {
    pub fn getchar(&mut self) -> Option<char> {
        let Some(raw) = self.get_raw_char() else {
            return None;
        };

        Some(char::try_from(raw).unwrap())
    }

    fn get_raw_char(&mut self) -> Option<u32> {
        let Some(a) = self.get_byte() else {
            return None;
        };

        if a & 0x80 == 0 {
            return Some(a);
        }

        if a & 0xE0 == 0xC0 {
            let b = self.get_continuation_byte();
            let code_point = ((a & 0x1F) << 6) | b;
            if code_point < 0x80 {
                panic!("invalid code point")
            }
            return Some(code_point);
        }
        if a & 0xF0 == 0xE0 {
            let b = self.get_continuation_byte();
            let c = self.get_continuation_byte();
            let code_point = ((a & 0x0F) << 12) | (b << 6) | c;
            if code_point < 0x0800 {
                panic!("invalid code point");
            }
            if (0xD800 <= code_point) && (code_point <= 0xDFFF) {
                panic!("invalid scalar value for lone surrogate");
            }
            return Some(code_point);
        }
        if a & 0xF8 == 0xF0 {
            let b = self.get_continuation_byte();
            let c = self.get_continuation_byte();
            let d = self.get_continuation_byte();
            let code_point = ((a & 0x07) << 18) | (b << 12) | (c << 6) | d;
            if code_point < 0x010000 || code_point > 0x10FFFF {
                panic!("invalid code point");
            }
            return Some(code_point);
        }

        panic!("invalid byte sequence");
    }

    fn get_continuation_byte(&mut self) -> u32 {
        let Some(byte) = self.get_byte() else {
            panic!("invalid byte sequence");
        };

        if (byte & 0xC0) == 0x80 {
            return byte & 0x3F;
        } else {
            panic!("invalid continuation byte");
        }
    }

    fn get_byte(&mut self) -> Option<u32> {
        let mut buf = [0 as u8; 1];
        match self.input.read(&mut buf).unwrap() {
            0 => None,
            _ => Some(buf[0] as u32),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum Token {
    Func,
    Extern,
    If,
    Else,
    For,
    While,
    Return,
    Var,

    Identifier(String),
    Integer(u64),

    Eq,
    Lt,
    Add,
    Sub,
    Mul,
    Div,

    BitwiseNot,
    LogicalNot,

    LParenth,
    RParenth,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    Colon,
    SemiColon,
    Comma,

    EoF,
}

struct Lexer<R: Read> {
    input: Reader<R>,
    last: char,
    row: isize,
    col: isize,
}

impl<R: Read> Lexer<R> {
    pub fn new(input: R) -> Lexer<R> {
        Lexer {
            input: Reader::<R> { input },
            last: ' ',
            row: 1,
            col: 0,
        }
    }

    pub fn gettok(&mut self) -> Token {
        while self.last.is_whitespace() {
            self.last = match self.getchar() {
                Some(ch) => ch,
                None => return Token::EoF,
            }
        }

        if self.last.is_alphabetic() {
            let mut word = String::new();

            while self.last.is_alphanumeric() || self.last == '_' {
                word.push(self.last);
                self.last = match self.getchar() {
                    Some(ch) => ch,
                    None => ' ',
                };
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

        if self.last.is_digit(10) {
            let mut number = String::new();

            while self.last.is_digit(10) {
                number.push(self.last);
                self.last = match self.getchar() {
                    Some(ch) => ch,
                    None => ' ',
                };
            }

            let number = number.parse::<u64>().unwrap();
            return Token::Integer(number);
        }

        if self.last == '#' {
            while self.last != '\n' && self.last != '\r' {
                self.last = match self.getchar() {
                    Some(ch) => ch,
                    None => return Token::EoF,
                };
                return self.gettok();
            }
        }

        let token = match self.last {
            '=' => Token::Eq,
            '<' => Token::Lt,
            '+' => Token::Add,
            '-' => Token::Sub,
            '*' => Token::Mul,
            '/' => Token::Div,
            '~' => Token::BitwiseNot,
            '!' => Token::LogicalNot,
            '(' => Token::LParenth,
            ')' => Token::RParenth,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            ':' => Token::Colon,
            ';' => Token::SemiColon,
            ',' => Token::Comma,
            _ => panic!(""),
        };
        self.last = match self.getchar() {
            Some(ch) => ch,
            None => ' ',
        };
        token
    }

    fn getchar(&mut self) -> Option<char> {
        let Some(ch) = self.input.getchar() else {
            return None;
        };

        if ch == '\n' {
            self.row += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }

        Some(ch)
    }
}

struct Parser<R: Read> {
    lexer: Lexer<R>,
    curr: Token,
}

impl<R: Read> Parser<R> {
    pub fn new(input: R) -> Parser<R> {
        Parser {
            lexer: Lexer::<R>::new(input),
            curr: Token::EoF,
        }
    }

    fn binop_precedence(token: &Token) -> isize {
        match token {
            Token::Eq => 2,
            Token::Lt => 10,
            Token::Add => 20,
            Token::Sub => 20,
            Token::Mul => 40,
            Token::Div => 40,
            _ => -1,
        }
    }

    pub fn parse(&mut self, unit: &mut ast::CompilationUnit) {
        self.get_next_token();
        loop {
            match self.curr {
                Token::EoF => return,
                Token::SemiColon => self.get_next_token(),
                Token::Func => self.parse_function(unit),
                Token::Extern => self.parse_extern(unit),
                _ => panic!("unexpected token"),
            }
        }
    }

    fn parse_function(&mut self, unit: &mut ast::CompilationUnit) {
        // eat 'func'
        self.get_next_token();

        let proto = self.parse_prototype();

        if self.curr != Token::LBrace {
            panic!("expected '{{'");
        }
        let body = self.parse_block_stmt();

        let func = ast::Function::new(proto, body);
        let decl = ast::Declaration::Function(func);

        unit.push(decl);
    }

    fn parse_extern(&mut self, unit: &mut ast::CompilationUnit) {
        // Eat 'extern'
        self.get_next_token();

        let proto = self.parse_prototype();
        let decl = ast::Declaration::Prototype(proto);

        unit.push(decl);
    }

    fn parse_prototype(&mut self) -> ast::Prototype {
        let func_name = if let Token::Identifier(ref s) = self.curr {
            s.clone()
        } else {
            panic!("expected function name");
        };

        self.get_next_token();

        if self.curr != Token::LParenth {
            panic!("expected '('");
        }
        self.get_next_token();

        let mut parameters = Vec::<ast::Parameter>::new();
        while let Token::Identifier(param_name) = self.curr.clone() {
            self.get_next_token();
            if self.curr != Token::Colon {
                panic!("expected ':' after parameter name");
            }

            self.get_next_token();
            let type_name = if let Token::Identifier(ref s) = self.curr {
                s.clone()
            } else {
                panic!("expected type after ':'");
            };
            parameters.push(ast::Parameter::new(param_name, type_name));

            self.get_next_token();
            match self.curr {
                Token::Comma => self.get_next_token(),
                Token::RParenth => break,
                _ => panic!("expected ')' or ','"),
            }
        }

        if self.curr != Token::RParenth {
            panic!("expected ')");
        }

        self.get_next_token();
        if self.curr != Token::Colon {
            return ast::Prototype::new(func_name, String::from("void"), parameters);
        }

        self.get_next_token();
        let return_type = if let Token::Identifier(ref s) = self.curr {
            s.clone()
        } else {
            panic!("expected return type");
        };

        self.get_next_token();
        ast::Prototype::new(func_name, return_type, parameters)
    }

    fn parse_stmt(&mut self) -> Option<ast::Stmt> {
        match self.curr {
            Token::LBrace => Some(self.parse_block_stmt()),
            Token::If => Some(self.parse_if_stmt()),
            Token::While => Some(self.parse_while_stmt()),
            Token::Var => Some(self.parse_var_decl_stmt()),
            Token::Return => Some(self.parse_return_stmt()),
            Token::SemiColon => {
                self.get_next_token();
                None
            }
            _ => Some(self.parse_expr_stmt()),
        }
    }

    // block ::= '{' stmt* '}'
    fn parse_block_stmt(&mut self) -> ast::Stmt {
        self.get_next_token();

        let mut stmts = Vec::<ast::Stmt>::new();
        while self.curr != Token::RBrace {
            if let Some(x) = self.parse_stmt() {
                stmts.push(x);
            }
        }

        self.get_next_token();

        ast::Stmt::Block(stmts)
    }

    // ifexpr
    //   ::= 'if' expr block
    //   ::= 'if' expr block else block
    fn parse_if_stmt(&mut self) -> ast::Stmt {
        self.get_next_token();

        let cond = self.parse_expr();

        if self.curr != Token::LBrace {
            panic!("expected '{{'");
        }
        let then_stmt = self.parse_block_stmt();

        if self.curr != Token::Else {
            return ast::Stmt::IfElse(ast::IfElse::new(cond, then_stmt, None));
        }

        self.get_next_token();
        if self.curr != Token::LBrace {
            panic!("expected '{{'");
        }
        let else_stmt = self.parse_block_stmt();

        ast::Stmt::IfElse(ast::IfElse::new(cond, then_stmt, Some(else_stmt)))
    }

    // whilestmt ::= 'while' expr block
    fn parse_while_stmt(&mut self) -> ast::Stmt {
        self.get_next_token();

        let cond = self.parse_expr();

        if self.curr != Token::LBrace {
            panic!("expected '{{'");
        }
        let body = self.parse_block_stmt();

        ast::Stmt::While(ast::While::new(cond, body))
    }

    // varstmt ::= 'var' identifier ':' identifier ('=' expr)? ';'
    fn parse_var_decl_stmt(&mut self) -> ast::Stmt {
        self.get_next_token();

        let var_name = if let Token::Identifier(ref s) = self.curr {
            s.clone()
        } else {
            panic!("expected variable name");
        };

        self.get_next_token();
        if self.curr != Token::Colon {
            panic!("expected ':'");
        }

        self.get_next_token();
        let type_name = if let Token::Identifier(ref s) = self.curr {
            s.clone()
        } else {
            panic!("expected type name");
        };

        self.get_next_token();
        let expr = if self.curr == Token::Eq {
            self.get_next_token();
            Some(self.parse_expr())
        } else {
            None
        };

        if self.curr != Token::SemiColon {
            panic!("expected ';'");
        }

        self.get_next_token();

        ast::Stmt::VarDecl(ast::VarDecl::new(var_name, type_name, expr))
    }

    // returnstmt ::= 'return' expr? ';'
    fn parse_return_stmt(&mut self) -> ast::Stmt {
        self.get_next_token();

        let expr = if self.curr != Token::SemiColon {
            Some(self.parse_expr())
        } else {
            None
        };

        if self.curr != Token::SemiColon {
            panic!("expected ';'");
        }

        self.get_next_token();

        ast::Stmt::Return(ast::Return::new(expr))
    }

    // exprstmt ::= expr ';'
    fn parse_expr_stmt(&mut self) -> ast::Stmt {
        let expr = self.parse_expr();

        if self.curr != Token::SemiColon {
            panic!("expected ';'");
        }

        self.get_next_token();

        ast::Stmt::ExprStmt(expr)
    }

    // expression
    //   ::= unary '=' expression
    fn parse_expr(&mut self) -> ast::Expr {
        self.parse_assignment()
    }

    // assignment ::= unary ('=' assignment)?
    fn parse_assignment(&mut self) -> ast::Expr {
        let lhs = self.parse_ternary();

        if self.curr == Token::Eq {
            self.get_next_token();
            let rhs = self.parse_assignment();
            let bin = ast::Binary::new(ast::BinaryOp::Assignment, lhs, rhs);
            ast::Expr::Binary(bin)
        } else {
            lhs
        }
    }

    fn parse_ternary(&mut self) -> ast::Expr {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> ast::Expr {
        self.parse_logical_and()
    }

    fn parse_logical_and(&mut self) -> ast::Expr {
        self.parse_bitwise_or()
    }

    fn parse_bitwise_or(&mut self) -> ast::Expr {
        self.parse_bitwise_xor()
    }

    fn parse_bitwise_xor(&mut self) -> ast::Expr {
        self.parse_bitwise_and()
    }

    fn parse_bitwise_and(&mut self) -> ast::Expr {
        self.parse_equality()
    }

    // expr
    //  ::= comparison
    //  ::= comparison ('==' comparison)*
    //  ::= comparison ('!=' comparison)*
    fn parse_equality(&mut self) -> ast::Expr {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> ast::Expr {
        self.parse_addition()
    }

    // expr ::= multiplication ( ('+' / '-') multiplication)*
    fn parse_addition(&mut self) -> ast::Expr {
        let mut lhs = self.parse_multiplication();
        loop {
            let binop = match self.curr {
                Token::Add => ast::BinaryOp::Add,
                Token::Sub => ast::BinaryOp::Sub,
                _ => return lhs,
            };
            self.get_next_token();
            lhs = ast::Expr::Binary(ast::Binary::new(binop, lhs, self.parse_multiplication()));
        }
    }

    // expr ::= multiplication ( ('*' / '/') multiplication)*
    fn parse_multiplication(&mut self) -> ast::Expr {
        let mut lhs = self.parse_unary();
        loop {
            let binop = match self.curr {
                Token::Mul => ast::BinaryOp::Mul,
                Token::Div => ast::BinaryOp::Div,
                _ => return lhs,
            };
            self.get_next_token();
            lhs = ast::Expr::Binary(ast::Binary::new(binop, lhs, self.parse_unary()));
        }
    }

    // unary
    //   ::= primary
    //   ::= '~' unary
    //   ::= '!' unary
    //   ::= '-' unary
    fn parse_unary(&mut self) -> ast::Expr {
        let op = match self.curr {
            Token::BitwiseNot => ast::UnaryOp::BitwiseNot,
            Token::LogicalNot => ast::UnaryOp::BitwiseNot,
            Token::Sub => ast::UnaryOp::Neg,
            _ => return self.parse_primary(),
        };

        self.get_next_token();
        ast::Expr::Unary(ast::Unary::new(op, self.parse_unary()))
    }

    // primary
    //   ::= identifierexpr
    //   ::= numberexpr
    //   ::= parenexpr
    fn parse_primary(&mut self) -> ast::Expr {
        match self.curr {
            Token::Identifier(ref s) => self.parse_identifier_expr(),
            Token::Integer(n) => self.parse_number_expr(),
            Token::LParenth => self.parse_paren_expr(),
            _ => panic!("unexpected token"),
        }
    }

    // identifierexpr
    //   ::= identifier
    //   ::= identifier '(' expression* ')'
    fn parse_identifier_expr(&mut self) -> ast::Expr {
        let name = if let Token::Identifier(ref s) = self.curr {
            s.clone()
        } else {
            panic!("expected identifier");
        };

        self.get_next_token();
        if self.curr != Token::LParenth {
            return ast::Expr::Variable(name);
        }

        // This is a function call
        self.get_next_token();
        let mut args = Vec::<ast::Expr>::new();
        while self.curr != Token::RParenth {
            args.push(self.parse_expr());
            if self.curr != Token::Comma {
                break;
            }
            self.get_next_token();
        }

        if self.curr != Token::RParenth {
            panic!("expected ')'");
        }

        self.get_next_token();
        ast::Expr::Call(ast::Call::new(name, args))
    }

    // parenexpr ::= '(' expression ')'
    fn parse_paren_expr(&mut self) -> ast::Expr {
        self.get_next_token();
        let expr = self.parse_expr();
        if self.curr != Token::RParenth {
            panic!("expected ')'");
        }
        self.get_next_token();
        expr
    }

    // numberexpr ::= number
    fn parse_number_expr(&mut self) -> ast::Expr {
        let number = if let Token::Integer(n) = self.curr {
            n
        } else {
            panic!("expected number");
        };

        self.get_next_token();
        ast::Expr::Integer(number)
    }

    fn get_next_token(&mut self) {
        self.curr = self.lexer.gettok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_0() {
        let input = String::from("");
        let mut reader = Reader {
            input: input.as_bytes(),
        };

        assert_eq!(reader.getchar(), None);
        assert_eq!(reader.getchar(), None);
    }

    #[test]
    fn ascii_1() {
        let input = String::from("a");
        let mut reader = Reader {
            input: input.as_bytes(),
        };

        assert_eq!(reader.getchar(), Some('a'));
        assert_eq!(reader.getchar(), None);
        assert_eq!(reader.getchar(), None);
    }

    #[test]
    fn ascii_n() {
        let input = String::from("abcd");
        let mut reader = Reader {
            input: input.as_bytes(),
        };

        assert_eq!(reader.getchar(), Some('a'));
        assert_eq!(reader.getchar(), Some('b'));
        assert_eq!(reader.getchar(), Some('c'));
        assert_eq!(reader.getchar(), Some('d'));
        assert_eq!(reader.getchar(), None);
        assert_eq!(reader.getchar(), None);
    }

    #[test]
    fn utf8_n() {
        let input = String::from("上山打老虎🐯");
        let mut reader = Reader {
            input: input.as_bytes(),
        };
        assert_eq!(reader.getchar(), Some('上'));
        assert_eq!(reader.getchar(), Some('山'));
        assert_eq!(reader.getchar(), Some('打'));
        assert_eq!(reader.getchar(), Some('老'));
        assert_eq!(reader.getchar(), Some('虎'));
        assert_eq!(reader.getchar(), Some('🐯'));
        assert_eq!(reader.getchar(), None);
        assert_eq!(reader.getchar(), None);
    }

    #[test]
    #[should_panic]
    fn invalid_utf8_1() {
        let input: [u8; 1] = [0x80];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn invalid_utf8_2() {
        let input: [u8; 2] = [0xC0, 0x00];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn invalid_utf8_3() {
        let input: [u8; 3] = [0xC0, 0x80, 0x00];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn invalid_utf8_4() {
        let input: [u8; 4] = [0xC0, 0x80, 0x80, 0x00];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    fn utf8_6() {
        let input: [u8; 6] = [0xF0, 0x9F, 0x90, 0xAF, 0xC0, 0x00];
        let mut reader = Reader { input: &input[..] };
        assert_eq!(reader.getchar(), Some('🐯'));
    }

    #[test]
    #[should_panic]
    fn invalid_utf8_6() {
        let input: [u8; 6] = [0xF0, 0x9F, 0x90, 0xAF, 0xC0, 0x00];
        let mut reader = Reader { input: &input[..] };
        assert_eq!(reader.getchar(), Some('🐯'));
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn byte_never_appear_case_11() {
        let input: [u8; 1] = [0xC0];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn byte_never_appear_case_12() {
        let input: [u8; 1] = [0xC1];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn byte_never_appear_case_13() {
        let input: [u8; 1] = [0xF5];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn byte_never_appear_case_14() {
        let input: [u8; 1] = [0xFF];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn byte_never_appear_case_2() {
        let input: [u8; 1] = [0xBF];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn byte_never_appear_case_41() {
        // An overlong encoding - 0xE0 followed by less than 0xA0
        // In that case, for a 3-byte encoding 1110wwww 10xxxxyy 10yyzzz,
        // there is wwww = 0, xxxx <= 0111, which can be encoded by
        // 2-byte sequence 110xxxyy 10yyzzz
        let input: [u8; 3] = [0xE0, 0x9F, 0x00];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn byte_never_appear_case_42() {
        // An overlong encoding - 0xF0 followed by less than 0x90
        // In that case, for a 4-byte encoding:
        // 11110uvv 10vvwwww 10xxxxyy 10yyzzz,
        // there is u = 0,vvvv = 0, which can be encoded by 3-byte sequence
        //
        let input: [u8; 4] = [0xF0, 0x8F, 0x00, 0x00];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn byte_never_appear_case_5() {
        // A 4-byte sequence that decodes to a value greater that U+10FFFF (0xF4
        // followed by 0x90 or greater)
        let input: [u8; 4] = [0xF4, 0x90, 0x00, 0x00];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    fn lexer_fib() {
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

        let mut lexer = Lexer::new(src.as_bytes());
        let tokens = [
            Token::Func,
            Token::Identifier(String::from("fib")),
            Token::LParenth,
            Token::Identifier(String::from("n")),
            Token::Colon,
            Token::Identifier(String::from("Int64")),
            Token::RParenth,
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
            Token::LParenth,
            Token::Identifier(String::from("n")),
            Token::Sub,
            Token::Integer(1),
            Token::RParenth,
            Token::Add,
            Token::Identifier(String::from("fib")),
            Token::LParenth,
            Token::Identifier(String::from("n")),
            Token::Sub,
            Token::Integer(2),
            Token::RParenth,
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
            println!("{i}: expectedd: {answer:?}, actual: {token:?}");
            assert_eq!(&{ token }, answer);
        }
    }

    #[test]
    fn parser_fib() {
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

        let mut parser = Parser::new(src.as_bytes());
        let mut unit = ast::CompilationUnit::new();
        parser.parse(&mut unit);

        let mut expected = ast::CompilationUnit::new();
        let func = Function::new(
            ast::Prototype::new(
                String::from("fib"),
                String::from("Int64"),
                vec![ast::Parameter::new(
                    String::from("n"),
                    String::from("Int64"),
                )],
            ),
            ast::Stmt::Block(vec![ast::Stmt::IfElse(ast::IfElse::new(
                ast::Expr::Variable(String::from("n")),
                ast::Stmt::Block(vec![ast::Stmt::IfElse(ast::IfElse::new(
                    ast::Expr::Binary(ast::Binary::new(
                        ast::BinaryOp::Sub,
                        ast::Expr::Variable(String::from("n")),
                        ast::Expr::Integer(1),
                    )),
                    ast::Stmt::Block(vec![ast::Stmt::Return(ast::Return::new(Some(
                        ast::Expr::Binary(ast::Binary::new(
                            ast::BinaryOp::Add,
                            ast::Expr::Call(ast::Call::new(
                                String::from("fib"),
                                vec![ast::Expr::Binary(ast::Binary::new(
                                    ast::BinaryOp::Sub,
                                    ast::Expr::Variable(String::from("n")),
                                    ast::Expr::Integer(1),
                                ))],
                            )),
                            ast::Expr::Call(ast::Call::new(
                                String::from("fib"),
                                vec![ast::Expr::Binary(ast::Binary::new(
                                    ast::BinaryOp::Sub,
                                    ast::Expr::Variable(String::from("n")),
                                    ast::Expr::Integer(2),
                                ))],
                            )),
                        )),
                    )))]),
                    Some(ast::Stmt::Block(vec![ast::Stmt::Return(ast::Return::new(
                        Some(ast::Expr::Integer(1)),
                    ))])),
                ))]),
                Some(ast::Stmt::Block(vec![ast::Stmt::Return(ast::Return::new(
                    Some(ast::Expr::Integer(1)),
                ))])),
            ))]),
        );
        let decl = ast::Declaration::Function(func);
        expected.push(decl);

        assert_eq!(unit, expected);
    }
}
