# object_name
::: {.callout-note title="Added in 0.6.0" .low-opacity}
:::

## What it does

This rule checks the names of variables and arguments introduced by assignments
and function definitions.
For `$` and `@` assignments, it checks only the base object name.

This rule is disabled by default.

The default styles are `snake_case` and `symbols`:

## Example

```r
badName <- 1
f <- function(badArg) {
  badArg
}
badName$member <- 1
```

Use instead:

```r
bad_name <- 1
f <- function(bad_arg) {
  bad_arg
}
bad_name$member <- 1
```

## Configuration

```toml
[lint.object_name]
styles = ["snake_case", "symbols"]
```

Built-in styles are `CamelCase`, `camelCase`, `snake_case`, `SNAKE_CASE`,
`dotted.case`, `lowercase`, `UPPERCASE`, and `symbols`.
Any combination of default styles can be included.

Additional acceptable names can be added via `regexes` when `styles` is set:

```toml
[lint.object_name.regexes]
prefixed = "^x_[a-z]+$"
```

If `styles` is omitted while `regexes` is supplied, the regular expressions are
used to define the accepted styles.
