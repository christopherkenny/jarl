use crate::checker::{Checker, DEFAULT_PACKAGES, PackageOrigin};
use crate::diagnostic::*;
use crate::utils::{get_function_name, get_function_namespace_prefix};
use air_r_syntax::*;
use biome_rowan::AstNode;

/// Version added: unreleased
///
/// ## What it does
///
/// Checks for calls to functions that can be made explicit with `::`.
///
/// ## Why is this bad?
///
/// Bare function calls can depend on package attachment. Qualifying calls with
/// `pkg::fun()` makes it clearer where the function comes from.
pub fn explicit_packages(ast: &RCall, checker: &Checker) -> anyhow::Result<Option<Diagnostic>> {
    let function = ast.function()?;

    if get_function_namespace_prefix(function.clone()).is_some() {
        return Ok(None);
    }

    let fn_name = get_function_name(function.clone());
    if fn_name.is_empty() {
        return Ok(None);
    }

    let call_range = ast.syntax().text_trimmed_range();
    let function_range = function.syntax().text_trimmed_range();

    if checker.has_active_local_binding(&fn_name) {
        return Ok(None);
    }

    if is_shadowed_by_local_binding(&fn_name, call_range.start(), checker) {
        return Ok(None);
    }

    if checker.import_from.contains_key(&fn_name) {
        return Ok(None);
    }

    let pkg = match checker.resolve_package(&fn_name) {
        PackageOrigin::Resolved(pkg) => pkg,
        PackageOrigin::Ambiguous(candidates) => {
            return Ok(Some(Diagnostic::new(
                ViolationData::new(
                    "explicit_packages".to_string(),
                    format!("Cannot choose an explicit package qualifier for `{fn_name}()`."),
                    Some(format!(
                        "`{fn_name}()` is exported by multiple loaded packages: {}.",
                        candidates.join(", ")
                    )),
                ),
                function_range,
                Fix::empty(),
            )));
        }
        PackageOrigin::Unknown => return Ok(None),
    };

    if is_default_package(&pkg) || checker.blanket_imports.contains(&pkg) {
        return Ok(None);
    }

    let replacement = format!("{pkg}::{fn_name}");

    Ok(Some(Diagnostic::new(
        ViolationData::new(
            "explicit_packages".to_string(),
            format!("Use an explicit package qualifier for `{fn_name}()`."),
            Some(format!("Use `{pkg}::{fn_name}()`.")),
        ),
        function_range,
        Fix {
            content: replacement,
            start: function_range.start().into(),
            end: function_range.end().into(),
            to_skip: false,
        },
    )))
}

fn is_default_package(pkg: &str) -> bool {
    DEFAULT_PACKAGES.contains(&pkg)
}

fn is_shadowed_by_local_binding(
    fn_name: &str,
    call_start: biome_rowan::TextSize,
    checker: &Checker,
) -> bool {
    checker
        .local_bindings
        .get(fn_name)
        .is_some_and(|starts| starts.iter().any(|start| *start < call_start))
}
