use crate::ast::Message;

/// Canonical intermediate representation consumed by generators.
///
/// The adapter normalizes `.msg` and `.srv` files into
/// one or multiple `Message` units.
#[derive(Debug, Clone)]
pub struct CanonicalIr {
    pub source_name: String,
    pub messages: Vec<Message>,
}
