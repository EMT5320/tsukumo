//! Skills directory + trait socket (placeholder only).
//!
//! No "领悟新技能" UI. Full skill self-precipitation is M2.

use std::path::{Path, PathBuf};

/// Minimal skill descriptor for the socket (not a full SKILL.md parser yet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skill {
    pub id: String,
    pub title: String,
}

/// Trait placeholder for future skill create / list / load.
///
/// A1 must not expose skill precipitation product surface.
pub trait SkillSocket {
    /// Directory that will hold SKILL.md files.
    fn skills_dir(&self) -> &Path;

    /// List installed skills (empty for the stub).
    fn list(&self) -> Vec<Skill>;

    /// Create / register a skill — stub returns `false` (not implemented).
    fn skill_create(&mut self, _skill: Skill) -> bool {
        false
    }
}

/// Empty skills socket backed by an on-disk `skills/` directory.
#[derive(Debug, Clone)]
pub struct SkillStub {
    dir: PathBuf,
}

impl SkillStub {
    pub fn open(skills_dir: impl AsRef<Path>) -> std::io::Result<Self> {
        let dir = skills_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    pub fn from_data_dir(data_dir: impl AsRef<Path>) -> std::io::Result<Self> {
        Self::open(data_dir.as_ref().join("skills"))
    }
}

impl SkillSocket for SkillStub {
    fn skills_dir(&self) -> &Path {
        &self.dir
    }

    fn list(&self) -> Vec<Skill> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn stub_lists_empty_and_refuses_create() {
        let dir = tempdir().unwrap();
        let mut stub = SkillStub::from_data_dir(dir.path()).unwrap();
        assert!(stub.skills_dir().is_dir());
        assert!(stub.list().is_empty());
        assert!(!stub.skill_create(Skill {
            id: "demo".into(),
            title: "Demo".into(),
        }));
    }
}
