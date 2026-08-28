use air_r_syntax::*;
use biome_rowan::AstNode;
use oak_semantic::semantic_index::{DefinitionKind, SemanticIndex};

use jarl_semantic::{SemanticInfo, assignment_lhs_is_complex};

use crate::checker::Checker;
use crate::diagnostic::*;
use crate::rule_set::Rule;

/// Version added: 0.7.0
///
/// ## What it does
///
/// Flags an assignment whose value is an `if` expression without a final
/// `else` branch, when it overwrites a value that was assigned earlier and
/// never used.
///
/// ## Why is this bad?
///
/// When no branch is taken, an `if` expression without a final `else`
/// evaluates to `NULL`. Assigning that result silently discards the previous
/// value, which is usually a bug: the earlier assignment reads like a fallback
/// but never acts as one.
///
/// ```r
/// x <- 1
/// x <- if (foo) y
/// ```
///
/// Here `x` is `NULL` when `foo` is `FALSE`, not `1`.
///
/// Writing `x <- if (foo) y` on its own is not reported: with nothing to
/// overwrite, `NULL` is usually the intended result.
///
/// ## Example
///
/// ```r
/// df <- data.frame()
/// df <- if (condition) {
///   read_data()
/// }
///
/// value <- 0
/// value <- if (a) {
///   1
/// } else if (b) {
///   2
/// }
/// ```
///
/// Use instead:
///
/// ```r
/// df <- data.frame()
/// if (condition) {
///   df <- read_data()
/// }
///
/// value <- if (a) {
///   1
/// } else if (b) {
///   2
/// } else {
///   0
/// }
/// ```
///
/// ## Limitations
///
/// Super-assignment (`x <<- if (a) 1`, `if (a) 1 ->> x`) is not checked: it
/// binds in an enclosing environment, where whether the overwritten value was
/// used cannot be determined reliably.
pub fn assignment_on_if_no_else(
    info: &SemanticInfo<'_>,
    semantic: &SemanticIndex,
    checker: &mut Checker,
) -> anyhow::Result<()> {
    let mut diagnostics = Vec::new();

    for scope_id in info.scope_ids() {
        let definitions = semantic.definitions(scope_id);
        for (def_id, def) in definitions.iter() {
            let DefinitionKind::Assignment(ptr) = def.kind() else {
                continue;
            };
            if info.is_in_nse(def.range()) || info.is_in_formula(def.range()) {
                continue;
            }
            let assignment = ptr.to_node(info.root());
            if !assigns_if_without_else(&assignment)? {
                continue;
            }

            // Every earlier binding of this name in this scope: the values
            // this assignment can overwrite. Restricting to the same scope is
            // what makes `x <- 1` outside a function irrelevant to a
            // `x <- if (a) y` inside it, which creates a fresh local instead.
            let mut overwrites_anything = false;
            let mut all_unused = true;
            for (previous_id, previous) in definitions.iter() {
                if previous_id == def_id
                    || previous.symbol() != def.symbol()
                    || previous.range().start() >= def.range().start()
                    || info.is_in_nse(previous.range())
                {
                    continue;
                }
                overwrites_anything = true;
                if info.is_definition_used(scope_id, previous_id) {
                    all_unused = false;
                    break;
                }
            }
            if !overwrites_anything || !all_unused {
                continue;
            }

            let name = semantic.symbols(scope_id).symbol(def.symbol()).name();
            diagnostics.push(Diagnostic::new(
                ViolationData::new(
                    Rule::AssignmentOnIfNoElse,
                    format!(
                        "Object `{name}` was assigned before and never used. This assignment \
                         overwrites it with `NULL` when no `if` branch is taken."
                    ),
                    Some(
                        "Move the assignment into the `if` branches or add a final `else` branch."
                            .to_string(),
                    ),
                ),
                assignment.syntax().text_trimmed_range(),
                Fix::empty(),
            ));
        }
    }

    for diagnostic in diagnostics {
        checker.report_diagnostic(Some(diagnostic));
    }

    Ok(())
}

/// Whether `assignment` is a plain binding whose value is an `if` expression
/// with no final `else`.
fn assigns_if_without_else(assignment: &RBinaryExpression) -> anyhow::Result<bool> {
    let operator = assignment.operator()?;
    // Super-assignment binds in an enclosing environment, see "Limitations".
    let value = match operator.kind() {
        RSyntaxKind::ASSIGN | RSyntaxKind::EQUAL => assignment.right()?,
        RSyntaxKind::ASSIGN_RIGHT => assignment.left()?,
        _ => return Ok(false),
    };
    // `names(x) <-` or `x[1] <-` read the current value before replacing part
    // of it, so nothing is overwritten wholesale.
    if assignment_lhs_is_complex(assignment) {
        return Ok(false);
    }
    let Some(if_statement) = as_if_statement(value)? else {
        return Ok(false);
    };
    has_no_final_else(&if_statement)
}

fn as_if_statement(expression: AnyRExpression) -> anyhow::Result<Option<RIfStatement>> {
    match expression {
        AnyRExpression::RIfStatement(if_statement) => Ok(Some(if_statement)),
        AnyRExpression::RParenthesizedExpression(parenthesized) => {
            as_if_statement(parenthesized.body()?)
        }
        _ => Ok(None),
    }
}

fn has_no_final_else(if_statement: &RIfStatement) -> anyhow::Result<bool> {
    let Some(else_clause) = if_statement.else_clause() else {
        return Ok(true);
    };

    let alternative = else_clause.alternative()?;
    let Some(next_if) = alternative.as_r_if_statement() else {
        return Ok(false);
    };

    has_no_final_else(next_if)
}
