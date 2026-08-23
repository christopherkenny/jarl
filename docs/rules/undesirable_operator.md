# undesirable_operator
::: {.callout-note title="Added in 0.6.0" .low-opacity}
:::

## What it does

Checks for use of banned operators.

## Why is this bad?

Some operators should not appear in production code. For example, `:::`
accesses a package's internal functions, and `<<-` and `->>` assign outside
the current environment.

## Configuration

By default, only `->>`, `:::`, and `<<-` are flagged. You can customize the
list in `jarl.toml`:

To replace the default list entirely:

```toml
[lint.undesirable_operator]
operators = ["$", "@"]
```

To add to the defaults or skip backtick-quoted calls to operators:

```toml
[lint.undesirable_operator]
extend-operators = ["$", "%in%"]
call-is-undesirable = false
```

Specifying both `operators` and `extend-operators` is an error. Setting
`call-is-undesirable = false` still reports the banned operators when
they are used in ordinary expressions.

## Example

```r
package:::internal_function()  # flagged by default
value <<- 1                    # flagged by default
```
