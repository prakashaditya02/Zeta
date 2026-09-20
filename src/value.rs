use std::rc::Rc;
use std::fmt;
use crate::chunk::*;

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Fun(Rc<Chunk>),
    Bool(bool),
    Nil,
    NativeFn(fn (&[Value]) -> Value),
}

impl Value {
    pub fn is_falsy(&self) -> bool {
        match self {
            Value::Number(n) => *n == 0.0,
            Value::String(s) => s.is_empty(),
            Value::Bool(b) => return !b,
            Value::Nil => return true,
            _ => return false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Fun(_) => write!(f, "<function>"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
            Value::NativeFn(_) => write!(f, "<native function>"),
        }
    }
}