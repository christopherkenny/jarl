pub(crate) mod assignment_on_if;

#[cfg(test)]
mod tests {
    use crate::rule_set::{Category, Rule};
    use crate::utils_test::*;
    use insta::assert_snapshot;

    fn snapshot_lint(code: &str) -> String {
        format_diagnostics(code, "assignment_on_if", None)
    }

    #[test]
    fn test_assignment_on_if_is_readability_and_disabled_by_default() {
        assert!(Rule::AssignmentOnIf.is_disabled_by_default());
        assert!(Rule::AssignmentOnIf.has_category(Category::Read));
    }

    #[test]
    fn test_lint_assignment_on_if() {
        assert_snapshot!(
            snapshot_lint("x <- if (condition) { value <- f(); value } else { other }"),
            @"
        warning: assignment_on_if
         --> <test>:1:1
          |
        1 | x <- if (condition) { value <- f(); value } else { other }
          | ---------------------------------------------------------- Avoid assigning the result of this `if` expression.
          |
          = help: Move the assignment into the `if` branches or extract the conditional into a helper function.
        Found 1 error.
        "
        );
    }

    #[test]
    fn test_assignment_on_if_flags_all_complex_branches() {
        for code in [
            "x <- if (a) { y <- 1; y } else 2",
            "x <- if (a) 1 else { y <- 2; y }",
            "x <- if (a) 1 else if (b) { y <- 2; y } else 3",
            "x <- if (a) 1 else if (b) 2 else ({ y <- 3; y })",
        ] {
            assert_eq!(
                check_code(code, "assignment_on_if", None).len(),
                1,
                "{code}"
            );
        }
    }

    #[test]
    fn test_assignment_on_if_flags_no_else_and_assignment_forms() {
        for code in [
            "x <- if (a) 1",
            "x <- if (a) 1 else 2",
            "x = if (a) 1",
            "x <<- if (a) 1",
            "(if (a) 1) -> x",
            "(if (a) 1) ->> x",
            "x[1] <- (((if (a) 1)))",
            "fn(x <- if (a) 1)",
            "x <- if (a) 1 else (if (b) 2)",
        ] {
            assert_eq!(
                check_code(code, "assignment_on_if", None).len(),
                1,
                "{code}"
            );
        }
    }

    #[test]
    fn test_no_lint_assignment_on_if() {
        for code in [
            "fn(arg = if (a) { y <- 1; y } else 2)",
            "x <- { if (a) { y <- 1; y } else 2 }",
            "if (a) x <- 1 else x <- 2",
        ] {
            expect_no_lint(code, "assignment_on_if", None);
        }
    }

    #[test]
    fn test_assignment_on_if_supersedes_no_else() {
        let diagnostics = check_code(
            "x <- if (a) 1",
            "assignment_on_if,assignment_on_if_no_else",
            None,
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message.name, "assignment_on_if");
    }
}
