use crate::{
    error::AssemblyError,
    parser::expression::{matcher::parenthesized, Expr},
};

type Decoder<T> = fn(&Expr) -> Result<T, AssemblyError>;

pub fn parenthes_within<T>(expr: &Expr, decoder: Decoder<T>) -> Result<T, AssemblyError> {
    parenthesized(&expr).and_then(|expr| decoder(&expr))
}
