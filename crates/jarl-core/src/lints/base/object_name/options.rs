use std::collections::BTreeMap;

use regex::Regex;
use serde::Deserialize;

const DEFAULT_STYLES: &[&str] = &["snake_case", "symbols"];
const STYLE_NAMES: &str =
    "CamelCase, camelCase, snake_case, SNAKE_CASE, dotted.case, lowercase, UPPERCASE, or symbols";

/// TOML options for `[lint.object_name]`.
///
/// `styles` selects the built-in naming styles. `regexes` adds named regular
/// expressions that can be used alongside the selected styles.
#[derive(Clone, Debug, PartialEq, Eq, Default, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct ObjectNameOptions {
    pub styles: Option<Vec<String>>,
    pub regexes: Option<BTreeMap<String, String>>,
}

/// Resolved options for the `object_name` rule.
#[derive(Clone, Debug)]
pub struct ResolvedObjectNameOptions {
    patterns: Vec<ObjectNamePattern>,
}

#[derive(Clone, Debug)]
struct ObjectNamePattern {
    label: String,
    regex: Regex,
}

impl ResolvedObjectNameOptions {
    pub fn resolve(options: Option<&ObjectNameOptions>) -> anyhow::Result<Self> {
        let custom_regexes = options.and_then(|options| options.regexes.as_ref());
        let custom_only = custom_regexes.is_some_and(|regexes| !regexes.is_empty());

        let styles = match options.and_then(|options| options.styles.as_ref()) {
            Some(styles) => styles.clone(),
            None if custom_only => Vec::new(),
            None => DEFAULT_STYLES
                .iter()
                .map(|style| (*style).to_string())
                .collect(),
        };

        let mut patterns = Vec::new();
        for style in styles {
            let Some(pattern) = builtin_pattern(&style) else {
                return Err(anyhow::anyhow!(
                    "Invalid style for `[lint.object_name]`: \"{style}\". Expected {STYLE_NAMES}."
                ));
            };
            patterns.push(ObjectNamePattern {
                label: style,
                regex: Regex::new(pattern).expect("built-in object name pattern should compile"),
            });
        }

        if let Some(regexes) = custom_regexes {
            for (name, pattern) in regexes {
                let regex = Regex::new(pattern).map_err(|error| {
                    anyhow::anyhow!(
                        "Invalid regular expression for `{name}` in `[lint.object_name]`: {error}"
                    )
                })?;
                patterns.push(ObjectNamePattern { label: name.clone(), regex });
            }
        }

        if patterns.is_empty() {
            return Err(anyhow::anyhow!(
                "At least one style or regular expression must be specified for `[lint.object_name]`."
            ));
        }

        Ok(Self { patterns })
    }

    pub(crate) fn matches(&self, name: &str) -> bool {
        self.patterns
            .iter()
            .any(|pattern| pattern.regex.is_match(name))
    }

    pub(crate) fn expected(&self) -> String {
        self.patterns
            .iter()
            .map(|pattern| format!("`{}`", pattern.label))
            .collect::<Vec<_>>()
            .join(" or ")
    }
}

fn builtin_pattern(style: &str) -> Option<&'static str> {
    match style {
        "symbols" => Some(r"^[^[:alnum:]]+$"),
        "CamelCase" => Some(r"^[.]?[A-Z][A-Za-z0-9]*$"),
        "camelCase" => Some(r"^[.]?[a-z][A-Za-z0-9]*$"),
        "snake_case" => Some(r"^[.]?[a-z0-9][a-z0-9_]*$"),
        "SNAKE_CASE" => Some(r"^[.]?[A-Z0-9][A-Z0-9_]*$"),
        "dotted.case" => Some(r"^[.]?[a-z0-9]+(?:\.[a-z0-9]+)*$"),
        "lowercase" => Some(r"^[.]?[a-z0-9]+$"),
        "UPPERCASE" => Some(r"^[.]?[A-Z0-9]+$"),
        _ => None,
    }
}
