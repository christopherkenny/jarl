pub(crate) mod assignment_on_if_no_else;

#[cfg(test)]
mod tests {
    use crate::rule_set::{Category, Rule};
    use crate::utils_test::*;
    use insta::assert_snapshot;

    fn snapshot_lint(code: &str) -> String {
        format_diagnostics(code, "assignment_on_if_no_else", None)
    }

    fn expect_no_lint_here(code: &str) {
        expect_no_lint(code, "assignment_on_if_no_else", None);
    }

    #[test]
    fn test_assignment_on_if_no_else_is_suspicious_and_enabled_by_default() {
        assert!(Rule::AssignmentOnIfNoElse.is_enabled_by_default());
        assert!(Rule::AssignmentOnIfNoElse.has_category(Category::Susp));
    }

    #[test]
    fn test_lint_assignment_on_if_no_else() {
        assert_snapshot!(
            snapshot_lint("df <- data.frame()\ndf <- if (cond) { read_data() }"),
            @"
        warning: assignment_on_if_no_else
         --> <test>:2:1
          |
        2 | df <- if (cond) { read_data() }
          | ------------------------------- Object `df` was assigned before and never used. This assignment overwrites it with `NULL` when no `if` branch is taken.
          |
          = help: Move the assignment into the `if` branches or add a final `else` branch.
        Found 1 error.
        "
        );
        assert_snapshot!(
            snapshot_lint("value <- 0\nvalue <- if (a) { 1 } else if (b) { 2 }"),
            @"
        warning: assignment_on_if_no_else
         --> <test>:2:1
          |
        2 | value <- if (a) { 1 } else if (b) { 2 }
          | --------------------------------------- Object `value` was assigned before and never used. This assignment overwrites it with `NULL` when no `if` branch is taken.
          |
          = help: Move the assignment into the `if` branches or add a final `else` branch.
        Found 1 error.
        "
        );
    }

    /// A prior binding of any kind counts, not only an assignment.
    #[test]
    fn test_assignment_on_if_no_else_reports_parameters_and_loop_variables() {
        assert_snapshot!(
            snapshot_lint("f <- function(x) {\n  x <- if (a) 1\n  x\n}"),
            @"
        warning: assignment_on_if_no_else
         --> <test>:2:3
          |
        2 |   x <- if (a) 1
          |   ------------- Object `x` was assigned before and never used. This assignment overwrites it with `NULL` when no `if` branch is taken.
          |
          = help: Move the assignment into the `if` branches or add a final `else` branch.
        Found 1 error.
        "
        );
        assert_snapshot!(
            snapshot_lint("for (i in 1:3) i <- if (a) 1"),
            @"
        warning: assignment_on_if_no_else
         --> <test>:1:16
          |
        1 | for (i in 1:3) i <- if (a) 1
          |                ------------- Object `i` was assigned before and never used. This assignment overwrites it with `NULL` when no `if` branch is taken.
          |
          = help: Move the assignment into the `if` branches or add a final `else` branch.
        Found 1 error.
        "
        );
    }

    /// Both branches bind `x` and neither value is read, so the overwrite
    /// discards them both.
    #[test]
    fn test_assignment_on_if_no_else_reports_when_every_prior_binding_is_unused() {
        assert_snapshot!(
            snapshot_lint("if (a) x <- 1 else x <- 2\nx <- if (foo) y"),
            @"
        warning: assignment_on_if_no_else
         --> <test>:2:1
          |
        2 | x <- if (foo) y
          | --------------- Object `x` was assigned before and never used. This assignment overwrites it with `NULL` when no `if` branch is taken.
          |
          = help: Move the assignment into the `if` branches or add a final `else` branch.
        Found 1 error.
        "
        );
    }

    #[test]
    fn test_assignment_on_if_no_else_supports_assignment_forms() {
        for code in [
            "value = 1\nvalue = if (a) 1",
            "1 -> value\n(if (a) 1) -> value",
            "value <- 1\nvalue <- (if (a) 1)",
        ] {
            assert_eq!(
                check_code(code, "assignment_on_if_no_else", None).len(),
                1,
                "Expected a diagnostic for {code}"
            );
        }
    }

    /// Without a prior binding there is nothing to overwrite: `NULL` is
    /// usually what the author wants.
    #[test]
    fn test_no_lint_without_a_prior_binding() {
        expect_no_lint_here("value <- if (a) { 1 }");
        expect_no_lint_here("value <- if (a) { 1 } else if (b) { 2 }");
        expect_no_lint_here("value <- 1");
        expect_no_lint_here("if (a) value <- 1");
        expect_no_lint_here("if (a) { value <- 1 }");
        expect_no_lint_here("fn(value = if (a) { 1 })");
    }

    #[test]
    fn test_no_lint_when_the_prior_binding_was_used() {
        expect_no_lint_here("x <- 1\nprint(x)\nx <- if (a) 2");
        expect_no_lint_here("x <- 1\nx <- if (a) x + 1");
        expect_no_lint_here("x <- 1\nf <- function() print(x)\nx <- if (a) 2");
        // Only one of the two prior bindings is read, but that is enough.
        expect_no_lint_here("x <- 1\nprint(x)\nx <- 2\nx <- if (a) 3");
    }

    #[test]
    fn test_no_lint_when_the_if_has_a_final_else() {
        expect_no_lint_here("value <- 0\nvalue <- if (a) { 1 } else { 2 }");
        expect_no_lint_here("value <- 0\nvalue <- if (a) { 1 } else if (b) { 2 } else { 3 }");
        expect_no_lint_here("value <- 0\nvalue <- if (a) { if (b) 1 } else { 2 }");
    }

    /// `<-` inside a function creates a fresh local, so the outer binding is
    /// not overwritten.
    #[test]
    fn test_no_lint_across_scopes() {
        expect_no_lint_here("x <- 1\nf <- function() {\n  x <- if (a) 2\n}");
    }

    /// `names(x) <-` reads the current value before replacing part of it.
    #[test]
    fn test_no_lint_on_complex_lhs() {
        expect_no_lint_here("x <- 1\nnames(x) <- if (a) 2");
        expect_no_lint_here("x <- 1\nx[1] <- if (a) 2");
    }

    /// Super-assignment binds in an enclosing environment, see the rule's
    /// documented limitations.
    #[test]
    fn test_no_lint_on_super_assignment() {
        expect_no_lint_here("x <- 1\nx <<- if (a) 2");
        expect_no_lint_here("x <- 1\n(if (a) 2) ->> x");
    }

    /// The rule inherits `SemanticInfo`'s notion of a use, so a read that oak
    /// alone doesn't record still counts.
    #[test]
    fn test_no_lint_on_semantic_uses() {
        expect_no_lint_here("library(glue)\nx <- 1\nglue(\"{x}\")\nx <- if (a) 2");
        expect_no_lint_here("x <- 1\ndo.call(\"x\", list())\nx <- if (a) 2");
    }

    /// Quoted code is captured, not executed, so it neither binds nor reads.
    #[test]
    fn test_no_lint_in_nse() {
        expect_no_lint_here("x <- 1\nprint(x)\nexpression(x <- if (a) 2)");
    }
}
