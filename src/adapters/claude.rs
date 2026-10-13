//! The Claude Code pack embedded in the binary (TECHSPEC 8.8).

pub struct Skill {
    pub name: &'static str,
    pub body: &'static str,
}

macro_rules! skill {
    ($name:literal) => {
        Skill {
            name: $name,
            body: include_str!(concat!("../../pack/claude/skills/", $name, "/SKILL.md")),
        }
    };
}

pub const SKILLS: [Skill; 4] = [
    skill!("bs-card"),
    skill!("bs-park"),
    skill!("bs-back"),
    skill!("bs-link"),
];
