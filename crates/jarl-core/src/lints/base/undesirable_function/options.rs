use std::collections::{HashMap, HashSet};

use crate::rule_options::resolve_with_extend;

/// Default functions that are considered undesirable.
const DEFAULT_FUNCTIONS: &[&str] = &["browser"];

/// A function name, or a map from one or more function names to suggestions.
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum UndesirableFunctionEntry {
    Name(String),
    Message(HashMap<String, String>),
}

/// TOML options for `[lint.undesirable_function]`.
///
/// Use `functions` to fully replace the default list of undesirable functions.
/// Use `extend-functions` to add to the default list.
/// Entries can be strings or inline tables mapping a function to a custom
/// suggestion.
/// Specifying both is an error.
#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct UndesirableFunctionOptions {
    pub functions: Option<Vec<UndesirableFunctionEntry>>,
    pub extend_functions: Option<Vec<UndesirableFunctionEntry>>,
}

/// Resolved options for the `undesirable_function` rule, ready for use during
/// linting.
#[derive(Clone, Debug)]
pub struct ResolvedUndesirableFunctionOptions {
    pub functions: HashSet<String>,
    pub messages: HashMap<String, String>,
}

impl ResolvedUndesirableFunctionOptions {
    pub fn resolve(options: Option<&UndesirableFunctionOptions>) -> anyhow::Result<Self> {
        let (base, extend) = match options {
            Some(opts) => (opts.functions.as_ref(), opts.extend_functions.as_ref()),
            None => (None, None),
        };

        let base_names = base.map(|entries| entry_names(entries));
        let extend_names = extend.map(|entries| entry_names(entries));
        let functions = resolve_with_extend(
            base_names.as_ref(),
            extend_names.as_ref(),
            DEFAULT_FUNCTIONS,
            "undesirable_function",
            "functions",
        )?;
        let mut messages = HashMap::new();

        if let Some(entries) = base.or(extend) {
            add_messages(entries, &mut messages);
        }

        Ok(Self { functions, messages })
    }
}

fn entry_names(entries: &[UndesirableFunctionEntry]) -> Vec<String> {
    let mut names = Vec::new();

    for entry in entries {
        match entry {
            UndesirableFunctionEntry::Name(function) => names.push(function.clone()),
            UndesirableFunctionEntry::Message(entries) => names.extend(entries.keys().cloned()),
        }
    }

    names
}

fn add_messages(entries: &[UndesirableFunctionEntry], messages: &mut HashMap<String, String>) {
    for entry in entries {
        if let UndesirableFunctionEntry::Message(entries) = entry {
            messages.extend(entries.clone());
        }
    }
}
