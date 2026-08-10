# assignment_on_if
::: {.callout-note title="Added in 0.6.0" .low-opacity}
:::

## What it does

Checks for assignments whose value is an `if()` expression, including
assignments used as named function arguments.

## Why is this bad?

Assigning the result of an `if()` expression makes the assignment easy to
overlook and can make the two branches harder to compare. Assigning in
each branch makes the side effect explicit.

## Example

```r
x <- if (condition) value else other
fn(arg = if (condition) value else other)
```

For simple values, especially with a vectorized condition, use `ifelse()`:

```r
x <- ifelse(condition, value, other)
fn(arg = ifelse(condition, value, other))
```

For scalar conditions with side effects or more complex branch logic, move
the assignment into each branch:

```r
if (condition) {
  x <- value
} else {
  x <- other
}
```
