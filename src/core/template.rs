//! Card templates (TECHSPEC 5.4): built-ins and template file parsing.

use crate::core::error::{Error, Result};
use crate::core::frontmatter::{self, FmRead};
use std::path::Path;

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

pub fn parse_template(file_stem: &str, text: &str) -> std::result::Result<Template, String> {
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

/// The template named `name`, ignoring case.
pub fn find<'a>(templates: &'a [Template], name: &str) -> Result<&'a Template> {
    let lower = name.to_lowercase();
    templates
        .iter()
        .find(|t| t.name.to_lowercase() == lower)
        .ok_or_else(|| {
            let names: Vec<&str> = templates.iter().map(|t| t.name.as_str()).collect();
            Error::NotFound(format!("unknown template '{name}' ({})", names.join(", ")))
        })
}

/// Built-ins, then every `*.md` file in `dir` by file name; problems are warnings.
pub fn load(dir: &Path) -> (Vec<Template>, Vec<String>) {
    let mut ts = builtins();
    let mut warnings = Vec::new();
    let mut paths: Vec<_> = match std::fs::read_dir(dir) {
        Ok(rd) => rd
            .filter_map(|e| {
                e.map_err(|e| warnings.push(format!("templates {}: {e}", dir.display())))
                    .ok()
            })
            .map(|e| e.path())
            .collect(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => {
            warnings.push(format!("templates {}: {e}", dir.display()));
            Vec::new()
        }
    };
    paths.retain(|p| !p.is_dir() && p.extension().is_some_and(|x| x == "md"));
    paths.sort();
    for path in paths {
        let file = path.file_name().unwrap_or_default().to_string_lossy();
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let parsed = std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|text| Ok((parse_template(&stem, &text)?, text)));
        match parsed {
            Ok((mut t, text)) => {
                if t.key.is_none()
                    && matches!(frontmatter::read(&text),
                        FmRead::Valid(fm) if fm.get("key").is_some_and(|k| !k.is_empty()))
                {
                    warnings.push(format!("template {}: key must be one character", t.name));
                }
                if t.fields().iter().any(|f| f == "Timeline") {
                    t.skeleton = t
                        .skeleton
                        .lines()
                        .filter(|l| l.strip_prefix("## ").map(str::trim) != Some("Timeline"))
                        .map(|l| format!("{l}\n"))
                        .collect();
                    warnings.push(format!("template {}: ## Timeline dropped", t.name));
                }
                match ts
                    .iter_mut()
                    .find(|b| b.name.to_lowercase() == t.name.to_lowercase())
                {
                    Some(b) => *b = t,
                    None => ts.push(t),
                }
            }
            Err(problem) => warnings.push(format!("template {file}: {problem}")),
        }
    }
    let mut seen: Vec<(char, String)> = Vec::new();
    for t in &mut ts {
        let Some(k) = t.key else { continue };
        match seen.iter().find(|(c, _)| *c == k) {
            Some((_, other)) => {
                warnings.push(format!(
                    "template {}: hotkey '{k}' already used by {other}, dropped",
                    t.name
                ));
                t.key = None;
            }
            None => seen.push((k, t.name.clone())),
        }
    }
    (ts, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    fn dir_with(name: &str, bytes: &[u8]) -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join(name), bytes).unwrap();
        tmp
    }

    fn chmod(path: &Path, mode: u32) {
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
    }

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

    #[test]
    fn fr12_user_file_adds_template() {
        let tmp = dir_with(
            "spike.md",
            b"---\nname: Spike\nkey: s\n---\n## Hypothesis\n",
        );
        let (ts, warnings) = load(tmp.path());
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(ts.len(), 5);
        let last = &ts[4];
        assert_eq!(last.name, "Spike");
        assert_eq!(last.key, Some('s'));
        assert_eq!(last.fields(), ["Hypothesis"]);
    }

    #[test]
    fn fr12_user_file_replaces_builtin_ignoring_case() {
        let tmp = dir_with("my-feature.md", b"---\nname: feature\n---\n## Why\n");
        let (ts, _) = load(tmp.path());
        assert_eq!(ts.len(), 4);
        assert_eq!(ts[0].name, "feature");
        assert_eq!(ts[0].fields(), ["Why"]);
        assert_eq!(ts[0].key, None);
    }

    #[test]
    fn fr12_name_defaults_to_capitalised_stem() {
        let tmp = dir_with("spike.md", b"---\nkey: s\n---\n## Hypothesis\n");
        let (ts, _) = load(tmp.path());
        assert_eq!(ts[4].name, "Spike");
    }

    #[test]
    fn ts5_4_file_without_frontmatter_is_all_skeleton() {
        let tmp = dir_with("spike.md", b"## Hypothesis\n");
        let (ts, _) = load(tmp.path());
        assert_eq!(ts[4].skeleton, "## Hypothesis\n");
        assert_eq!(ts[4].key, None);
    }

    #[test]
    fn ts5_4_file_without_key_has_no_hotkey() {
        let tmp = dir_with("spike.md", b"---\nname: Spike\n---\n## Hypothesis\n");
        let (ts, _) = load(tmp.path());
        assert_eq!(ts[4].name, "Spike");
        assert_eq!(ts[4].key, None);
    }

    #[test]
    fn ts5_4_non_utf8_file_is_skipped_with_warning() {
        let tmp = dir_with("bad.md", b"## A\n\xff\n");
        let (ts, warnings) = load(tmp.path());
        assert_eq!(ts, builtins());
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].starts_with("template bad.md: "), "{warnings:?}");
    }

    #[test]
    fn ts5_4_invalid_frontmatter_file_is_skipped_with_warning() {
        let tmp = dir_with("bad.md", b"---\nname Spike\n---\n## A\n");
        let (ts, warnings) = load(tmp.path());
        assert_eq!(ts, builtins());
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].starts_with("template bad.md: "), "{warnings:?}");
    }

    #[test]
    fn ts5_4_unreadable_file_is_skipped_with_warning() {
        let tmp = dir_with("spike.md", b"---\nname: Spike\n---\n## A\n");
        let file = tmp.path().join("spike.md");
        chmod(&file, 0o000);
        if fs::read(&file).is_ok() {
            return; // running as root: mode 000 does not stop reads
        }
        let (ts, warnings) = load(tmp.path());
        assert_eq!(ts, builtins());
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(
            warnings[0].starts_with("template spike.md: "),
            "{warnings:?}"
        );
    }

    #[test]
    fn ts5_4_unreadable_folder_only_warns() {
        let tmp = dir_with("spike.md", b"## A\n");
        chmod(tmp.path(), 0o000);
        let root = fs::read_dir(tmp.path()).is_ok();
        let (ts, warnings) = load(tmp.path());
        chmod(tmp.path(), 0o755);
        if root {
            return; // running as root: mode 000 does not stop reads
        }
        assert_eq!(ts, builtins());
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].starts_with("templates "), "{warnings:?}");
    }

    #[test]
    fn ts5_4_missing_folder_gives_builtins_silently() {
        let tmp = tempfile::tempdir().unwrap();
        let (ts, warnings) = load(&tmp.path().join("missing"));
        assert_eq!(ts, builtins());
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[test]
    fn ts5_4_dangling_symlink_is_skipped_with_warning() {
        let tmp = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(tmp.path().join("gone"), tmp.path().join("x.md")).unwrap();
        let (ts, warnings) = load(tmp.path());
        assert_eq!(ts, builtins());
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].starts_with("template x.md: "), "{warnings:?}");
    }

    #[test]
    fn ts5_4_timeline_heading_in_template_is_dropped() {
        let tmp = dir_with("spike.md", b"## Hypothesis\n## Timeline\n");
        let (ts, warnings) = load(tmp.path());
        assert_eq!(ts[4].fields(), ["Hypothesis"]);
        assert_eq!(warnings, ["template Spike: ## Timeline dropped"]);
    }

    #[test]
    fn ts5_4_duplicate_hotkey_is_dropped_from_later_template() {
        let tmp = dir_with("spike.md", b"---\nkey: f\n---\n## Hypothesis\n");
        let (ts, warnings) = load(tmp.path());
        assert_eq!(ts[0].key, Some('f'));
        assert_eq!(ts[4].key, None);
        assert_eq!(
            warnings,
            ["template Spike: hotkey 'f' already used by Feature, dropped"]
        );
        let mut keys: Vec<char> = ts.iter().filter_map(|t| t.key).collect();
        let n = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), n, "{ts:?}");
    }

    #[test]
    fn ts5_4_multi_character_key_is_ignored_with_warning() {
        let tmp = dir_with("spike.md", b"---\nkey: ff\n---\n## Hypothesis\n");
        let (ts, warnings) = load(tmp.path());
        assert_eq!(ts[4].key, None);
        assert_eq!(warnings, ["template Spike: key must be one character"]);
    }

    #[test]
    fn ts6_2_find_template_ignores_case() {
        let ts = builtins();
        assert_eq!(find(&ts, "feature").unwrap().name, "Feature");
    }

    #[test]
    fn ts6_2_unknown_template_lists_names() {
        let ts = builtins();
        let e = find(&ts, "epic").unwrap_err();
        assert_eq!(
            e,
            Error::NotFound("unknown template 'epic' (Feature, Bug, Research, Chore)".into())
        );
        assert_eq!(e.exit_code(), 3);
    }
}
