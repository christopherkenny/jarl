pub(crate) mod assignment_on_if;

#[cfg(test)]
mod tests {
    use crate::utils_test::*;
    use insta::assert_snapshot;

    fn snapshot_lint(code: &str) -> String {
        format_diagnostics(code, "assignment_on_if", None)
    }

    #[test]
    fn test_lint_assignment_on_if() {
        assert_snapshot!(snapshot_lint("x <- if (condition) value"), @"
        warning: assignment_on_if
         --> <test>:1:1
          |
        1 | x <- if (condition) value
          | ------------------------- Avoid assigning the result of an `if()` expression.
          |
          = help: Use `ifelse()` for simple values, or move the assignment into each branch of the `if()` expression.
        Found 1 error.
        ");
        assert_snapshot!(snapshot_lint("x <- if (condition) { value } else { other }"), @"
        warning: assignment_on_if
         --> <test>:1:1
          |
        1 | x <- if (condition) { value } else { other }
          | -------------------------------------------- Avoid assigning the result of an `if()` expression.
          |
          = help: Use `ifelse()` for simple values, or move the assignment into each branch of the `if()` expression.
        Found 1 error.
        ");
        assert_snapshot!(snapshot_lint("fn(arg = if (x) y else z)"), @"
        warning: assignment_on_if
         --> <test>:1:4
          |
        1 | fn(arg = if (x) y else z)
          |    --------------------- Avoid assigning the result of an `if()` expression.
          |
          = help: Use `ifelse()` for simple values, or move the assignment into each branch of the `if()` expression.
        Found 1 error.
        ");
    }

    #[test]
    fn test_lint_assignment_on_if_with_other_assignments() {
        assert_snapshot!(snapshot_lint("x = if (condition) value else other"), @"
        warning: assignment_on_if
         --> <test>:1:1
          |
        1 | x = if (condition) value else other
          | ----------------------------------- Avoid assigning the result of an `if()` expression.
          |
          = help: Use `ifelse()` for simple values, or move the assignment into each branch of the `if()` expression.
        Found 1 error.
        ");
        assert_snapshot!(snapshot_lint("x <<- if (condition) value else other"), @"
        warning: assignment_on_if
         --> <test>:1:1
          |
        1 | x <<- if (condition) value else other
          | ------------------------------------- Avoid assigning the result of an `if()` expression.
          |
          = help: Use `ifelse()` for simple values, or move the assignment into each branch of the `if()` expression.
        Found 1 error.
        ");
        assert_snapshot!(snapshot_lint("(if (condition) value else other) -> x"), @"
        warning: assignment_on_if
         --> <test>:1:1
          |
        1 | (if (condition) value else other) -> x
          | -------------------------------------- Avoid assigning the result of an `if()` expression.
          |
          = help: Use `ifelse()` for simple values, or move the assignment into each branch of the `if()` expression.
        Found 1 error.
        ");
        assert_snapshot!(snapshot_lint("(if (condition) value else other) ->> x"), @"
        warning: assignment_on_if
         --> <test>:1:1
          |
        1 | (if (condition) value else other) ->> x
          | --------------------------------------- Avoid assigning the result of an `if()` expression.
          |
          = help: Use `ifelse()` for simple values, or move the assignment into each branch of the `if()` expression.
        Found 1 error.
        ");
    }

    #[test]
    fn test_no_lint_assignment_on_if() {
        expect_no_lint("if (condition) value else other", "assignment_on_if", None);
        expect_no_lint("x <- value", "assignment_on_if", None);
        expect_no_lint("fn(if (x) y else z)", "assignment_on_if", None);
        expect_no_lint("if (condition) x <- value", "assignment_on_if", None);
        expect_no_lint(
            "x <- { if (condition) value else other }",
            "assignment_on_if",
            None,
        );
        expect_no_lint(
            "if (condition) { arg <- value } else { arg <- other }\nfn(arg = arg)",
            "assignment_on_if",
            None,
        );
    }

    #[test]
    fn test_lint_assignment_on_if_with_parentheses() {
        assert_snapshot!(snapshot_lint("x <- (if (condition) value else other)"), @"
        warning: assignment_on_if
         --> <test>:1:1
          |
        1 | x <- (if (condition) value else other)
          | -------------------------------------- Avoid assigning the result of an `if()` expression.
          |
          = help: Use `ifelse()` for simple values, or move the assignment into each branch of the `if()` expression.
        Found 1 error.
        ");
        assert_snapshot!(snapshot_lint("fn(arg = (if (x) y else z))"), @"
        warning: assignment_on_if
         --> <test>:1:4
          |
        1 | fn(arg = (if (x) y else z))
          |    ----------------------- Avoid assigning the result of an `if()` expression.
          |
          = help: Use `ifelse()` for simple values, or move the assignment into each branch of the `if()` expression.
        Found 1 error.
        ");
    }
}
