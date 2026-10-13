//! Static reading of the pack's `SKILL.md` files (TECHSPEC 8, 12.1 Pack).

use std::collections::BTreeMap;

pub const SESSION_A: &str = "aaaaaaaa-1111-4111-8111-111111111111";
pub const SESSION_B: &str = "bbbbbbbb-2222-4222-8222-222222222222";
pub const SESSION_C: &str = "cccccccc-3333-4333-8333-333333333333";

/// A frontmatter value: a scalar, or the indented block under a key (`metadata`).
#[derive(Debug, PartialEq)]
pub enum Value {
    Text(String),
    Map(BTreeMap<String, String>),
}

/// `pack/claude/skills/<name>/SKILL.md` of this checkout.
pub fn read_skill(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("pack/claude/skills")
        .join(name)
        .join("SKILL.md");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn unquote(v: &str) -> String {
    let v = v.trim();
    for q in ['"', '\''] {
        if let Some(inner) = v.strip_prefix(q).and_then(|s| s.strip_suffix(q)) {
            return inner.to_string();
        }
    }
    v.to_string()
}

/// The leading `---` block as keys and values.
pub fn frontmatter(text: &str) -> BTreeMap<String, Value> {
    let mut out = BTreeMap::new();
    let mut last = String::new();
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some("---"), "no frontmatter");
    for line in lines.take_while(|l| *l != "---") {
        let (k, v) = line.split_once(':').expect("key: value");
        if line.starts_with([' ', '\t']) {
            let Value::Map(m) = out
                .entry(last.clone())
                .or_insert(Value::Map(BTreeMap::new()))
            else {
                panic!("{last}: both a value and a block");
            };
            m.insert(k.trim().to_string(), unquote(v));
        } else {
            last = k.to_string();
            if !v.trim().is_empty() {
                out.insert(last.clone(), Value::Text(unquote(v)));
            }
        }
    }
    out
}

/// The commands inside `` !`...` ``, in file order.
pub fn inline_commands(text: &str) -> Vec<String> {
    text.split("!`")
        .skip(1)
        .map(|s| s.split('`').next().unwrap().to_string())
        .collect()
}

/// Every fenced line and code span (inline commands included) starting with
/// `brain-swap `, in file order; a heredoc line is cut before `<<`.
pub fn commands(text: &str) -> Vec<String> {
    let mut fenced = false;
    let mut found = Vec::new();
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
        } else if fenced {
            found.push(line.trim());
        } else {
            found.extend(line.split('`').skip(1).step_by(2));
        }
    }
    found
        .into_iter()
        .filter(|c| c.starts_with("brain-swap "))
        .map(|c| c.split("<<").next().unwrap().trim().to_string())
        .collect()
}

/// `cmd` with the session variable set to `session` and the placeholders filled.
pub fn substitute(cmd: &str, session: &str) -> String {
    cmd.replace("${CLAUDE_SESSION_ID}", session)
        .replace("<ID>", "W-12")
        .replace("<column>", "Doing")
        .replace("<name>", "home")
        .replace("<template>", "Feature")
}

/// `cmd` split on blanks, single and double quotes grouping and removed.
pub fn shell_words(cmd: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word: Option<String> = None;
    let mut quote = None;
    for c in cmd.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), c) => word.get_or_insert_with(String::new).push(c),
            (None, '"' | '\'') => {
                quote = Some(c);
                word.get_or_insert_with(String::new);
            }
            (None, c) if c.is_whitespace() => words.extend(word.take()),
            (None, c) => word.get_or_insert_with(String::new).push(c),
        }
    }
    words.extend(word);
    words
}
