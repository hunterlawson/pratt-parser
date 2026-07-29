use std::fmt::{Debug, Display};

use strum::{Display, EnumIter, IntoEnumIterator};

/// Enums that implement Operator are valid operators for use by the parser
pub trait Operator: Display + IntoEnumIterator + Clone + Copy + Debug + PartialEq {
    /// Get the binding power for this operator type
    fn binding_power(&self) -> (Option<u16>, Option<u16>);
}

/// Enums that implement delimited are valid delimited types for use by the parser
pub trait Delimited: IntoEnumIterator + PartialEq + Clone + Debug {
    /// Get the left and right delineators for this type
    fn delimiters(&self) -> Option<(String, String)> {
        None
    }
    /// Set the value of this delimited type
    fn set(&mut self, input: String) {}
}

/// Default operator type
#[derive(Display, EnumIter, PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum DefaultOperators {
    // arithmetic
    #[strum(to_string = "+")]
    Add,
    #[strum(to_string = "-")]
    Sub,
    #[strum(to_string = "*")]
    Mul,
    #[strum(to_string = "/")]
    Div,
    #[strum(to_string = "%")]
    Mod,
    // comparison
    #[strum(to_string = "==")]
    Eq,
    #[strum(to_string = "!=")]
    Neq,
    #[strum(to_string = ">")]
    Gt,
    #[strum(to_string = "<")]
    Lt,
    #[strum(to_string = ">=")]
    Gte,
    #[strum(to_string = "<=")]
    Lte,
    // logical
    #[strum(to_string = "&&")]
    And,
    #[strum(to_string = "||")]
    Or,
    #[strum(to_string = "!")]
    Not,
}

impl Operator for DefaultOperators {
    fn binding_power(&self) -> (Option<u16>, Option<u16>) {
        (None, None)
    }
}
/// Null delimited type
#[derive(EnumIter, Debug, PartialEq, Clone)]
pub enum NullDelimiter {}
impl Delimited for NullDelimiter {}
