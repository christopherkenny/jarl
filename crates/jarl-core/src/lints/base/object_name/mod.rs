pub(crate) mod object_name;
pub(crate) mod options;

#[cfg(test)]
mod tests {
    use crate::lints::base::object_name::options::{ObjectNameOptions, ResolvedObjectNameOptions};
    use crate::rule_options::ResolvedRuleOptions;
    use crate::settings::{LinterSettings, Settings};
    use crate::utils_test::*;
    use insta::assert_snapshot;
    use std::collections::BTreeMap;

    fn snapshot_lint(code: &str) -> String {
        format_diagnostics(code, "object_name", None)
    }

    fn settings_with_options(options: ObjectNameOptions) -> Settings {
        Settings {
            linter: LinterSettings {
                rule_options: ResolvedRuleOptions {
                    object_name: ResolvedObjectNameOptions::resolve(Some(&options)).unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }

    #[test]
    fn test_lint_default_styles() {
        assert_snapshot!(
            snapshot_lint("badName <- 1\ngood_name <- 2\n`%>%` <- 3"),
            @r"
        warning: object_name
         --> <test>:1:1
          |
        1 | badName <- 1
          | ------- Variable and function name style should match `snake_case` or `symbols`.
          |
        Found 1 error.
        "
        );
    }

    #[test]
    fn test_lint_right_assignment_and_quoted_names() {
        assert_snapshot!(
            snapshot_lint("1 -> badName\n`badName` <- 1"),
            @r"
        warning: object_name
         --> <test>:1:6
          |
        1 | 1 -> badName
          |      ------- Variable and function name style should match `snake_case` or `symbols`.
          |
        warning: object_name
         --> <test>:2:1
          |
        2 | `badName` <- 1
          | --------- Variable and function name style should match `snake_case` or `symbols`.
          |
        Found 2 errors.
        "
        );
    }

    #[test]
    fn test_lint_custom_style() {
        let settings = settings_with_options(ObjectNameOptions {
            styles: Some(vec!["CamelCase".to_string()]),
            regexes: None,
        });

        assert_snapshot!(
            format_diagnostics_with_settings(
                "goodName <- 1\nBadName <- 2\nnot_good <- 3",
                "object_name",
                None,
                Some(settings),
            ),
            @r"
        warning: object_name
         --> <test>:1:1
          |
        1 | goodName <- 1
          | -------- Variable and function name style should match `CamelCase`.
          |
        warning: object_name
         --> <test>:3:1
          |
        3 | not_good <- 3
          | -------- Variable and function name style should match `CamelCase`.
          |
        Found 2 errors.
        "
        );
    }

    #[test]
    fn test_lint_named_regex() {
        let mut regexes = BTreeMap::new();
        regexes.insert("ends-in-id".to_string(), "_id$".to_string());
        let settings = settings_with_options(ObjectNameOptions {
            styles: Some(Vec::new()),
            regexes: Some(regexes),
        });

        assert_snapshot!(
            format_diagnostics_with_settings(
                "user_id <- 1\nuser_name <- 2",
                "object_name",
                None,
                Some(settings),
            ),
            @r"
        warning: object_name
         --> <test>:2:1
          |
        2 | user_name <- 2
          | --------- Variable and function name style should match `ends-in-id`.
          |
        Found 1 error.
        "
        );
    }

    #[test]
    fn test_no_lint_for_out_of_scope_assignments() {
        expect_no_lint("x[1] <- 1\nfoo := 1", "object_name", None);
        expect_no_lint(
            "for (badName in values) print(badName)",
            "object_name",
            None,
        );
        expect_no_lint("foo(badName = 1)", "object_name", None);
        expect_no_lint("print.foo <- function(x) x", "object_name", None);
    }

    #[test]
    fn test_no_lint_for_base_s3_methods_and_symbol_only_names() {
        for code in [
            "as.Date.foo <- function(x) x",
            "as.POSIXct.foo <- function(x) x",
            "`[<-.data.frame` <- function(x, i, value) x",
            "`$<-.data.frame` <- function(x, name, value) x",
            "`%%` <- function(e1, e2) e1",
        ] {
            assert_eq!(check_code(code, "object_name", None).len(), 0, "{code}");
        }
    }

    #[test]
    fn test_lints_literal_dynamic_name_calls() {
        for code in [
            "assign('badName', 1)",
            "assign(x = 'badName', value = 1)",
            "assign(value = 1, x = 'badName')",
            "setGeneric('badName')",
            "setGeneric(name = 'badName')",
        ] {
            assert_eq!(check_code(code, "object_name", None).len(), 1);
        }

        expect_no_lint("assign(name = 'badName', value = 1)", "object_name", None);
        expect_no_lint("assign(name_var, 1)", "object_name", None);
        expect_no_lint("setGeneric(name_var)", "object_name", None);
    }

    #[test]
    fn test_normalizes_r_name_syntax() {
        expect_no_lint(
            "`%foo%` <- 1\n`foo<-` <- function(x, value) x\n.First <- 1\n.Last <- 1",
            "object_name",
            None,
        );
    }

    #[test]
    fn test_use_method_uses_assigned_function_name() {
        let code = "foo <- function(x) UseMethod('bar')\nbar.BadClass <- function(x) x";
        let diagnostics = check_code(code, "object_name", None);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            get_diagnostic_highlight(code, "object_name", None),
            "bar.BadClass"
        );
    }

    #[test]
    fn test_lints_function_formals() {
        let code = "f <- function(badName, good_name, ...) badName";
        let diagnostics = check_code(code, "object_name", None);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            get_diagnostic_highlight(code, "object_name", None),
            "badName"
        );
    }

    #[test]
    fn test_lints_extraction_assignment_roots() {
        for code in [
            "badName$member <- 1",
            "badName@member <- 1",
            "badName$member$other <- 1",
            "1 -> badName$member",
            "1 -> badName@member",
        ] {
            assert_eq!(
                get_diagnostic_highlight(code, "object_name", None),
                "badName"
            );
        }

        expect_no_lint("good_name$badMember <- 1", "object_name", None);
    }

    #[test]
    fn test_suppression() {
        expect_no_lint(
            "# jarl-ignore object_name: this name is intentional\nbadName <- 1",
            "object_name",
            None,
        );
        expect_no_lint(
            "# jarl-ignore-file object_name: this file is intentional\nbadName <- 1",
            "object_name",
            None,
        );
    }
}
