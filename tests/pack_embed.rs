//! TECHSPEC 8.1, 8.8: the binary embeds exactly the four skill files of the pack.

mod common;

use brain_swap::adapters::claude::SKILLS;
use common::skill_md::read_skill;

#[test]
fn ts8_8_embedded_skills_equal_pack_files() {
    let names: Vec<_> = SKILLS.iter().map(|s| s.name).collect();
    assert_eq!(names, ["bs-card", "bs-park", "bs-back", "bs-link"]);
    for s in SKILLS {
        assert_eq!(s.body, read_skill(s.name), "{}", s.name);
    }
}

#[test]
fn ts8_1_pack_holds_exactly_the_four_embedded_skills() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("pack/claude/skills");
    let mut dirs: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    dirs.sort();
    let mut names: Vec<_> = SKILLS.iter().map(|s| s.name).collect();
    names.sort();
    assert_eq!(dirs, names);
    assert_eq!(names, ["bs-back", "bs-card", "bs-link", "bs-park"]);
    for d in &dirs {
        assert!(dir.join(d).join("SKILL.md").is_file(), "{d}");
    }
}
