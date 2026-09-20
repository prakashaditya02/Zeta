use std::rc::Rc;
use crate::chunk::*;

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Fun(Rc<Chunk>),
    Bool(bool),
    Nil,
}

impl Value {
    pub fn is_falsy(&self) -> bool {
        match self {
            Value::Number(n) => *n == 0.0,
            Value::String(s) => s.is_empty(),
            Value::Fun(_) => return false,
            Value::Bool(b) => return !b,
            Value::Nil => return true,
        }
    }
}