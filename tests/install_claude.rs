//! TECHSPEC 8.8 and 6.7 `install claude`: the embedded skills copied into Claude Code's
//! skills folder under a temp `HOME` (FR-67).

mod common;

use brain_swap::adapters::claude::{SKILLS, is_ours};
use common::cli::Sandbox;
use std::fs;
use std::path::{Path, PathBuf};

const RELOAD: &str = "run /reload-skills in sessions already open";

fn skills(s: &Sandbox) -> PathBuf {
    s.root().join("home/.claude/skills")
}

/// Runs `args` expecting exit 0: stdout, stderr.
fn run(s: &Sandbox, args: &[&str]) -> (String, String) {
    let out = s.cmd(args).assert().code(0).get_output().clone();
    let text = |b: Vec<u8>| String::from_utf8(b).unwrap();
    (text(out.stdout), text(out.stderr))
}

/// Every entry under `dir`, relative to it, sorted; symlinks are listed, not followed.
fn tree(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        out.push(p.strip_prefix(dir).unwrap().to_path_buf());
        if fs::symlink_metadata(&p).unwrap().is_dir() {
            out.extend(
                tree(&p)
                    .into_iter()
                    .map(|q| p.strip_prefix(dir).unwrap().join(q)),
            );
        }
    }
    out.sort();
    out
}

#[test]
fn fr67_install_writes_the_copies() {
    let s = Sandbox::new();
    run(&s, &["install", "claude"]);
    for skill in &SKILLS {
        let body = fs::read(skills(&s).join(skill.name).join("SKILL.md")).unwrap();
        assert_eq!(body, skill.body.as_bytes(), "{}", skill.name);
    }
    assert!(!s.root().join("data/brain-swap/pack").exists());
}

#[test]
fn fr67_install_writes_plain_folders() {
    let s = Sandbox::new();
    run(&s, &["install", "claude"]);
    for skill in &SKILLS {
        let dir = skills(&s).join(skill.name);
        assert!(fs::symlink_metadata(&dir).unwrap().file_type().is_dir());
        assert_eq!(tree(&dir), [PathBuf::from("SKILL.md")]);
        let md = fs::symlink_metadata(dir.join("SKILL.md")).unwrap();
        assert!(md.file_type().is_file());
    }
}

#[test]
fn ts8_8_install_prints_reload_hint_when_it_created_the_skills_dir() {
    let s = Sandbox::new();
    let (out, _) = run(&s, &["install", "claude"]);
    assert!(out.lines().any(|l| l == RELOAD), "{out}");
}

#[test]
fn ts8_8_install_omits_reload_hint_when_the_skills_dir_existed() {
    let s = Sandbox::new();
    fs::create_dir_all(skills(&s)).unwrap();
    let (out, _) = run(&s, &["install", "claude"]);
    assert!(!out.contains("run /reload-skills"), "{out}");
}

#[test]
fn ts8_8_install_suggests_the_allow_rule() {
    let s = Sandbox::new();
    fs::create_dir_all(skills(&s)).unwrap();
    let (out, _) = run(&s, &["install", "claude"]);
    assert!(
        out.lines()
            .any(|l| l.starts_with("tip: ") && l.contains("Bash(brain-swap:*)")),
        "{out}"
    );
}

#[test]
fn ts8_8_install_text_output() {
    let s = Sandbox::new();
    let (out, _) = run(&s, &["install", "claude"]);
    let root = s.root().display().to_string();
    insta::assert_snapshot!(out.replace(&root, "<sandbox>"));
}

#[test]
fn ts6_7_install_json_lists_linked_paths() {
    let s = Sandbox::new();
    let (out, err) = run(&s, &["--json", "install", "claude"]);
    let root = s.root().display().to_string();
    assert_eq!(out.lines().count(), 1, "{out}");
    insta::assert_snapshot!(out.replace(&root, "<sandbox>"));
    assert!(err.lines().all(|l| l.starts_with("warning: ")), "{err}");
    assert!(
        err.lines().any(|l| l == format!("warning: {RELOAD}")),
        "{err}"
    );
    assert!(err.contains("Bash(brain-swap:*)"), "{err}");
}

#[test]
fn fr67_install_never_edits_claude_settings() {
    let s = Sandbox::new();
    let claude = s.root().join("home/.claude");
    let files = [
        ("settings.json", "{\"permissions\":{\"allow\":[]}}\n"),
        ("settings.local.json", "{\"hooks\":{}}\n"),
        ("skills/other/SKILL.md", "---\nname: other\n---\nother\n"),
    ];
    for (path, text) in files {
        let p = claude.join(path);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }
    let before = tree(&claude);
    run(&s, &["install", "claude"]);
    for (path, text) in files {
        assert_eq!(
            fs::read_to_string(claude.join(path)).unwrap(),
            text,
            "{path}"
        );
    }
    let mut expected = before;
    for skill in &SKILLS {
        let dir = Path::new("skills").join(skill.name);
        expected.push(dir.join("SKILL.md"));
        expected.push(dir);
    }
    expected.sort();
    assert_eq!(tree(&claude), expected);
}

#[test]
fn ts8_8_install_leaves_no_temp_entries() {
    let s = Sandbox::new();
    run(&s, &["install", "claude"]);
    let names = tree(&skills(&s));
    assert!(
        names
            .iter()
            .all(|p| !p.to_string_lossy().contains(".bs-tmp-")),
        "{names:?}"
    );
}

/// `body` with its `metadata` version set to `0.0.9`.
fn older(body: &str) -> String {
    let lines = body
        .lines()
        .map(|l| match l.trim_start().starts_with("version:") {
            true => "  version: 0.0.9",
            false => l,
        });
    lines.map(|l| format!("{l}\n")).collect()
}

#[test]
fn ts8_8_is_ours_accepts_our_frontmatter() {
    for skill in &SKILLS {
        assert!(is_ours(skill.body), "{}", skill.name);
        let old = older(skill.body);
        assert!(old.contains("version: 0.0.9"), "{}", skill.name);
        assert!(is_ours(&old), "{}", skill.name);
    }
}

#[test]
fn ts8_8_is_ours_rejects_other_text() {
    let texts = [
        "mine",
        "---\nname: bs-card\npack: brain-swap\n---\nbody\n",
        "---\nname: bs-card\nmetadata:\n  pack: other\n---\nbody\n",
        "---\nname: bs-card\n---\nmetadata:\n  pack: brain-swap\n",
    ];
    for text in texts {
        assert!(!is_ours(text), "{text}");
    }
}

#[test]
fn fr67_reinstall_of_the_same_version_is_idempotent() {
    let s = Sandbox::new();
    run(&s, &["install", "claude"]);
    run(&s, &["install", "claude"]);
    for skill in &SKILLS {
        let body = fs::read(skills(&s).join(skill.name).join("SKILL.md")).unwrap();
        assert_eq!(body, skill.body.as_bytes(), "{}", skill.name);
    }
}

#[test]
fn fr67_upgrade_rewrites_our_older_copies() {
    let s = Sandbox::new();
    for skill in &SKILLS {
        let dir = skills(&s).join(skill.name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("SKILL.md"), older(skill.body)).unwrap();
    }
    run(&s, &["install", "claude"]);
    for skill in &SKILLS {
        let body = fs::read_to_string(skills(&s).join(skill.name).join("SKILL.md")).unwrap();
        assert_eq!(body, skill.body, "{}", skill.name);
    }
}

/// Runs `args` expecting exit 1: stdout, stderr.
fn fail(s: &Sandbox, args: &[&str]) -> (String, String) {
    let out = s.cmd(args).assert().code(1).get_output().clone();
    let text = |b: Vec<u8>| String::from_utf8(b).unwrap();
    (text(out.stdout), text(out.stderr))
}

/// A real folder `bs-park` holding a `SKILL.md` with the text `mine`; returns the folder.
fn foreign_park(s: &Sandbox) -> PathBuf {
    let dir = skills(s).join("bs-park");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("SKILL.md"), "mine").unwrap();
    dir
}

fn refusal(path: &Path) -> String {
    format!(
        "{} exists and was not created by brain-swap (use --force)",
        path.display()
    )
}

/// `<skills>/bs-back` as a symlink to `<sandbox>/elsewhere/bs-back`, a folder holding our copy.
fn foreign_link(s: &Sandbox) -> PathBuf {
    let target = s.root().join("elsewhere/bs-back");
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join("SKILL.md"), SKILLS[2].body).unwrap();
    let link = skills(s).join("bs-back");
    fs::create_dir_all(skills(s)).unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    link
}

fn assert_our_copy(s: &Sandbox, name: &str) {
    let dir = skills(s).join(name);
    assert!(
        fs::symlink_metadata(&dir).unwrap().file_type().is_dir(),
        "{name}"
    );
    let body = fs::read_to_string(dir.join("SKILL.md")).unwrap();
    let skill = SKILLS.iter().find(|k| k.name == name).unwrap();
    assert_eq!(body, skill.body, "{name}");
}

#[test]
fn fr67_foreign_directory_is_refused_without_force() {
    let s = Sandbox::new();
    let dir = foreign_park(&s);
    let (_, err) = fail(&s, &["install", "claude"]);
    assert_eq!(err, format!("brain-swap: {}\n", refusal(&dir)));
    assert_eq!(tree(&dir), [PathBuf::from("SKILL.md")]);
    assert_eq!(fs::read_to_string(dir.join("SKILL.md")).unwrap(), "mine");
}

#[test]
fn fr67_a_refusal_writes_nothing() {
    let s = Sandbox::new();
    foreign_park(&s);
    fail(&s, &["install", "claude"]);
    assert_eq!(
        tree(&skills(&s)),
        [Path::new("bs-park"), Path::new("bs-park/SKILL.md")]
    );
}

#[test]
fn fr67_foreign_file_and_symlinks_are_refused_without_force() {
    let s = Sandbox::new();
    let file = skills(&s).join("bs-card");
    fs::create_dir_all(skills(&s)).unwrap();
    fs::write(&file, "mine").unwrap();
    let (_, err) = fail(&s, &["install", "claude"]);
    assert_eq!(err, format!("brain-swap: {}\n", refusal(&file)));
    assert_eq!(fs::read_to_string(&file).unwrap(), "mine");
    assert_eq!(tree(&skills(&s)), [PathBuf::from("bs-card")]);

    let s = Sandbox::new();
    let link = foreign_link(&s);
    let target = s.root().join("elsewhere/bs-back");
    let (_, err) = fail(&s, &["install", "claude"]);
    assert_eq!(err, format!("brain-swap: {}\n", refusal(&link)));
    assert_eq!(fs::read_link(&link).unwrap(), target);
    assert_eq!(tree(&target), [PathBuf::from("SKILL.md")]);
    assert_eq!(
        fs::read_to_string(target.join("SKILL.md")).unwrap(),
        SKILLS[2].body
    );
    assert_eq!(tree(&skills(&s)), [PathBuf::from("bs-back")]);

    let s = Sandbox::new();
    let dangling = skills(&s).join("bs-link");
    fs::create_dir_all(skills(&s)).unwrap();
    std::os::unix::fs::symlink("/nowhere/bs-link", &dangling).unwrap();
    let (_, err) = fail(&s, &["install", "claude"]);
    assert_eq!(err, format!("brain-swap: {}\n", refusal(&dangling)));
    assert_eq!(
        fs::read_link(&dangling).unwrap(),
        Path::new("/nowhere/bs-link")
    );
    assert_eq!(tree(&skills(&s)), [PathBuf::from("bs-link")]);
}

#[test]
fn ts6_2_a_refusal_under_json_is_the_error_envelope() {
    let s = Sandbox::new();
    let dir = foreign_park(&s);
    let (out, _) = fail(&s, &["--json", "install", "claude"]);
    assert_eq!(out.lines().count(), 1, "{out}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let expected = serde_json::json!({
        "v": 1,
        "error": {"code": "invalid_input", "message": refusal(&dir)},
    });
    assert_eq!(v, expected);
}

#[test]
fn fr67_force_replaces_a_foreign_directory() {
    let s = Sandbox::new();
    let dir = foreign_park(&s);
    let (out, _) = run(&s, &["--json", "install", "claude", "--force"]);
    assert_our_copy(&s, "bs-park");
    assert_eq!(tree(&dir), [PathBuf::from("SKILL.md")]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["removed"], serde_json::json!([dir]));
}

#[test]
fn fr67_force_replaces_a_foreign_file_and_symlink() {
    let s = Sandbox::new();
    fs::create_dir_all(skills(&s)).unwrap();
    fs::write(skills(&s).join("bs-card"), "mine").unwrap();
    foreign_link(&s);
    run(&s, &["install", "claude", "--force"]);
    assert_our_copy(&s, "bs-card");
    assert_our_copy(&s, "bs-back");
    assert!(s.root().join("elsewhere/bs-back/SKILL.md").exists());
}

/// `<T>/repo` holding a copy of the repository's `pack/claude/skills/`, `bs-park` marked.
fn checkout(s: &Sandbox) -> PathBuf {
    let repo = s.root().join("repo");
    let from = Path::new(env!("CARGO_MANIFEST_DIR")).join("pack/claude/skills");
    for skill in &SKILLS {
        let dir = repo.join("pack/claude/skills").join(skill.name);
        fs::create_dir_all(&dir).unwrap();
        fs::copy(from.join(skill.name).join("SKILL.md"), dir.join("SKILL.md")).unwrap();
    }
    let park = repo.join("pack/claude/skills/bs-park/SKILL.md");
    let text = fs::read_to_string(&park).unwrap() + "checkout marker\n";
    fs::write(park, text).unwrap();
    repo
}

/// Each installed `SKILL.md` equals the checkout's file of that name.
fn assert_checkout_copies(s: &Sandbox, repo: &Path) {
    for skill in &SKILLS {
        let dir = skills(s).join(skill.name);
        assert!(fs::symlink_metadata(&dir).unwrap().file_type().is_dir());
        let want = fs::read(
            repo.join("pack/claude/skills")
                .join(skill.name)
                .join("SKILL.md"),
        );
        assert_eq!(
            fs::read(dir.join("SKILL.md")).unwrap(),
            want.unwrap(),
            "{}",
            skill.name
        );
    }
    let park = fs::read_to_string(skills(s).join("bs-park/SKILL.md")).unwrap();
    assert!(park.contains("checkout marker"), "{park}");
}

#[test]
fn fr67_link_copies_the_checkout_skills() {
    let s = Sandbox::new();
    let repo = checkout(&s);
    run(&s, &["install", "claude", "--link", repo.to_str().unwrap()]);
    assert_checkout_copies(&s, &repo);
}

#[test]
fn fr67_link_with_a_relative_repo_reads_the_checkout() {
    let s = Sandbox::new();
    let repo = checkout(&s);
    let mut cmd = s.cmd(&["install", "claude", "--link", "."]);
    cmd.current_dir(&repo).assert().code(0);
    let park = fs::read_to_string(skills(&s).join("bs-park/SKILL.md")).unwrap();
    assert!(park.contains("checkout marker"), "{park}");
}

#[test]
fn fr67_link_to_a_folder_without_the_skills_fails_and_changes_nothing() {
    let s = Sandbox::new();
    let repo = checkout(&s);
    let missing = repo.join("pack/claude/skills/bs-back/SKILL.md");
    fs::remove_file(&missing).unwrap();
    let args = ["install", "claude", "--link", repo.to_str().unwrap()];
    let out = s.cmd(&args).assert().code(1).get_output().clone();
    let err = String::from_utf8(out.stderr).unwrap();
    let canonical = fs::canonicalize(&repo)
        .unwrap()
        .join("pack/claude/skills/bs-back/SKILL.md");
    assert!(err.contains(&canonical.display().to_string()), "{err}");
    for skill in &SKILLS {
        assert!(!skills(&s).join(skill.name).exists(), "{}", skill.name);
    }
}

#[test]
fn fr67_link_refuses_a_foreign_entry_without_force() {
    let s = Sandbox::new();
    let repo = checkout(&s);
    let park = skills(&s).join("bs-park");
    fs::create_dir_all(&park).unwrap();
    fs::write(park.join("notes.txt"), "mine\n").unwrap();
    let before = tree(&park);
    s.cmd(&["install", "claude", "--link", repo.to_str().unwrap()])
        .assert()
        .code(1);
    assert_eq!(tree(&park), before);
    assert_eq!(
        fs::read_to_string(park.join("notes.txt")).unwrap(),
        "mine\n"
    );
}

#[test]
fn fr67_install_after_link_switches_to_the_embedded_skills() {
    let s = Sandbox::new();
    let repo = checkout(&s);
    run(&s, &["install", "claude", "--link", repo.to_str().unwrap()]);
    run(&s, &["install", "claude"]);
    for skill in &SKILLS {
        let body = fs::read_to_string(skills(&s).join(skill.name).join("SKILL.md")).unwrap();
        assert_eq!(body, skill.body, "{}", skill.name);
        assert!(!body.contains("checkout marker"), "{}", skill.name);
    }
}

#[test]
fn fr67_link_after_install_switches_to_the_checkout() {
    let s = Sandbox::new();
    let repo = checkout(&s);
    run(&s, &["install", "claude"]);
    run(&s, &["install", "claude", "--link", repo.to_str().unwrap()]);
    assert_checkout_copies(&s, &repo);
}

/// Every entry under `dir` with the bytes of each file (`None` for folders).
fn contents(dir: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    let read = |p: PathBuf| {
        let bytes = fs::read(dir.join(&p)).ok();
        (p, bytes)
    };
    tree(dir).into_iter().map(read).collect()
}

/// Asserts each skill folder is gone and `out` holds one `removed <folder>` line for each.
fn assert_removed(s: &Sandbox, out: &str) {
    for skill in &SKILLS {
        let dir = skills(s).join(skill.name);
        assert!(fs::symlink_metadata(&dir).is_err(), "{}", skill.name);
        let line = format!("removed {}", dir.display());
        assert!(out.lines().any(|l| l == line), "{out}");
    }
}

#[test]
fn fr67_remove_deletes_our_copies() {
    let s = Sandbox::new();
    run(&s, &["install", "claude"]);
    let (out, _) = run(&s, &["install", "claude", "--remove"]);
    assert_removed(&s, &out);
}

#[test]
fn fr67_remove_keeps_foreign_entries_and_settings() {
    let s = Sandbox::new();
    run(&s, &["install", "claude"]);
    let claude = s.root().join("home/.claude");
    fs::write(skills(&s).join("bs-card/SKILL.md"), "mine").unwrap();
    fs::create_dir_all(skills(&s).join("other")).unwrap();
    fs::write(skills(&s).join("other/SKILL.md"), "other\n").unwrap();
    fs::write(claude.join("settings.json"), "{}\n").unwrap();
    let card = contents(&skills(&s).join("bs-card"));
    let other = contents(&skills(&s).join("other"));
    let (out, _) = run(&s, &["install", "claude", "--remove"]);
    assert_eq!(contents(&skills(&s).join("bs-card")), card);
    assert_eq!(contents(&skills(&s).join("other")), other);
    assert_eq!(fs::read(claude.join("settings.json")).unwrap(), b"{}\n");
    assert!(!out.contains("bs-card"), "{out}");
}

#[test]
fn fr67_remove_deletes_checkout_copies_but_not_the_checkout() {
    let s = Sandbox::new();
    let repo = checkout(&s);
    run(&s, &["install", "claude", "--link", repo.to_str().unwrap()]);
    let before = contents(&repo);
    let (out, _) = run(&s, &["install", "claude", "--remove"]);
    assert_removed(&s, &out);
    assert_eq!(contents(&repo), before);
}

#[test]
fn fr67_remove_with_nothing_installed_exits_0() {
    let s = Sandbox::new();
    let (out, _) = run(&s, &["--json", "install", "claude", "--remove"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["removed"], serde_json::json!([]));
}

#[test]
fn ts6_7_remove_json_lists_removed_paths() {
    let s = Sandbox::new();
    run(&s, &["install", "claude"]);
    let (out, _) = run(&s, &["--json", "install", "claude", "--remove"]);
    let root = s.root().display().to_string();
    assert_eq!(out.lines().count(), 1, "{out}");
    insta::assert_snapshot!(out.replace(&root, "<sandbox>"));
}

#[test]
fn fr67_remove_ignores_force_and_link() {
    let s = Sandbox::new();
    run(&s, &["install", "claude"]);
    fs::create_dir_all(skills(&s).join("other")).unwrap();
    fs::write(skills(&s).join("other/SKILL.md"), "other\n").unwrap();
    let (out, _) = run(&s, &["install", "claude", "--remove", "--force"]);
    assert_removed(&s, &out);
    assert_eq!(
        fs::read(skills(&s).join("other/SKILL.md")).unwrap(),
        b"other\n"
    );

    let s = Sandbox::new();
    let repo = checkout(&s);
    run(&s, &["install", "claude"]);
    let before = contents(&repo);
    let args = [
        "install",
        "claude",
        "--remove",
        "--link",
        repo.to_str().unwrap(),
    ];
    let (out, _) = run(&s, &args);
    assert_removed(&s, &out);
    assert_eq!(contents(&repo), before);
}
