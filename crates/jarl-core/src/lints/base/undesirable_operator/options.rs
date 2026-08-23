use std::collections::HashSet;

use crate::rule_options::resolve_with_extend;

/// Default operators that are considered undesirable.
const DEFAULT_OPERATORS: &[&str] = &["->>", ":::", "<<-"];

/// TOML options for `[lint.undesirable_operator]`.
///
/// Use `operators` to fully replace the default list of undesirable operators.
/// Use `extend-operators` to add to the default list.
/// Specifying both is an error.
/// Set `call-is-undesirable = false` to skip backtick-quoted calls to operators.
/// Banned operators are still reported when they are used in ordinary
/// expressions.
#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct UndesirableOperatorOptions {
    pub operators: Option<Vec<String>>,
    pub extend_operators: Option<Vec<String>>,
    pub call_is_undesirable: Option<bool>,
}

/// Resolved options for the `undesirable_operator` rule, ready for use during
/// linting.
#[derive(Clone, Debug)]
pub struct ResolvedUndesirableOperatorOptions {
    pub operators: HashSet<String>,
    pub call_is_undesirable: bool,
}

impl ResolvedUndesirableOperatorOptions {
    pub fn resolve(options: Option<&UndesirableOperatorOptions>) -> anyhow::Result<Self> {
        let (base, extend) = match options {
            Some(opts) => (opts.operators.as_ref(), opts.extend_operators.as_ref()),
            None => (None, None),
        };

        let operators = resolve_with_extend(
            base,
            extend,
            DEFAULT_OPERATORS,
            "undesirable_operator",
            "operators",
        )?;
        let call_is_undesirable = options
            .and_then(|opts| opts.call_is_undesirable)
            .unwrap_or(true);

        Ok(Self { operators, call_is_undesirable })
    }
}
