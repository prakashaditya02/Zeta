use std::rc::Rc;
use crate::chunk::*;

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Fun(Rc<Chunk>)
}