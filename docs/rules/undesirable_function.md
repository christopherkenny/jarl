# undesirable_function
::: {.callout-note title="Added in 0.5.0" .low-opacity}
:::

## What it does

Checks for calls to functions listed as undesirable.

## Why is this bad?

Some functions should not appear in production code. For example,
`browser()` is a debugging tool that interrupts execution, and should be
removed before committing.

## Configuration

By default, only `browser` is flagged. You can customise the list in
`jarl.toml`:

```toml
[lint.undesirable_function]
# Replace the default list entirely:
functions = ["browser", "debug"]

# Or add to the defaults, with optional suggestions:
extend-functions = [
  { setwd = 'Use here::here().' },
  "sprintf",
  { transmute = 'Use mutate(.keep = "none").' },
]
```

Use a string with just the function name for the default diagnostic. Use an
inline table to attach custom suggestion text to the default message.

## Example

```r
do_something <- function(abc = 1) {
   xyz <- abc + 1
   browser()      # flagged by default
   xyz
}
```
