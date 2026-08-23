use std::collections::BTreeMap;

use regex::Regex;

const DEFAULT_STYLES: &[&str] = &["snake_case", "symbols"];

const STYLE_NAMES: &[&str] = &[
    "symbols",
    "CamelCase",
    "camelCase",
    "snake_case",
    "SNAKE_CASE",
    "dotted.case",
    "lowercase",
    "UPPERCASE",
];

/// TOML options for [lint.object_name].
///
/// styles accepts the built-in styles from lintr::object_name_linter().
/// regexes is a mapping from a label to a regular expression. If styles
/// is omitted while regexes is non-empty, the custom regexes replace the
/// default styles. If styles is present, custom regexes are added to it.
#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct ObjectNameOptions {
    pub styles: Option<Vec<String>>,
    pub regexes: Option<BTreeMap<String, String>>,
}

/// A compiled style or custom regular expression used by object_name.
#[derive(Clone, Debug)]
pub(crate) struct ObjectNamePattern {
    pub(crate) label: String,
    pub(crate) regex: Regex,
}

/// Resolved options for the object_name rule.
#[derive(Clone, Debug)]
pub struct ResolvedObjectNameOptions {
    pub(crate) patterns: Vec<ObjectNamePattern>,
}

impl ResolvedObjectNameOptions {
    pub fn resolve(options: Option<&ObjectNameOptions>) -> anyhow::Result<Self> {
        let custom_regexes = options
            .and_then(|opts| opts.regexes.as_ref())
            .filter(|regexes| !regexes.is_empty());

        let styles = match options.and_then(|opts| opts.styles.as_ref()) {
            Some(styles) => styles.clone(),
            None if custom_regexes.is_some() => Vec::new(),
            None => DEFAULT_STYLES
                .iter()
                .map(|style| (*style).to_string())
                .collect(),
        };

        if styles.is_empty() && custom_regexes.is_none() {
            return Err(anyhow::anyhow!(
                "At least one style or regex must be configured in [lint.object_name]."
            ));
        }

        let mut patterns =
            Vec::with_capacity(styles.len() + custom_regexes.map_or(0, |regexes| regexes.len()));

        for style in styles {
            let pattern = style_regex(&style).ok_or_else(|| {
                anyhow::anyhow!(
                    "Unknown style `{style}` in [lint.object_name]. Expected one of: {}.",
                    STYLE_NAMES.join(", ")
                )
            })?;
            patterns.push(compile_pattern(style, pattern, None)?);
        }

        if let Some(regexes) = custom_regexes {
            for (label, pattern) in regexes {
                let display_label = if label.is_empty() {
                    format!("/{pattern}/")
                } else {
                    label.clone()
                };
                patterns.push(compile_pattern(display_label, pattern, Some(label))?);
            }
        }

        Ok(Self { patterns })
    }

    pub(crate) fn matches(&self, name: &str) -> bool {
        self.patterns
            .iter()
            .any(|pattern| pattern.regex.is_match(name))
    }

    pub(crate) fn expected(&self) -> String {
        match self.patterns.as_slice() {
            [] => String::new(),
            [pattern] => format!("`{}`", pattern.label),
            patterns => {
                let labels: Vec<String> = patterns
                    .iter()
                    .map(|pattern| format!("`{}`", pattern.label))
                    .collect();
                let (last, rest) = labels.split_last().expect("patterns is non-empty");
                format!("{} or {}", rest.join(", "), last)
            }
        }
    }
}

fn compile_pattern(
    label: String,
    pattern: &str,
    original_label: Option<&String>,
) -> anyhow::Result<ObjectNamePattern> {
    let regex = Regex::new(pattern).map_err(|err| {
        let label = original_label.map_or_else(|| label.clone(), Clone::clone);
        anyhow::anyhow!("Invalid regex `{pattern}` for `{label}` in [lint.object_name]: {err}")
    })?;
    Ok(ObjectNamePattern { label, regex })
}

fn style_regex(style: &str) -> Option<&'static str> {
    match style {
        // A non-empty name made entirely of non-alphanumeric characters.
        "symbols" => Some(r"^[^[:alnum:]]+$"),
        "CamelCase" => Some(r"^[.]?[A-Z][A-Za-z0-9]*$"),
        "camelCase" => Some(r"^[.]?[a-z][A-Za-z0-9]*$"),
        "snake_case" => Some(r"^[.]?[a-z0-9][a-z0-9_]*$"),
        "SNAKE_CASE" => Some(r"^[.]?[A-Z0-9][A-Z0-9_]*$"),
        "dotted.case" => Some(r"^[.]?[a-z0-9]+(?:[.][a-z0-9]+)*$"),
        "lowercase" => Some(r"^[.]?[a-z0-9]+$"),
        "UPPERCASE" => Some(r"^[.]?[A-Z0-9]+$"),
        _ => None,
    }
}
