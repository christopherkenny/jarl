# assignment_on_if
::: {.callout-note title="Added in 0.6.0" .low-opacity}
:::

## What it does

Flags assignments whose value is an `if` expression.
When enabled, it supersedes `assignment_on_if_no_else`.

## Why is this bad?

Assigning the result of an `if` expression can make the assignment easy to
overlook and can make the two branches harder to compare. Without a final
`else`, the assignment can also replace an existing value with `NULL`.

## Example

```r
x <- if (condition) {
  value <- f()
  g(value)
} else {
  other
}
```

Use instead:

```r
if (condition) {
  value <- f()
  x <- g(value)
} else {
  x <- other
}
```
