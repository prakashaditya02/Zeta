use crate::token::*;
use crate::expr::*;
use crate::token::TokenType as tt;
use crate::precedence::Precedence as Prec;
use crate::stmt::Stmt;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0
        }
    }

    pub fn parse_prog(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while self.peek().token != TokenType::Eof {
            statements.push(self.parse_stmt());
        }
        statements
    }

    fn parse_expr(&mut self) -> Expr {
        let c = &self.tokens[self.current];
        match c.token {

            TokenType::Number => {
                let num = c.lexeme.parse().unwrap();
                self.advance();
                return Expr::Number(num)
            }

            TokenType::Minus | TokenType::Bang => {
                let operator = self.get_unary_op(&self.peek().token);
                self.advance();
                let right = self.parse_prec(Prec::Unary);
                return Expr::Unary { 
                    operator: operator, 
                    right: Box::new(right)
                }
            },

            TokenType::LeftParen => {
                self.advance();
                let group = self.parse_prec(Prec::None);
                self.expect_token(TokenType::RightParen, "Expected ')'");
                return Expr::Grouping(Box::new(group));
            },

            TokenType::String => {
                let val = c.lexeme.clone();
                self.advance();
                Expr::String(val)
            },

            TokenType::Identifier => {
                let var = c.lexeme.clone();
                self.advance();
                Expr::Identifier(var)
            }

            TokenType::True => {
                self.advance();
                Expr::Bool(true)
            },

            TokenType::False => {
                self.advance();
                Expr::Bool(false)
            },

            TokenType::Nil => {
                self.advance();
                Expr::Nil
            }

            _ => panic!()
        }
    }

    fn parse_stmt(&mut self) -> Stmt {
        match self.peek().token {
            tt::Let => {
                self.advance();
                if self.peek().token != tt::Identifier {
                    panic!("Expected Identifier")
                }
                let name = self.peek().lexeme.clone();
                let mut val: Option<Expr> = None;
                self.advance();
                if self.peek().token == tt::Equal {
                    self.advance();
                    val = Some(self.parse_prec(Prec::None));
                }
                self.expect_token(tt::Semicolon, "Expected semicolon");
                return Stmt::Let { name: name, value: (val) }
            },

            tt::If => {
                self.advance();

                self.expect_token(tt::LeftParen, "Expected opening parentheses");
                let condition = self.parse_prec(Prec::None);
                self.expect_token(tt::RightParen, "Expected closing parentheses");

                let then = self.parse_block();

                let mut else_body = None;
                if self.peek().token == tt::Else {
                    self.advance();
                    else_body = Some(self.parse_block());
                }
                return Stmt::If { condition: condition, then_branch: then, else_branch: else_body }
            },

            tt::While => {
                self.advance();

                self.expect_token(tt::LeftParen, "Expected opening parentheses");
                let condition = self.parse_prec(Prec::None);
                self.expect_token(tt::RightParen, "Expected closing parentheses");
                let body = self.parse_block();
                return Stmt::While { condition: condition, body: body }
            },

            tt::Fun => {
                self.advance();

                if self.peek().token != tt::Identifier { panic!("Expected Identifier") }
                let name = self.peek().lexeme.clone();
                self.advance();

                self.expect_token(tt::LeftParen, "Expected opening parantheses");
                let mut arguments = Vec::new();
                loop {
                    if self.peek().token == tt::RightParen {
                        self.advance();
                        break;
                    }

                    if self.peek().token != tt::Identifier { panic!("Expected ")}
                    let arg: String = self.peek().lexeme.clone();
                    arguments.push(arg);
                    self.advance();
                    if self.peek().token != tt::RightParen {
                        self.expect_token(tt::Comma, "Expected Comma or closing parentheses");
                    }

                }

                let body = self.parse_block();
                Stmt::Fun { name: name, parameters: arguments, body: body }
            },

            tt::Return => {
                self.advance();
                let mut val = None;
                if self.peek().token != tt::Semicolon {
                    val = Some(self.parse_prec(Prec::None));
                }
                self.expect_token(tt::Semicolon, "Expected semicolon");
                Stmt::Return(val)
            },

            _ => {
                let expr = self.parse_prec(Prec::None);
                self.expect_token(tt::Semicolon, "Expected semicolon");
                return Stmt::Expression(expr)
            }   
        }
    }

    fn parse_call(&mut self, left: Expr) -> Expr {
        self.advance();
        let mut arguments = Vec::new();
        
        loop {
            if self.peek().token == TokenType::RightParen {
                self.advance();
                return Expr::Call { callee: Box::new(left), arguments: arguments }
            }

            let arg = self.parse_prec(Prec::Assignment);
            arguments.push(arg);

            if self.peek().token == TokenType::Comma {
                self.advance();
            } else if self.peek().token == TokenType::RightParen {
                self.advance();
                break;
            } else {
                panic!();
            }
        }
        return Expr::Call { callee: Box::new(left), arguments: arguments }
    }

    fn parse_block(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        self.expect_token(tt::LeftBrace, "Expect '{'");
        loop {
            if self.peek().token == tt::RightBrace {
                self.advance();
                break;
            }
            let statement = self.parse_stmt();
            stmts.push(statement);
        }
        return stmts;
    }

    fn get_unary_op(&self, token_type: &tt) -> UnaryOp {
        match token_type {
            tt::Bang => UnaryOp::Not,
            tt::Minus => UnaryOp::Negate,
            _ => panic!()
        }
    }

    fn get_binary_op(&self, token_type: &tt) -> BinaryOp {
        match token_type {
            tt::Plus => BinaryOp::Add,
            tt::Minus => BinaryOp::Subtract,
            tt::Star => BinaryOp::Multiply,
            tt::Slash => BinaryOp::Divide,
            tt::EqualEqual => BinaryOp::Equal,
            tt::BangEqual => BinaryOp::NotEqual,
            tt::Greater => BinaryOp::Greater,
            tt::GreaterEqual => BinaryOp::GreaterEqual,
            tt::Less => BinaryOp::Less,
            tt::LessEqual => BinaryOp::LessEqual,
            _ => panic!()
        }
    }

    fn get_infix_prec(&self, token_type: &tt) -> Prec {
        match token_type {
            tt::Plus | tt::Minus => Prec::Term,
            tt::Star | tt::Slash => Prec::Factor,
            tt::BangEqual | tt::EqualEqual => Prec::Equality,
            tt::Less | tt::Greater | tt::LessEqual | tt::GreaterEqual => Prec::Comparison,
            tt::Or => Prec::Or,
            tt::And => Prec::And,
            tt::Equal => Prec::Assignment,
            tt::LeftParen => Prec::Call,
            _ => Prec::None 
        }
    }

    fn parse_prec(&mut self, min_prec: Prec) -> Expr {
        let mut left = self.parse_expr();
        let mut precedence = self.get_infix_prec(&self.peek().token);
        
        while precedence > min_prec {
            if self.peek().token == tt::LeftParen {
                left = self.parse_call(left);
            } else if  self.peek().token == tt::Equal {
                match left {
                    Expr::Identifier(name) => {
                        let n = name.clone();
                        self.advance();
                        let value = self.parse_prec(min_prec);
                        left = Expr::Assignment { name:n, value:Box::new(value) };
                    }
                    _ => panic!("Variable name must be an Identifier")
                }
            } else if self.peek().token == tt::And {
                self.advance();
                let right = self.parse_prec(Prec::And);
                left = Expr::Logical {
                    left: Box::new(left),
                    operator: LogicalOp::And,
                    right: Box::new(right) };
            } else if self.peek().token == tt::Or {
                self.advance();
                let right = self.parse_prec(Prec::Or);
                left = Expr::Logical {
                    left: Box::new(left),
                    operator: LogicalOp::Or,
                    right: Box::new(right) };
            } else {
                let operator = self.get_binary_op(&self.peek().token);
                self.advance();
                let right = self.parse_prec(precedence);
                left = Expr::Binary { 
                    left: Box::new(left),
                    operator: operator,
                    right: Box::new(right) };
            }
            precedence = self.get_infix_prec(&self.peek().token);
        }
        return left;
    }

    fn advance(&mut self) {
        if (self.tokens[self.current]).token != tt::Eof {
            self.current += 1;
        }
    }

    fn peek(&self) -> &Token {
        return &self.tokens[self.current];
    }

    fn expect_token(&mut self, expected: TokenType, err: &str) {
        if self.peek().token != expected {
            panic!("{}", err);
        }
        self.advance();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_program_helper(source: &str) -> Vec<Stmt> {
        let mut lexer = Lexer::new(source.to_string());
        let tokens = lexer.scan_tokens();

        let mut parser = Parser::new(tokens);
        parser.parse_prog()
    }   

    fn parse(source: &str) -> Expr {
        let mut lexer = Lexer::new(source.to_string());
        let tokens = lexer.scan_tokens();

        let mut parser = Parser::new(tokens);
        parser.parse_prec(Prec::None)
    }

    #[test]
    fn parses_print_call_statement() {
        let stmts = parse_program_helper("print(1 + 2 * 3);");
        assert_eq!(
            stmts,
            vec![
                Stmt::Expression(
                    Expr::Call {
                        callee: Box::new(
                            Expr::Identifier("print".to_string())
                        ),
                        arguments: vec![
                            Expr::Binary {
                                left: Box::new(Expr::Number(1.0)),
                                operator: BinaryOp::Add,
                                right: Box::new(
                                    Expr::Binary {
                                        left: Box::new(Expr::Number(2.0)),
                                        operator: BinaryOp::Multiply,
                                        right: Box::new(Expr::Number(3.0)),
                                    }
                                ),
                            }
                        ],
                    }
                )
            ]
        );
    }

    #[test]
    fn parses_precedence() {
        let expr = parse("1 + 2 * 3");

        assert_eq!(
            expr,
            Expr::Binary {
                left: Box::new(Expr::Number(1.0)),
                operator: BinaryOp::Add,
                right: Box::new(
                    Expr::Binary {
                        left: Box::new(Expr::Number(2.0)),
                        operator: BinaryOp::Multiply,
                        right: Box::new(Expr::Number(3.0)),
                    }
                ),
            }
        );
    }

    #[test]
    fn parses_negation() {
        let expr = parse("-123");

        assert_eq!(
            expr,
            Expr::Unary {
                operator: UnaryOp::Negate,
                right: Box::new(Expr::Number(123.0)),
            }
        );
    }

    #[test]
    fn parses_double_negation() {
        let expr = parse("--123");

        assert_eq!(
            expr,
            Expr::Unary {
                operator: UnaryOp::Negate,
                right: Box::new(
                    Expr::Unary {
                        operator: UnaryOp::Negate,
                        right: Box::new(Expr::Number(123.0)),
                    }
                ),
            }
        );
    }

    #[test]
    fn grouping_overrides_precedence() {
        let expr = parse("(1 + 2) * 3");

        assert_eq!(
            expr,
            Expr::Binary {
                left: Box::new(
                    Expr::Grouping(
                        Box::new(
                            Expr::Binary {
                                left: Box::new(Expr::Number(1.0)),
                                operator: BinaryOp::Add,
                                right: Box::new(Expr::Number(2.0)),
                            }
                        )
                    )
                ),
                operator: BinaryOp::Multiply,
                right: Box::new(Expr::Number(3.0)),
            }
        );
    }

    #[test]
    fn parses_unary_with_binary() {
        let expr = parse("-1 + 2");

        assert_eq!(
            expr,
            Expr::Binary {
                left: Box::new(
                    Expr::Unary {
                        operator: UnaryOp::Negate,
                        right: Box::new(Expr::Number(1.0)),
                    }
                ),
                operator: BinaryOp::Add,
                right: Box::new(Expr::Number(2.0)),
            }
        );
    }

    #[test]
    fn parses_identifier_expression() {
        let expr = parse("foo + 123 * bar");

        assert_eq!(
            expr,
            Expr::Binary {
                left: Box::new(
                    Expr::Identifier("foo".to_string())
                ),
                operator: BinaryOp::Add,
                right: Box::new(
                    Expr::Binary {
                        left: Box::new(Expr::Number(123.0)),
                        operator: BinaryOp::Multiply,
                        right: Box::new(
                            Expr::Identifier("bar".to_string())
                        ),
                    }
                ),
            }
        );
    }

    #[test]
    fn parses_simple_assignment() {
        let expr = parse("x = 5");
        assert_eq!(
            expr,
            Expr::Assignment {
                name: "x".to_string(),
                value: Box::new(Expr::Number(5.0)),
            }
        );
    }

    #[test]
    fn parses_right_associative_assignment() {
        let expr = parse("x = y = 3");
        assert_eq!(
            expr,
            Expr::Assignment {
                name: "x".to_string(),
                value: Box::new(Expr::Assignment {
                    name: "y".to_string(),
                    value: Box::new(Expr::Number(3.0)),
                }),
            }
        );
    }

    #[test]
    fn parses_let_with_value() {
        let stmts = parse_program_helper("let x = 5;");
        assert_eq!(
            stmts,
            vec![
                Stmt::Let {
                    name: "x".to_string(),
                    value: Some(Expr::Number(5.0)),
                }
            ]
        );
    }

    #[test]
    fn parses_let_with_semicolon() {
        let stmts = parse_program_helper("let x = 5;");
        assert_eq!(
            stmts,
            vec![
                Stmt::Let {
                    name: "x".to_string(),
                    value: Some(Expr::Number(5.0)),
                }
            ]
        );
    }

    #[test]
    fn parses_function() {
        let stmts = parse_program_helper(
            "fun add(a, b) { return a + b; }"
        );

        assert_eq!(
            stmts,
            vec![
                Stmt::Fun {
                    name: "add".to_string(),
                    parameters: vec![
                        "a".to_string(),
                        "b".to_string(),
                    ],
                    body: vec![
                        Stmt::Return(Some(
                            Expr::Binary {
                                left: Box::new(Expr::Identifier("a".to_string())),
                                operator: BinaryOp::Add,
                                right: Box::new(Expr::Identifier("b".to_string())),
                            }
                        ))
                    ],
                }
            ]
        );
    }

    #[test]
    fn parses_return() {
        let stmts = parse_program_helper("return 1 + 2;");

        assert_eq!(
            stmts,
            vec![
                Stmt::Return(Some(
                    Expr::Binary {
                        left: Box::new(Expr::Number(1.0)),
                        operator: BinaryOp::Add,
                        right: Box::new(Expr::Number(2.0)),
                    }
                ))
            ]
        );
    }
}