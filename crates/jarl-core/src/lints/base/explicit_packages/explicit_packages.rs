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

#[cfg(test)]
mod tests {
    use super::explicit_packages;
    use crate::checker::Checker;
    use crate::declare_ns;
    use crate::rule_options::ResolvedRuleOptions;
    use crate::suppression::SuppressionManager;
    use crate::utils_test::{format_diagnostics_with_cache, get_fixed_text_with_cache};
    use air_r_parser::RParserOptions;
    use insta::assert_snapshot;
    use std::sync::Arc;

    declare_ns! {
        "stats" => ["filter"],
        "dplyr" => ["tibble", "filter", "mutate"],
        "tibble" => ["tibble"],
    }

    #[test]
    fn test_diagnostic() {
        assert_snapshot!(
            format_diagnostics_with_cache(
                "library(dplyr)\ntibble(x = 1)",
                "explicit_packages",
                None,
                &NS
            ),
            @r"
        warning: explicit_packages
         --> <test>:2:1
          |
        2 | tibble(x = 1)
          | ------ Use an explicit package qualifier for `tibble()`.
          |
          = help: Use `dplyr::tibble()`.
        Found 1 error.
        "
        );
    }

    #[test]
    fn test_fix() {
        assert_snapshot!(
            get_fixed_text_with_cache(
                vec!["library(dplyr)\ntibble(x = 1)"],
                "explicit_packages",
                &NS
            ),
            @r"
        OLD:
        ====
        library(dplyr)
        tibble(x = 1)
        NEW:
        ====
        library(dplyr)
        dplyr::tibble(x = 1)
        "
        );
    }

    #[test]
    fn test_fix_with_comments_inside_call() {
        assert_snapshot!(
            get_fixed_text_with_cache(
                vec![
                    "library(dplyr)\nmutate(\n  x,\n  # keep this comment\n  y = 1\n)"
                ],
                "explicit_packages",
                &NS
            ),
            @r"
        OLD:
        ====
        library(dplyr)
        mutate(
          x,
          # keep this comment
          y = 1
        )
        NEW:
        ====
        library(dplyr)
        dplyr::mutate(
          x,
          # keep this comment
          y = 1
        )
        "
        );
    }

    #[test]
    fn test_no_lint_for_local_binding_or_ambiguous_origin() {
        assert_eq!(
            format_diagnostics_with_cache(
                "library(dplyr)\ntibble <- function(x) x\ntibble(1)",
                "explicit_packages",
                None,
                &NS
            ),
            "All checks passed!"
        );

        assert_snapshot!(
            format_diagnostics_with_cache(
                "library(dplyr)\ntibble(1)\ntibble <- function(x) x",
                "explicit_packages",
                None,
                &NS
            ),
            @r"
        warning: explicit_packages
         --> <test>:2:1
          |
        2 | tibble(1)
          | ------ Use an explicit package qualifier for `tibble()`.
          |
          = help: Use `dplyr::tibble()`.
        Found 1 error.
        "
        );

        assert_snapshot!(
            format_diagnostics_with_cache(
                "library(dplyr)\nfilter(mtcars, mpg > 20)",
                "explicit_packages",
                None,
                &NS
            ),
            @r"
        warning: explicit_packages
         --> <test>:2:1
          |
        2 | filter(mtcars, mpg > 20)
          | ------ Cannot choose an explicit package qualifier for `filter()`.
          |
          = help: `filter()` is exported by multiple loaded packages: stats, dplyr.
        Found 1 error.
        "
        );
    }

    #[test]
    fn test_reexport_uses_provider_package() {
        let cache = Arc::new(
            crate::package_cache::PackageCache::from_exports_and_imports(
                &[("dplyr", &["tibble"]), ("tibble", &["tibble"])],
                &[("dplyr", "tibble", "tibble")],
            ),
        );

        assert_snapshot!(
            get_fixed_text_with_cache(
                vec!["library(dplyr)\nlibrary(tibble)\ntibble(x = 1)"],
                "explicit_packages",
                &cache
            ),
            @r"
        OLD:
        ====
        library(dplyr)
        library(tibble)
        tibble(x = 1)
        NEW:
        ====
        library(dplyr)
        library(tibble)
        tibble::tibble(x = 1)
        "
        );
    }

    #[test]
    fn test_no_lint_for_namespace_imports() {
        let parsed = air_r_parser::parse("tibble(x = 1)", RParserOptions::default());
        let mut checker = checker_for_direct_test(&parsed.syntax());
        checker.loaded_packages = vec!["dplyr".to_string()];
        checker.package_cache = Some(NS.clone());
        checker
            .import_from
            .insert("tibble".to_string(), "dplyr".to_string());

        assert!(!has_explicit_packages_lint(&parsed, &checker));

        let parsed = air_r_parser::parse("tibble(x = 1)", RParserOptions::default());
        let mut checker = checker_for_direct_test(&parsed.syntax());
        checker.loaded_packages = vec!["dplyr".to_string()];
        checker.package_cache = Some(NS.clone());
        checker.blanket_imports = vec!["dplyr".to_string()];

        assert!(!has_explicit_packages_lint(&parsed, &checker));
    }

    fn checker_for_direct_test(syntax: &air_r_syntax::RSyntaxNode) -> Checker {
        Checker::new(
            SuppressionManager::from_node(syntax, ""),
            Arc::new(ResolvedRuleOptions::default()),
        )
    }

    fn has_explicit_packages_lint(parsed: &air_r_parser::Parse, checker: &Checker) -> bool {
        let expr = parsed.tree().expressions().into_iter().next().unwrap();
        let call = expr.as_r_call().unwrap();
        explicit_packages(&call, checker).unwrap().is_some()
    }
}
