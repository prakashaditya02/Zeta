#[derive(PartialEq, Debug)]
pub enum Expr {
    Nil,
    Assignment {name: String, value: Box<Expr>},
    Binary {left: Box<Expr>, operator: BinaryOp, right: Box<Expr>},
    Logical {left: Box<Expr>, operator: LogicalOp, right: Box<Expr>},
    Unary {operator: UnaryOp, right: Box<Expr>},
    Call {callee: Box<Expr>, arguments: Vec<Expr>},
    Number(f64),
    Grouping(Box<Expr>),
    String(String),
    Bool(bool),
    Identifier(String),
}

#[derive(PartialEq, Debug)]
pub enum BinaryOp {
    Add, Subtract, Multiply, Divide,
    Equal, NotEqual, Greater, GreaterEqual, Less, LessEqual,
}

#[derive(PartialEq, Debug)]
pub enum UnaryOp {
    Negate, Not, 
}

#[derive(PartialEq, Debug)]
pub enum LogicalOp {
    And, Or,
}