use crate::diagnostic::*;
use air_r_syntax::*;
use biome_rowan::AstNode;

/// Version added: 0.6.0
///
/// ## What it does
///
/// Checks for assignments whose value is an `if()` expression, including
/// assignments used as named function arguments.
///
/// ## Why is this bad?
///
/// Assigning the result of an `if()` expression makes the assignment easy to
/// overlook and can make the two branches harder to compare. Assigning in
/// each branch makes the side effect explicit.
///
/// ## Example
///
/// ```r
/// x <- if (condition) value else other
/// fn(arg = if (condition) value else other)
/// ```
///
/// For simple values, especially with a vectorized condition, use `ifelse()`:
///
/// ```r
/// x <- ifelse(condition, value, other)
/// fn(arg = ifelse(condition, value, other))
/// ```
///
/// For scalar conditions with side effects or more complex branch logic, move
/// the assignment into each branch:
///
/// ```r
/// if (condition) {
///   x <- value
/// } else {
///   x <- other
/// }
/// ```
pub fn assignment_on_if(ast: &RIfStatement) -> anyhow::Result<Option<Diagnostic>> {
    // Parentheses do not change the fact that the `if` is the whole assigned
    // value, so walk through them before looking for an assignment-bearing
    // parent.
    let mut node = ast.syntax().clone();

    loop {
        let Some(parent) = node.parent() else {
            return Ok(None);
        };

        if let Some(assignment) = RBinaryExpression::cast_ref(&parent) {
            let operator = assignment.operator()?;

            let value_is_if = match operator.kind() {
                RSyntaxKind::ASSIGN | RSyntaxKind::EQUAL | RSyntaxKind::SUPER_ASSIGN => {
                    assignment.right()?.syntax() == &node
                }
                RSyntaxKind::ASSIGN_RIGHT | RSyntaxKind::SUPER_ASSIGN_RIGHT => {
                    assignment.left()?.syntax() == &node
                }
                _ => false,
            };

            if value_is_if {
                return Ok(Some(diagnostic(assignment.syntax().text_trimmed_range())));
            }

            return Ok(None);
        }

        if let Some(argument) = RArgument::cast_ref(&parent) {
            let Some(value) = argument.value() else {
                return Ok(None);
            };

            if value.syntax() == &node && argument.name_clause().is_some() {
                return Ok(Some(diagnostic(argument.syntax().text_trimmed_range())));
            }

            return Ok(None);
        }

        if parent.kind() == RSyntaxKind::R_PARENTHESIZED_EXPRESSION {
            node = parent;
            continue;
        }

        return Ok(None);
    }
}

fn diagnostic(range: biome_rowan::TextRange) -> Diagnostic {
    Diagnostic::new(
        ViolationData::new(
            "assignment_on_if".to_string(),
            "Avoid assigning the result of an `if()` expression.".to_string(),
            Some("Use `ifelse()` for simple values, or move the assignment into each branch of the `if()` expression.".to_string()),
        ),
        range,
        Fix::empty(),
    )
}
