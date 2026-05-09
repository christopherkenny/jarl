pub mod explicit_packages;

#[cfg(test)]
mod tests {
    use super::explicit_packages::explicit_packages;
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
    fn test_function_local_bindings_do_not_leak_to_top_level() {
        assert_snapshot!(
            format_diagnostics_with_cache(
                "library(dplyr)\nf <- function() {\n  tibble <- function(x) x\n  tibble(1)\n}\ntibble(1)",
                "explicit_packages",
                None,
                &NS
            ),
            @r"
        warning: explicit_packages
         --> <test>:6:1
          |
        6 | tibble(1)
          | ------ Use an explicit package qualifier for `tibble()`.
          |
          = help: Use `dplyr::tibble()`.
        Found 1 error.
        "
        );
    }

    #[test]
    fn test_no_lint_for_function_parameters() {
        assert_eq!(
            format_diagnostics_with_cache(
                "library(dplyr)\nf <- function(tibble, x = tibble(1)) tibble(1)",
                "explicit_packages",
                None,
                &NS
            ),
            "All checks passed!"
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
    fn test_reexport_uses_loaded_package_when_provider_is_not_loaded() {
        let cache = Arc::new(
            crate::package_cache::PackageCache::from_exports_and_imports(
                &[("dplyr", &["tibble"]), ("tibble", &["tibble"])],
                &[("dplyr", "tibble", "tibble")],
            ),
        );

        assert_snapshot!(
            get_fixed_text_with_cache(
                vec!["library(dplyr)\ntibble(x = 1)"],
                "explicit_packages",
                &cache
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
