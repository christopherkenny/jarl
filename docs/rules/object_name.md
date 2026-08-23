# object_name
::: {.callout-note title="Added in 0.6.0" .low-opacity}
:::

## What it does

Checks the names of objects assigned with a standard R assignment operator,
simple function formal parameters, and literal names passed to `assign()`
or `setGeneric()`.
By default, names must use `snake_case` or consist only of symbols, matching
the default styles of lintr::object_name_linter().

The rule is opt-in because naming conventions are necessarily project
specific. Enable it with select = ["object_name"] or
extend-select = ["object_name"].

## Why is this bad?

Consistent names make code easier to search and read. A single naming
convention also prevents readers from having to remember whether a project
uses, for example, `snake_case`, `camelCase`, or `CamelCase`.

## Scope

This first implementation checks simple identifier, quoted-name, and
backtick-name assignment targets using `<-`, `=`, `<<-`, `->`, and `->>`,
along with simple function formal parameters and root identifiers in `$`
and `@` assignments. For extraction assignments, only the root object is
checked; member and slot names are not. It does not check loop variables,
subsetting assignments such as `x[i] <-`, dynamic expressions passed to
assign() or setGeneric(), rlang/data.table :=, or named arguments in other
function calls. The formal parameters `...` and `..1`-style names are
ignored. Common S3 methods and package NAMESPACE S3 methods are exempted;
known S3 generics from installed package metadata are also exempted when that
metadata is available. A function definition containing `UseMethod()` is
treated as declaring the generic named by its assignment target, and static
`setGeneric()` declarations are recognized as well.

## Configuration

The built-in styles are `symbols`, `CamelCase`, `camelCase`, `snake_case`,
`SNAKE_CASE`, `dotted.case`, `lowercase`, and `UPPERCASE`. Named custom
regular expressions can be supplied through `regexes`; if `styles` is
omitted, a non-empty `regexes` table replaces the default styles.

```toml
[lint.object_name]
styles = ["snake_case"]
regexes = { "ends-in-id" = "_id$" }
```

## Examples

```r
badName <- 1
good_name <- 2
```

Use instead:

```r
bad_name <- 1
good_name <- 2
```
