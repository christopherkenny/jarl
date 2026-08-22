use crate::diagnostic::*;
use crate::lints::base::assignment_on_if_no_else::assignment_on_if_no_else::assigned_if_statement;
use air_r_syntax::*;
use biome_rowan::AstNode;

pub struct AssignmentOnIf;

/// Version added: 0.6.0
///
/// ## What it does
///
/// Flags assignments whose value is an `if` expression.
/// When enabled, it supersedes `assignment_on_if_no_else`.
///
/// ## Why is this bad?
///
/// Assigning the result of an `if` expression can make the assignment easy to
/// overlook and can make the two branches harder to compare. Without a final
/// `else`, the assignment can also replace an existing value with `NULL`.
///
/// ## Example
///
/// ```r
/// x <- if (condition) {
///   value <- f()
///   g(value)
/// } else {
///   other
/// }
/// ```
///
/// Use instead:
///
/// ```r
/// if (condition) {
///   value <- f()
///   x <- g(value)
/// } else {
///   x <- other
/// }
/// ```
///
impl Violation for AssignmentOnIf {
    fn name(&self) -> String {
        "assignment_on_if".to_string()
    }

    fn body(&self) -> String {
        "Avoid assigning the result of this `if` expression.".to_string()
    }

    fn suggestion(&self) -> Option<String> {
        Some(
            "Move the assignment into the `if` branches or extract the conditional into a helper function."
                .to_string(),
        )
    }
}

pub fn assignment_on_if(ast: &RBinaryExpression) -> anyhow::Result<Option<Diagnostic>> {
    if assigned_if_statement(ast)?.is_none() {
        return Ok(None);
    }

    Ok(Some(Diagnostic::new(
        AssignmentOnIf,
        ast.syntax().text_trimmed_range(),
        Fix::empty(),
    )))
}
