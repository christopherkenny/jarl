use std::collections::HashSet;

use air_r_syntax::{
    AnyRArgumentName, AnyRExpression, AnyRParameterName, AnyRValue, RArgument, RBinaryExpression,
    RCall, RParameter, RSyntaxKind, RSyntaxNode,
};
use biome_rowan::{AstNode, AstSeparatedList, TextRange};
use oak_core::syntax_ext::{RIdentifierExt, RStringValueExt};

use crate::diagnostic::{Diagnostic, Fix, ViolationData};
use crate::lints::base::object_name::options::ResolvedObjectNameOptions;
use crate::rule_set::Rule;
use crate::utils::get_function_name;

// Common base/S3 and group generics. Installed-package metadata extends this
// set when the package cache is available.
const S3_GENERICS: &[&str] = &[
    "[",
    "[[",
    "$",
    "[<-",
    "[[<-",
    "$<-",
    "all.equal",
    "anova",
    "AIC",
    "as.character",
    "as.complex",
    "as.data.frame",
    "as.Date",
    "as.difftime",
    "as.double",
    "as.factor",
    "as.function",
    "as.integer",
    "as.list",
    "as.logical",
    "as.matrix",
    "as.name",
    "as.numeric",
    "as.POSIXct",
    "as.POSIXlt",
    "as.raw",
    "as.symbol",
    "as.vector",
    "attributes",
    "attributes<-",
    "BIC",
    "body<-",
    "c",
    "cbind",
    "class",
    "class<-",
    "colnames",
    "colnames<-",
    "coef",
    "conditionCall",
    "conditionMessage",
    "confint",
    "deviance",
    "df.residual",
    "dim",
    "dim<-",
    "dimnames",
    "dimnames<-",
    "duplicated",
    "environment<-",
    "format",
    "formula",
    "fitted",
    "head",
    "is.na",
    "is.na<-",
    "length",
    "length<-",
    "levels",
    "levels<-",
    "logLik",
    "Math",
    "mean",
    "median",
    "na.exclude",
    "na.omit",
    "names",
    "names<-",
    "oldClass",
    "oldClass<-",
    "Ops",
    "plot",
    "print",
    "predict",
    "quantile",
    "rbind",
    "rep",
    "residuals",
    "rownames",
    "rownames<-",
    "round",
    "seq",
    "sort",
    "split",
    "str",
    "Summary",
    "summary",
    "t",
    "terms",
    "transform",
    "trunc",
    "unique",
    "update",
    "vcov",
    "weights",
    "within",
    // S3 group generics and their operator methods.
    "+",
    "-",
    "*",
    "/",
    "^",
    "%%",
    "%/%",
    "==",
    "!=",
    "<",
    ">",
    "<=",
    ">=",
    "&",
    "|",
    "!",
];

const SPECIAL_NAMES: &[&str] = &[
    ".Last.lib",
    ".First",
    ".Last",
    ".onAttach",
    ".onDetach",
    ".onLoad",
    ".onUnload",
];

/// Version added: 0.6.0
///
/// ## What it does
///
/// Checks the names of objects assigned with a standard R assignment operator,
/// simple function formal parameters, and literal names passed to `assign()`
/// or `setGeneric()`.
/// By default, names must use `snake_case` or consist only of symbols, matching
/// the default styles of lintr::object_name_linter().
///
/// The rule is opt-in because naming conventions are necessarily project
/// specific. Enable it with select = ["object_name"] or
/// extend-select = ["object_name"].
///
/// ## Why is this bad?
///
/// Consistent names make code easier to search and read. A single naming
/// convention also prevents readers from having to remember whether a project
/// uses, for example, `snake_case`, `camelCase`, or `CamelCase`.
///
/// ## Scope
///
/// This first implementation checks simple identifier, quoted-name, and
/// backtick-name assignment targets using `<-`, `=`, `<<-`, `->`, and `->>`,
/// along with simple function formal parameters and root identifiers in `$`
/// and `@` assignments. For extraction assignments, only the root object is
/// checked; member and slot names are not. Literal first arguments to
/// `assign()` and `setGeneric()` are also checked; dynamic expressions remain
/// out of scope. It does not check loop variables, subsetting assignments such
/// as `x[i] <-`, rlang/data.table :=, or named arguments in other function
/// calls. The formal parameters `...` and `..1`-style names are ignored.
/// Common S3 methods, package NAMESPACE S3 methods, and known S3 generics from
/// installed package metadata are exempted when that metadata is available.
/// A function definition containing UseMethod() is treated as declaring the
/// generic named by its assignment target; setGeneric() declarations are also
/// recognized when their name is static.
///
/// ## Configuration
///
/// The built-in styles are `symbols`, `CamelCase`, `camelCase`, `snake_case`,
/// `SNAKE_CASE`, `dotted.case`, `lowercase`, and `UPPERCASE`. Named custom
/// regular expressions can be supplied through `regexes`; if `styles` is
/// omitted, a non-empty `regexes` table replaces the default styles.
///
/// ```toml
/// [lint.object_name]
/// styles = ["snake_case"]
/// regexes = { "ends-in-id" = "_id$" }
/// ```
///
/// ## Examples
///
/// ```r
/// badName <- 1
/// good_name <- 2
/// ```
///
/// Use instead:
///
/// ```r
/// bad_name <- 1
/// good_name <- 2
/// ```
pub fn object_name(
    syntax: &RSyntaxNode,
    options: &ResolvedObjectNameOptions,
    namespace_s3_generics: &HashSet<String>,
) -> Vec<Diagnostic> {
    let local_s3_generics = collect_declared_s3_generics(syntax);
    let mut diagnostics = Vec::new();

    for node in syntax.descendants() {
        if let Some(parameter) = RParameter::cast(node.clone()) {
            let Some(AnyRParameterName::RIdentifier(identifier)) = parameter.name().ok() else {
                continue;
            };
            let name = identifier.name_text();
            if let Some(diagnostic) = name_diagnostic(
                &name,
                identifier.syntax().text_trimmed_range(),
                options,
                namespace_s3_generics,
                &local_s3_generics,
            ) {
                diagnostics.push(diagnostic);
            }
            continue;
        }

        if let Some(call) = RCall::cast(node.clone()) {
            if let Some((name, range)) = static_call_name_target(&call)
                && let Some(diagnostic) = name_diagnostic(
                    &name,
                    range,
                    options,
                    namespace_s3_generics,
                    &local_s3_generics,
                )
            {
                diagnostics.push(diagnostic);
            }
            continue;
        }

        let Some(binary) = RBinaryExpression::cast(node) else {
            continue;
        };
        let Ok(operator) = binary.operator() else {
            continue;
        };
        let target = match operator.kind() {
            RSyntaxKind::ASSIGN | RSyntaxKind::EQUAL | RSyntaxKind::SUPER_ASSIGN => {
                binary.left().ok()
            }
            RSyntaxKind::ASSIGN_RIGHT | RSyntaxKind::SUPER_ASSIGN_RIGHT => binary.right().ok(),
            _ => None,
        };
        let Some(target) = target else {
            continue;
        };
        let Some((name, range)) = assignment_target(&target) else {
            continue;
        };

        if let Some(diagnostic) = name_diagnostic(
            &name,
            range,
            options,
            namespace_s3_generics,
            &local_s3_generics,
        ) {
            diagnostics.push(diagnostic);
        }
    }

    diagnostics
}

fn name_diagnostic(
    name: &str,
    range: TextRange,
    options: &ResolvedObjectNameOptions,
    namespace_s3_generics: &HashSet<String>,
    local_s3_generics: &HashSet<String>,
) -> Option<Diagnostic> {
    let normalized = normalize_name(name);
    if normalized.is_empty()
        || is_special_name(&normalized)
        || is_s3_method(&normalized, namespace_s3_generics, local_s3_generics)
        || options.matches(&normalized)
    {
        return None;
    }

    let body = format!(
        "Variable and function name style should match {}.",
        options.expected()
    );
    Some(Diagnostic::new(
        ViolationData::new(Rule::ObjectName, body, None),
        range,
        Fix::empty(),
    ))
}

/// Remove R syntax that is not part of the object's semantic name.
///
/// This mirrors lintr's `strip_names()`: quoting/backticks and the `%` wrapper
/// around infix operators are syntax, and the trailing `<-` in an assignment
/// function name such as `` `names<-` `` is not part of the name to style.
fn normalize_name(name: &str) -> String {
    let mut normalized = name.trim_matches('`').to_string();
    if let Some(stripped) = normalized.strip_prefix('%') {
        normalized = stripped.to_string();
    }
    if let Some(stripped) = normalized.strip_suffix('%') {
        normalized = stripped.to_string();
    }
    if let Some(stripped) = normalized.strip_suffix("<-") {
        normalized = stripped.to_string();
    }
    normalized.trim_matches('`').to_string()
}

fn assignment_target(target: &AnyRExpression) -> Option<(String, TextRange)> {
    match target {
        AnyRExpression::RIdentifier(identifier) => {
            let name = identifier.name_text();
            (!name.is_empty()).then_some((name, target.syntax().text_trimmed_range()))
        }
        AnyRExpression::AnyRValue(AnyRValue::RStringValue(value)) => {
            let name = value.string_text()?;
            (!name.is_empty()).then_some((name, target.syntax().text_trimmed_range()))
        }
        AnyRExpression::RExtractExpression(extract) => {
            let operator = extract.operator().ok()?;
            if !matches!(operator.kind(), RSyntaxKind::DOLLAR | RSyntaxKind::AT) {
                return None;
            }

            let left = extract.left().ok()?;
            extraction_root_target(&left)
        }
        _ => None,
    }
}

fn extraction_root_target(target: &AnyRExpression) -> Option<(String, TextRange)> {
    match target {
        AnyRExpression::RIdentifier(identifier) => {
            let name = identifier.name_text();
            (!name.is_empty()).then_some((name, target.syntax().text_trimmed_range()))
        }
        AnyRExpression::RExtractExpression(extract) => {
            let operator = extract.operator().ok()?;
            if !matches!(operator.kind(), RSyntaxKind::DOLLAR | RSyntaxKind::AT) {
                return None;
            }

            let left = extract.left().ok()?;
            extraction_root_target(&left)
        }
        _ => None,
    }
}

fn is_special_name(name: &str) -> bool {
    name == "..." || SPECIAL_NAMES.contains(&name)
}

fn is_s3_method(
    name: &str,
    namespace_s3_generics: &HashSet<String>,
    local_generics: &HashSet<String>,
) -> bool {
    if name.starts_with('.') || !name.contains('.') {
        return false;
    }

    let parts: Vec<&str> = name.split('.').collect();
    if parts.len() < 2 {
        return false;
    }

    (1..parts.len()).any(|end| {
        let generic = parts[..end].join(".");
        S3_GENERICS.contains(&generic.as_str())
            || namespace_s3_generics.contains(&generic)
            || local_generics.contains(&generic)
    })
}

fn collect_declared_s3_generics(syntax: &RSyntaxNode) -> HashSet<String> {
    let mut generics = HashSet::new();

    for node in syntax.descendants() {
        let Some(binary) = RBinaryExpression::cast(node) else {
            continue;
        };
        let Ok(operator) = binary.operator() else {
            continue;
        };
        if !matches!(
            operator.kind(),
            RSyntaxKind::ASSIGN | RSyntaxKind::EQUAL | RSyntaxKind::SUPER_ASSIGN
        ) {
            continue;
        }

        let Ok(left) = binary.left() else {
            continue;
        };
        let AnyRExpression::RIdentifier(identifier) = left else {
            continue;
        };
        let Ok(right) = binary.right() else {
            continue;
        };
        let AnyRExpression::RFunctionDefinition(function) = right else {
            continue;
        };

        if function
            .syntax()
            .descendants()
            .filter_map(RCall::cast)
            .any(|call| {
                call.function()
                    .ok()
                    .is_some_and(|function| get_function_name(function) == "UseMethod")
            })
        {
            generics.insert(normalize_name(&identifier.name_text()));
        }
    }

    for node in syntax.descendants() {
        let Some(call) = RCall::cast(node) else {
            continue;
        };
        let Ok(function) = call.function() else {
            continue;
        };
        if get_function_name(function) != "setGeneric" {
            continue;
        }

        let Some(name) = call
            .arguments()
            .ok()
            .and_then(|arguments| arguments.items().iter().next().and_then(Result::ok))
            .and_then(|argument| argument.value())
            .and_then(|value| match value {
                AnyRExpression::AnyRValue(AnyRValue::RStringValue(string)) => string.string_text(),
                AnyRExpression::RIdentifier(identifier) => Some(identifier.name_text()),
                _ => None,
            })
        else {
            continue;
        };

        generics.insert(normalize_name(&name));
    }

    generics
}

fn static_call_name_target(call: &RCall) -> Option<(String, TextRange)> {
    let function_name = get_function_name(call.function().ok()?);
    let expected_name = match function_name.as_str() {
        "assign" => "x",
        "setGeneric" => "name",
        _ => return None,
    };

    let arguments = call.arguments().ok()?;
    let items = arguments.items();
    let argument = items.iter().filter_map(Result::ok).find(|argument| {
        argument_name(argument)
            .as_deref()
            .is_none_or(|name| name == expected_name)
    })?;

    literal_string_argument(&argument)
}

fn argument_name(argument: &RArgument) -> Option<String> {
    argument
        .name_clause()
        .and_then(|clause| match clause.name().ok()? {
            AnyRArgumentName::RIdentifier(identifier) => Some(identifier.name_text()),
            AnyRArgumentName::RStringValue(value) => value.string_text(),
            _ => None,
        })
}

fn literal_string_argument(argument: &RArgument) -> Option<(String, TextRange)> {
    let value = argument.value()?;
    let range = value.syntax().text_trimmed_range();
    let AnyRExpression::AnyRValue(AnyRValue::RStringValue(string)) = value else {
        return None;
    };
    let name = string.string_text()?;
    (!name.is_empty()).then_some((name, range))
}
