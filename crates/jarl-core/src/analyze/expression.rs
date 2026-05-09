use air_r_syntax::{
    AnyRExpression, AnyRParameterName, RBinaryExpressionFields, RForStatementFields,
    RIfStatementFields, RSyntaxKind, RWhileStatementFields,
};
use std::collections::HashSet;

use crate::analyze;
use crate::checker::Checker;

/// Dispatch an expression to its appropriate set of rules and recurse into children.
///
/// Some expression types do both (e.g. RBinaryExpression), some only do the
/// dispatch to rules (e.g. RIdentifier), some only do the recursive call (e.g.
/// RFunctionDefinition).
///
/// Not all patterns are covered but they don't necessarily have to be.
/// For instance, there are currently no rule for RNaExpression and
/// it doesn't have any children expression on which we need to call
/// check_expression().
///
/// If a rule needs to be applied on RNaExpression in the future, then
/// we can add the corresponding match arm at this moment.
pub(crate) fn check_expression(
    expression: &air_r_syntax::AnyRExpression,
    checker: &mut Checker,
) -> anyhow::Result<()> {
    match expression {
        AnyRExpression::AnyRValue(children) => {
            analyze::anyvalue::anyvalue(children, checker)?;
        }
        AnyRExpression::RBinaryExpression(children) => {
            analyze::binary_expression::binary_expression(children, checker)?;
            let RBinaryExpressionFields { left, operator, right } = children.as_fields();
            let left = left?;
            let operator = operator?;
            let right = right?;

            check_expression(&left, checker)?;
            check_expression(&right, checker)?;

            match operator.kind() {
                RSyntaxKind::ASSIGN | RSyntaxKind::EQUAL => {
                    if let Some(name) = assigned_name(&left) {
                        checker.add_active_local_binding(name);
                    }
                }
                RSyntaxKind::ASSIGN_RIGHT => {
                    if let Some(name) = assigned_name(&right) {
                        checker.add_active_local_binding(name);
                    }
                }
                _ => {}
            }
        }
        AnyRExpression::RBracedExpressions(children) => {
            for expr in children.expressions() {
                check_expression(&expr, checker)?;
            }
        }
        AnyRExpression::RCall(children) => {
            analyze::call::call(children, checker)?;

            if let Some(ns_expr) = children.function()?.as_r_namespace_expression() {
                analyze::namespace_expression::namespace_expression(ns_expr, checker)?;
            }

            for arg in children.arguments()?.items() {
                if let Some(expr) = arg.unwrap().as_fields().value {
                    check_expression(&expr, checker)?;
                }
            }
        }
        AnyRExpression::RForStatement(children) => {
            analyze::for_loop::for_loop(children, checker)?;
            let RForStatementFields { variable, sequence, body, .. } = children.as_fields();
            let variable = variable?;
            analyze::identifier::identifier(&variable, checker)?;

            check_expression(&sequence?, checker)?;
            if let Ok(token) = variable.name_token() {
                checker.add_active_local_binding(token.token_text_trimmed().text().to_string());
            }
            check_expression(&body?, checker)?;
        }
        AnyRExpression::RFunctionDefinition(children) => {
            analyze::function_definition::function_definition(children, checker)?;
            let params = children.parameters()?.items();
            let mut local_names = HashSet::new();
            let mut defaults = Vec::new();
            for param in params {
                let param = param?;
                if let Ok(name) = param.name()
                    && let Some(name) = parameter_name(&name)
                {
                    local_names.insert(name);
                }
                let default = param.default();
                if let Some(default) = default
                    && let Ok(default) = default.value()
                {
                    defaults.push(default);
                }
            }
            checker.enter_local_scope(local_names);
            for default in defaults {
                check_expression(&default, checker)?;
            }
            check_expression(&children.body()?, checker)?;
            checker.exit_local_scope();
        }
        AnyRExpression::RIdentifier(x) => {
            analyze::identifier::identifier(x, checker)?;
        }
        AnyRExpression::RIfStatement(children) => {
            analyze::if_::if_(children, checker)?;

            let RIfStatementFields { condition, consequence, else_clause, .. } =
                children.as_fields();
            check_expression(&condition?, checker)?;
            check_expression(&consequence?, checker)?;
            if let Some(else_clause) = else_clause {
                let alternative = else_clause.alternative();
                check_expression(&alternative?, checker)?;
            }
        }
        AnyRExpression::RNamespaceExpression(children) => {
            analyze::namespace_expression::namespace_expression(children, checker)?;
        }
        AnyRExpression::RParenthesizedExpression(children) => {
            let body = children.body();
            check_expression(&body?, checker)?;
        }
        AnyRExpression::RRepeatStatement(children) => {
            let body = children.body();
            check_expression(&body?, checker)?;
        }
        AnyRExpression::RSubset(children) => {
            analyze::subset::subset(children, checker)?;

            for arg in children.arguments()?.items() {
                if let Some(expr) = arg?.value() {
                    check_expression(&expr, checker)?;
                }
            }
        }
        AnyRExpression::RSubset2(children) => {
            for arg in children.arguments()?.items() {
                if let Some(expr) = arg?.value() {
                    check_expression(&expr, checker)?;
                }
            }
        }
        AnyRExpression::RUnaryExpression(children) => {
            analyze::unary_expression::unary_expression(children, checker)?;

            let argument = children.argument();
            check_expression(&argument?, checker)?;
        }
        AnyRExpression::RWhileStatement(children) => {
            analyze::while_::while_(children, checker)?;
            let RWhileStatementFields { condition, body, .. } = children.as_fields();
            check_expression(&condition?, checker)?;
            check_expression(&body?, checker)?;
        }
        _ => {}
    }

    Ok(())
}

fn assigned_name(expression: &AnyRExpression) -> Option<String> {
    let identifier = expression.as_r_identifier()?;
    let token = identifier.name_token().ok()?;
    Some(token.token_text_trimmed().text().to_string())
}

fn parameter_name(name: &AnyRParameterName) -> Option<String> {
    let identifier = name.as_r_identifier()?;
    let token = identifier.name_token().ok()?;
    Some(token.token_text_trimmed().text().to_string())
}
