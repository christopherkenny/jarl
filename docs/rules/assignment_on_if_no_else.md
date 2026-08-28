# assignment_on_if_no_else
::: {.callout-note title="Added in 0.7.0" .low-opacity}
:::

## What it does

Flags an assignment whose value is an `if` expression without a final
`else` branch, when it overwrites a value that was assigned earlier and
never used.

## Why is this bad?

When no branch is taken, an `if` expression without a final `else`
evaluates to `NULL`. Assigning that result silently discards the previous
value, which is usually a bug: the earlier assignment reads like a fallback
but never acts as one.

```r
x <- 1
x <- if (foo) y
```

Here `x` is `NULL` when `foo` is `FALSE`, not `1`.

Writing `x <- if (foo) y` on its own is not reported: with nothing to
overwrite, `NULL` is usually the intended result.

## Example

```r
df <- data.frame()
df <- if (condition) {
  read_data()
}

value <- 0
value <- if (a) {
  1
} else if (b) {
  2
}
```

Use instead:

```r
df <- data.frame()
if (condition) {
  df <- read_data()
}

value <- if (a) {
  1
} else if (b) {
  2
} else {
  0
}
```

## Limitations

Super-assignment (`x <<- if (a) 1`, `if (a) 1 ->> x`) is not checked: it
binds in an enclosing environment, where whether the overwritten value was
used cannot be determined reliably.
