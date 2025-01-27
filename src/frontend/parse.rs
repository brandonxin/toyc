use super::char::Decode;
use super::lex::Lexer;
use super::token::Token;
use crate::ast;

#[derive(Debug)]
pub struct Parser<D: Decode<R>, R: std::io::Read> {
    lexer: Lexer<D, R>,
    curr: Token,
}

impl<D: Decode<R>, R: std::io::Read> Parser<D, R> {
    pub fn new(input: R) -> Parser<D, R> {
        Parser {
            lexer: Lexer::<D, R>::new(D::new(input)),
            curr: Token::EOF,
        }
    }

    // root : function
    //      | extern
    pub fn parse(&mut self, unit: &mut ast::Module) {
        self.get_next_token();
        loop {
            match self.curr {
                Token::EOF => return,
                Token::SemiColon => self.get_next_token(),
                Token::Func => unit.push(ast::GlobalDecl::Function(self.parse_function())),
                Token::Extern => unit.push(ast::GlobalDecl::FuncDecl(self.parse_extern())),
                _ => panic!("unexpected token"),
            }
        }
    }

    // function : 'func' func_decl body
    fn parse_function(&mut self) -> ast::Func {
        // eat 'func'
        self.get_next_token();

        let proto = self.parse_func_decl();

        if self.curr != Token::LBrace {
            panic!("expected '{{'");
        }
        let body = self.parse_block_stmt();

        ast::Func::new(proto, body)
    }

    // extern : 'extern' func_decl ';'
    fn parse_extern(&mut self) -> ast::FuncDecl {
        // Eat 'extern'
        self.get_next_token();

        let decl = self.parse_func_decl();

        if self.curr != Token::SemiColon {
            panic!("expected ';'");
        }
        self.get_next_token();

        decl
    }

    // func_decl : identifier '(' params ')' ( ':' type )?
    // params    : ( param ( ',' param )* ','? )?
    // param     : identifier ':' type
    fn parse_func_decl(&mut self) -> ast::FuncDecl {
        let func_name = if let Token::Identifier(ref s) = self.curr {
            s.clone()
        } else {
            panic!("expected function name");
        };

        self.get_next_token();

        if self.curr != Token::LParen {
            panic!("expected '('");
        }
        self.get_next_token();

        let mut parameters = Vec::<ast::Param>::new();
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
            parameters.push(ast::Param::new(param_name, type_name));

            self.get_next_token();
            match self.curr {
                Token::Comma => self.get_next_token(),
                Token::RParen => break,
                _ => panic!("expected ')' or ','"),
            }
        }

        if self.curr != Token::RParen {
            panic!("expected ')");
        }

        self.get_next_token();
        if self.curr != Token::Colon {
            return ast::FuncDecl::new(func_name, String::from("void"), parameters);
        }

        self.get_next_token();
        let return_type = if let Token::Identifier(ref s) = self.curr {
            s.clone()
        } else {
            panic!("expected return type");
        };

        self.get_next_token();
        ast::FuncDecl::new(func_name, return_type, parameters)
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

        ast::Stmt::Block { stmts }
    }

    // ifstmt
    //   ::= 'if' expr block
    //   ::= 'if' expr block else block
    fn parse_if_stmt(&mut self) -> ast::Stmt {
        self.get_next_token();

        let cond = Box::new(self.parse_expr());

        if self.curr != Token::LBrace {
            panic!("expected '{{'");
        }
        let then_stmt = Box::new(self.parse_block_stmt());
        let else_stmt = None;

        if self.curr != Token::Else {
            return ast::Stmt::IfElse {
                cond,
                then_stmt,
                else_stmt,
            };
        }

        self.get_next_token();
        if self.curr != Token::LBrace {
            panic!("expected '{{'");
        }
        let else_stmt = Some(Box::new(self.parse_block_stmt()));

        ast::Stmt::IfElse {
            cond,
            then_stmt,
            else_stmt,
        }
    }

    // whilestmt ::= 'while' expr block
    fn parse_while_stmt(&mut self) -> ast::Stmt {
        self.get_next_token();

        let cond = Box::new(self.parse_expr());

        if self.curr != Token::LBrace {
            panic!("expected '{{'");
        }
        let body = Box::new(self.parse_block_stmt());

        ast::Stmt::While { cond, body }
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
        let expr = if self.curr == Token::Assign {
            self.get_next_token();
            Some(Box::new(self.parse_expr()))
        } else {
            None
        };

        if self.curr != Token::SemiColon {
            panic!("expected ';'");
        }

        self.get_next_token();

        ast::Stmt::VarDecl {
            var_name,
            type_name,
            expr,
        }
    }

    // returnstmt ::= 'return' expr? ';'
    fn parse_return_stmt(&mut self) -> ast::Stmt {
        self.get_next_token();

        let expr = if self.curr != Token::SemiColon {
            Some(Box::new(self.parse_expr()))
        } else {
            None
        };

        if self.curr != Token::SemiColon {
            panic!("expected ';'");
        }

        self.get_next_token();

        ast::Stmt::Return { expr }
    }

    // exprstmt ::= expr ';'
    fn parse_expr_stmt(&mut self) -> ast::Stmt {
        let expr = Box::new(self.parse_expr());

        if self.curr != Token::SemiColon {
            panic!("expected ';'");
        }

        self.get_next_token();

        ast::Stmt::ExprStmt { expr }
    }

    // expression
    //   ::= assignment
    fn parse_expr(&mut self) -> ast::Expr {
        self.parse_assignment()
    }

    // assignment ::= logical_or ('=' assignment)?
    fn parse_assignment(&mut self) -> ast::Expr {
        let lhs = self.parse_logical_or();

        if self.curr == Token::Assign {
            self.get_next_token();
            let rhs = self.parse_assignment();
            ast::Expr::Binary {
                op: ast::BinaryOp::Assignment,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
        } else {
            lhs
        }
    }

    fn parse_logical_or(&mut self) -> ast::Expr {
        self.parse_logical_and()
    }

    fn parse_logical_and(&mut self) -> ast::Expr {
        self.parse_bitwise_or()
    }

    // Associativity: Left
    // bitwise_or ::= bitwise_xor ('|' bitwise_xor)*
    fn parse_bitwise_or(&mut self) -> ast::Expr {
        let mut lhs = self.parse_bitwise_xor();
        loop {
            if self.curr != Token::BitwiseOr {
                return lhs;
            }
            self.get_next_token();

            lhs = ast::Expr::Binary {
                op: ast::BinaryOp::BitwiseOr,
                lhs: Box::new(lhs),
                rhs: Box::new(self.parse_bitwise_xor()),
            };
        }
    }

    // Associativity: Left
    // bitwise_xor ::= bitwise_and ('^' bitwise_and)*
    fn parse_bitwise_xor(&mut self) -> ast::Expr {
        let mut lhs = self.parse_bitwise_and();
        loop {
            if self.curr != Token::BitwiseXor {
                return lhs;
            }
            self.get_next_token();

            lhs = ast::Expr::Binary {
                op: ast::BinaryOp::BitwiseXor,
                lhs: Box::new(lhs),
                rhs: Box::new(self.parse_bitwise_and()),
            };
        }
    }

    // Associativity: Left
    // bitwise_and ::= equality ('&' equality)*
    fn parse_bitwise_and(&mut self) -> ast::Expr {
        let mut lhs = self.parse_equality();
        loop {
            if self.curr != Token::BitwiseAnd {
                return lhs;
            }
            self.get_next_token();

            lhs = ast::Expr::Binary {
                op: ast::BinaryOp::BitwiseAnd,
                lhs: Box::new(lhs),
                rhs: Box::new(self.parse_equality()),
            };
        }
    }

    // equality ::= relational (equality_op relational)*
    //
    // equality_op
    //  ::= '=='
    //  ::= '!='
    fn parse_equality(&mut self) -> ast::Expr {
        let mut lhs = self.parse_relational();
        loop {
            let op = match self.curr {
                Token::Eq => ast::BinaryOp::Eq,
                Token::Ne => ast::BinaryOp::Ne,
                _ => return lhs,
            };
            self.get_next_token();

            lhs = ast::Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(self.parse_relational()),
            };
        }
    }

    // relational ::= shift (relational_op shift)*
    //
    // relational_op
    //   ::= '<'
    //   ::= '>'
    //   ::= '<='
    //   ::= '>='
    fn parse_relational(&mut self) -> ast::Expr {
        let mut lhs = self.parse_shift();
        loop {
            let op = match self.curr {
                Token::Lt => ast::BinaryOp::Lt,
                Token::Gt => ast::BinaryOp::Gt,
                Token::Le => ast::BinaryOp::Le,
                Token::Ge => ast::BinaryOp::Ge,
                _ => return lhs,
            };
            self.get_next_token();

            lhs = ast::Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(self.parse_shift()),
            };
        }
    }

    // Associativity: Left
    // shift ::= addition (('<<' / '>>') addition)*
    fn parse_shift(&mut self) -> ast::Expr {
        let mut lhs = self.parse_addition();
        loop {
            let op = match self.curr {
                Token::LShift => ast::BinaryOp::LShift,
                Token::RShift => ast::BinaryOp::RShift,
                _ => return lhs,
            };
            self.get_next_token();

            lhs = ast::Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(self.parse_addition()),
            };
        }
    }

    // expr ::= multiplication ( ('+' / '-') multiplication)*
    fn parse_addition(&mut self) -> ast::Expr {
        let mut lhs = self.parse_multiplication();
        loop {
            let op = match self.curr {
                Token::Add => ast::BinaryOp::Add,
                Token::Sub => ast::BinaryOp::Sub,
                _ => return lhs,
            };
            self.get_next_token();

            lhs = ast::Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(self.parse_multiplication()),
            };
        }
    }

    // expr ::= unary ( ('*' / '/' / '%') unary)*
    fn parse_multiplication(&mut self) -> ast::Expr {
        let mut lhs = self.parse_unary();
        loop {
            let op = match self.curr {
                Token::Mul => ast::BinaryOp::Mul,
                Token::Div => ast::BinaryOp::Div,
                Token::Mod => ast::BinaryOp::Mod,
                _ => return lhs,
            };
            self.get_next_token();

            lhs = ast::Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(self.parse_unary()),
            };
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
            Token::LogicalNot => ast::UnaryOp::LogicalNot,
            Token::Sub => ast::UnaryOp::Neg,
            _ => return self.parse_primary(),
        };

        self.get_next_token();
        ast::Expr::Unary {
            op,
            operand: Box::new(self.parse_unary()),
        }
    }

    // primary
    //   ::= identifierexpr
    //   ::= numberexpr
    //   ::= parenexpr
    fn parse_primary(&mut self) -> ast::Expr {
        match self.curr {
            Token::Identifier(_) => self.parse_identifier_expr(),
            Token::Integer(_) => self.parse_number_expr(),
            Token::LParen => self.parse_paren_expr(),
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
        if self.curr != Token::LParen {
            return ast::Expr::Variable { name };
        }

        // This is a function call
        self.get_next_token();
        let mut args = Vec::<ast::Expr>::new();
        while self.curr != Token::RParen {
            args.push(self.parse_expr());
            if self.curr != Token::Comma {
                break;
            }
            self.get_next_token();
        }

        if self.curr != Token::RParen {
            panic!("expected ')'");
        }

        self.get_next_token();
        ast::Expr::Call {
            callee: name,
            arguments: args,
        }
    }

    // parenexpr ::= '(' expression ')'
    fn parse_paren_expr(&mut self) -> ast::Expr {
        self.get_next_token();
        let expr = self.parse_expr();
        if self.curr != Token::RParen {
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
        ast::Expr::Integer { value: number }
    }

    fn get_next_token(&mut self) {
        self.curr = self.lexer.gettok();
    }
}

#[cfg(test)]
mod tests {
    use super::super::utf8::Utf8Decoder;
    use super::ast::*;
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

        let mut parser = Parser::<Utf8Decoder<_>, _>::new(src.as_bytes());
        let mut unit = ast::Module::new();
        parser.parse(&mut unit);

        let mut expected = ast::Module::new();
        let func = Func::new(
            ast::FuncDecl::new(
                String::from("fib"),
                String::from("Int64"),
                vec![ast::Param::new(String::from("n"), String::from("Int64"))],
            ),
            ast::Stmt::Block {
                stmts: vec![ast::Stmt::IfElse {
                    cond: Box::new(ast::Expr::Variable {
                        name: String::from("n"),
                    }),
                    then_stmt: Box::new(ast::Stmt::Block {
                        stmts: vec![ast::Stmt::IfElse {
                            cond: Box::new(ast::Expr::Binary {
                                op: ast::BinaryOp::Sub,
                                lhs: Box::new(ast::Expr::Variable {
                                    name: String::from("n"),
                                }),
                                rhs: Box::new(ast::Expr::Integer { value: 1 }),
                            }),
                            then_stmt: Box::new(ast::Stmt::Block {
                                stmts: vec![ast::Stmt::Return {
                                    expr: Some(Box::new(ast::Expr::Binary {
                                        op: ast::BinaryOp::Add,
                                        lhs: Box::new(ast::Expr::Call {
                                            callee: String::from("fib"),
                                            arguments: vec![ast::Expr::Binary {
                                                op: ast::BinaryOp::Sub,
                                                lhs: Box::new(ast::Expr::Variable {
                                                    name: String::from("n"),
                                                }),
                                                rhs: Box::new(ast::Expr::Integer { value: 1 }),
                                            }],
                                        }),
                                        rhs: Box::new(ast::Expr::Call {
                                            callee: String::from("fib"),
                                            arguments: vec![ast::Expr::Binary {
                                                op: ast::BinaryOp::Sub,
                                                lhs: Box::new(ast::Expr::Variable {
                                                    name: String::from("n"),
                                                }),
                                                rhs: Box::new(ast::Expr::Integer { value: 2 }),
                                            }],
                                        }),
                                    })),
                                }],
                            }),
                            else_stmt: Some(Box::new(ast::Stmt::Block {
                                stmts: vec![ast::Stmt::Return {
                                    expr: Some(Box::new(ast::Expr::Integer { value: 1 })),
                                }],
                            })),
                        }],
                    }),
                    else_stmt: Some(Box::new(ast::Stmt::Block {
                        stmts: vec![ast::Stmt::Return {
                            expr: Some(Box::new(ast::Expr::Integer { value: 1 })),
                        }],
                    })),
                }],
            },
        );
        let decl = ast::GlobalDecl::Function(func);
        expected.push(decl);

        assert_eq!(unit, expected);
    }

    #[test]
    fn fib_with_rel_op() {
        let src = String::from(
            r"func fib(n: Int64) : Int64 {
            if n < 2 {
                return 1;
            } else {
                return fib(n - 1) + fib(n - 2);
            }
}
",
        );

        let mut parser = Parser::<Utf8Decoder<_>, _>::new(src.as_bytes());
        let mut unit = ast::Module::new();
        parser.parse(&mut unit);

        let mut expected = ast::Module::new();
        let func = Func::new(
            ast::FuncDecl::new(
                String::from("fib"),
                String::from("Int64"),
                vec![ast::Param::new(String::from("n"), String::from("Int64"))],
            ),
            ast::Stmt::Block {
                stmts: vec![ast::Stmt::IfElse {
                    cond: Box::new(ast::Expr::Binary {
                        op: ast::BinaryOp::Lt,
                        lhs: Box::new(ast::Expr::Variable {
                            name: String::from("n"),
                        }),
                        rhs: Box::new(ast::Expr::Integer { value: 2 }),
                    }),
                    then_stmt: Box::new(ast::Stmt::Block {
                        stmts: vec![ast::Stmt::Return {
                            expr: Some(Box::new(ast::Expr::Integer { value: 1 })),
                        }],
                    }),
                    else_stmt: Some(Box::new(ast::Stmt::Block {
                        stmts: vec![ast::Stmt::Return {
                            expr: Some(Box::new(ast::Expr::Binary {
                                op: ast::BinaryOp::Add,
                                lhs: Box::new(ast::Expr::Call {
                                    callee: String::from("fib"),
                                    arguments: vec![ast::Expr::Binary {
                                        op: ast::BinaryOp::Sub,
                                        lhs: Box::new(ast::Expr::Variable {
                                            name: String::from("n"),
                                        }),
                                        rhs: Box::new(ast::Expr::Integer { value: 1 }),
                                    }],
                                }),
                                rhs: Box::new(ast::Expr::Call {
                                    callee: String::from("fib"),
                                    arguments: vec![ast::Expr::Binary {
                                        op: ast::BinaryOp::Sub,
                                        lhs: Box::new(ast::Expr::Variable {
                                            name: String::from("n"),
                                        }),
                                        rhs: Box::new(ast::Expr::Integer { value: 2 }),
                                    }],
                                }),
                            })),
                        }],
                    })),
                }],
            },
        );
        let decl = ast::GlobalDecl::Function(func);
        expected.push(decl);

        assert_eq!(unit, expected);
    }

    #[test]
    fn extern_func() {
        let src = r"extern foo(a: Int64, b: Int64): Int64;".to_owned();
        let mut parser = Parser::<Utf8Decoder<_>, _>::new(src.as_bytes());

        parser.get_next_token();

        let decl = parser.parse_extern();
        assert_eq!(
            decl,
            ast::FuncDecl::new(
                String::from("foo"),
                String::from("Int64"),
                vec![
                    ast::Param::new(String::from("a"), String::from("Int64")),
                    ast::Param::new(String::from("b"), String::from("Int64")),
                ]
            )
        );
    }

    #[test]
    fn extern_func2() {
        let src = r"extern foo(a: Int64, b: Int64): Int64;
extern foo(a: Int64, b: Int64): Int64;"
            .to_owned();
        let mut parser = Parser::<Utf8Decoder<_>, _>::new(src.as_bytes());

        parser.get_next_token();

        for _ in 0..2 {
            let decl = parser.parse_extern();
            assert_eq!(
                decl,
                ast::FuncDecl::new(
                    String::from("foo"),
                    String::from("Int64"),
                    vec![
                        ast::Param::new(String::from("a"), String::from("Int64")),
                        ast::Param::new(String::from("b"), String::from("Int64")),
                    ]
                )
            );
        }
    }

    #[test]
    fn bitwise_or() {
        let src = String::from("a | b");
        let mut parser = Parser::<Utf8Decoder<_>, _>::new(src.as_bytes());

        parser.get_next_token();
        let expr = parser.parse_expr();
        assert_eq!(
            expr,
            ast::Expr::Binary {
                op: ast::BinaryOp::BitwiseOr,
                lhs: Box::new(ast::Expr::Variable {
                    name: String::from("a")
                }),
                rhs: Box::new(ast::Expr::Variable {
                    name: String::from("b")
                })
            }
        )
    }

    #[test]
    fn bitwise_or_associativity() {
        let src = String::from("a | b | c");
        let mut parser = Parser::<Utf8Decoder<_>, _>::new(src.as_bytes());

        parser.get_next_token();
        let expr = parser.parse_expr();
        assert_eq!(
            expr,
            ast::Expr::Binary {
                op: ast::BinaryOp::BitwiseOr,
                lhs: Box::new(ast::Expr::Binary {
                    op: ast::BinaryOp::BitwiseOr,
                    lhs: Box::new(ast::Expr::Variable {
                        name: String::from("a")
                    }),
                    rhs: Box::new(ast::Expr::Variable {
                        name: String::from("b")
                    })
                }),
                rhs: Box::new(ast::Expr::Variable {
                    name: String::from("c")
                })
            }
        )
    }

    #[test]
    fn bitwise_or_and_precedence() {
        let src = String::from("a | b & c");
        let mut parser = Parser::<Utf8Decoder<_>, _>::new(src.as_bytes());

        parser.get_next_token();
        let expr = parser.parse_expr();
        assert_eq!(
            expr,
            ast::Expr::Binary {
                op: ast::BinaryOp::BitwiseOr,
                lhs: Box::new(ast::Expr::Variable {
                    name: String::from("a")
                }),
                rhs: Box::new(ast::Expr::Binary {
                    op: ast::BinaryOp::BitwiseAnd,
                    lhs: Box::new(ast::Expr::Variable {
                        name: String::from("b")
                    }),
                    rhs: Box::new(ast::Expr::Variable {
                        name: String::from("c")
                    })
                })
            }
        )
    }

    #[test]
    fn bitwise_or_and_precedence2() {
        let src = String::from("a | b & c & d");
        let mut parser = Parser::<Utf8Decoder<_>, _>::new(src.as_bytes());

        parser.get_next_token();
        let expr = parser.parse_expr();
        assert_eq!(
            expr,
            ast::Expr::Binary {
                op: ast::BinaryOp::BitwiseOr,
                lhs: Box::new(ast::Expr::Variable {
                    name: String::from("a")
                }),
                rhs: Box::new(ast::Expr::Binary {
                    op: ast::BinaryOp::BitwiseAnd,
                    lhs: Box::new(ast::Expr::Binary {
                        op: ast::BinaryOp::BitwiseAnd,
                        lhs: Box::new(ast::Expr::Variable {
                            name: String::from("b")
                        }),
                        rhs: Box::new(ast::Expr::Variable {
                            name: String::from("c")
                        })
                    }),
                    rhs: Box::new(ast::Expr::Variable {
                        name: String::from("d")
                    }),
                })
            }
        )
    }

    #[test]
    fn shift() {
        let src = String::from("a << b");
        let mut parser = Parser::<Utf8Decoder<_>, _>::new(src.as_bytes());

        parser.get_next_token();
        let expr = parser.parse_expr();
        assert_eq!(
            expr,
            ast::Expr::Binary {
                op: ast::BinaryOp::LShift,
                lhs: Box::new(ast::Expr::Variable {
                    name: String::from("a")
                }),
                rhs: Box::new(ast::Expr::Variable {
                    name: String::from("b")
                })
            }
        )
    }

    #[test]
    fn shift2() {
        let src = String::from("a << b >> c");
        let mut parser = Parser::<Utf8Decoder<_>, _>::new(src.as_bytes());

        parser.get_next_token();
        let expr = parser.parse_expr();
        assert_eq!(
            expr,
            ast::Expr::Binary {
                op: ast::BinaryOp::RShift,
                lhs: Box::new(ast::Expr::Binary {
                    op: ast::BinaryOp::LShift,
                    lhs: Box::new(ast::Expr::Variable {
                        name: String::from("a")
                    }),
                    rhs: Box::new(ast::Expr::Variable {
                        name: String::from("b")
                    })
                }),
                rhs: Box::new(ast::Expr::Variable {
                    name: String::from("c")
                })
            }
        )
    }

    #[test]
    fn shift3() {
        let src = String::from("a << b << c");
        let mut parser = Parser::<Utf8Decoder<_>, _>::new(src.as_bytes());

        parser.get_next_token();
        let expr = parser.parse_expr();
        assert_eq!(
            expr,
            ast::Expr::Binary {
                op: ast::BinaryOp::LShift,
                lhs: Box::new(ast::Expr::Binary {
                    op: ast::BinaryOp::LShift,
                    lhs: Box::new(ast::Expr::Variable {
                        name: String::from("a")
                    }),
                    rhs: Box::new(ast::Expr::Variable {
                        name: String::from("b")
                    })
                }),
                rhs: Box::new(ast::Expr::Variable {
                    name: String::from("c")
                }),
            }
        )
    }
}
