# explicit_packages
::: {.callout-note title="Added in 0.6.0" .low-opacity}
:::

## What it does

Checks for calls to imported functions that can be made explicit with `::`.
This rule is disabled by default.

## Why is this bad?

Bare function calls can depend on package attachment order. Qualifying calls
with `pkg::fun()` makes it clearer where the function comes from.

This rule uses statically visible package context: `library()` / `require()`
calls in scripts, and `DESCRIPTION` / `NAMESPACE` metadata in R packages.
Calls to default R packages are left unchanged.

If a function is exported by more than one loaded package, the rule reports the
ambiguity without applying a fix. Re-exports are resolved to the provider package
when that provider is also part of the file's package context; otherwise, they
are resolved to the loaded package that re-exports the function.

## Example

```r
library(dplyr)
library(tibble)

tibble(x = 1)
mutate(x, y = 1)
```

Use instead:
```r
library(dplyr)
library(tibble)

tibble::tibble(x = 1)
dplyr::mutate(x, y = 1)
```
