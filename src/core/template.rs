//! Card templates (TECHSPEC 5.4): built-ins and template file parsing.

use crate::core::frontmatter::{self, FmRead};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    pub name: String,
    pub key: Option<char>,
    pub skeleton: String,
}

impl Template {
    /// The `## ` headings of the skeleton outside code fences.
    pub fn fields(&self) -> Vec<String> {
        let mut fenced = false;
        let mut out = Vec::new();
        for line in self.skeleton.lines() {
            if line.starts_with("```") {
                fenced = !fenced;
            } else if let Some(h) = line.strip_prefix("## ").filter(|_| !fenced) {
                out.push(h.trim().to_string());
            }
        }
        out
    }
}

pub fn parse_template(file_stem: &str, text: &str) -> Result<Template, String> {
    let (fm, skeleton) = match frontmatter::read(text) {
        FmRead::None => (None, text),
        FmRead::Valid(fm) => {
            let rest = &text[fm.end..];
            (Some(fm), rest)
        }
        FmRead::Invalid { problem, .. } => return Err(problem),
    };
    let get = |k| fm.as_ref().and_then(|f| f.get(k)).filter(|v| !v.is_empty());
    let name = get("name").map_or_else(
        || {
            let mut c = file_stem.chars();
            c.next()
                .map(|f| f.to_uppercase().chain(c).collect())
                .unwrap_or_default()
        },
        str::to_string,
    );
    let key = get("key").and_then(|k| {
        let mut c = k.chars();
        c.next().filter(|_| c.next().is_none())
    });
    Ok(Template {
        name,
        key,
        skeleton: skeleton.to_string(),
    })
}

pub fn builtins() -> Vec<Template> {
    [
        ("feature", include_str!("../../templates/feature.md")),
        ("bug", include_str!("../../templates/bug.md")),
        ("research", include_str!("../../templates/research.md")),
        ("chore", include_str!("../../templates/chore.md")),
    ]
    .into_iter()
    .map(|(stem, text)| parse_template(stem, text).expect("built-in template is valid"))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr11_builtins_are_feature_bug_research_chore() {
        let names: Vec<String> = builtins().into_iter().map(|t| t.name).collect();
        assert_eq!(names, ["Feature", "Bug", "Research", "Chore"]);
    }

    #[test]
    fn fr11_builtin_hotkeys_are_f_b_r_c() {
        let keys: Vec<Option<char>> = builtins().iter().map(|t| t.key).collect();
        assert_eq!(keys, [Some('f'), Some('b'), Some('r'), Some('c')]);
    }

    #[test]
    fn fr11_builtin_fields_follow_5_4() {
        let fields: Vec<Vec<String>> = builtins().iter().map(Template::fields).collect();
        assert_eq!(
            fields,
            [
                vec!["Goal", "Acceptance", "Context"],
                vec!["Symptom", "Expected", "Repro", "Context"],
                vec!["Question", "Done when", "Context"],
                vec!["Task", "Why", "Context"],
            ]
        );
    }

    #[test]
    fn fr13_skeleton_is_text_after_frontmatter() {
        let feature = &builtins()[0];
        assert!(feature.skeleton.starts_with("## Goal"));
        assert!(!feature.skeleton.contains("---"));
        assert!(!feature.skeleton.contains("name:"));
    }

    #[test]
    fn fr11_fields_skip_fenced_headings() {
        let t = parse_template("x", "## A\n```\n## B\n```\n").unwrap();
        assert_eq!(t.fields(), ["A"]);
    }
}
