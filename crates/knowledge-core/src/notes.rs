use anyhow::{anyhow, bail, Result};
use camino::{Utf8Component, Utf8Path, Utf8PathBuf};
use std::fs;

pub struct NoteStore {
    root: Utf8PathBuf,
}

pub fn validate_note_relative_path(path: &str) -> Result<()> {
    if path.is_empty() {
        bail!("note path must not be empty");
    }

    if path.contains('\\') || path.split('/').any(str::is_empty) {
        bail!("note path must contain only non-empty slash-separated components");
    }

    let path = Utf8Path::new(path);
    let mut has_component = false;
    for component in path.components() {
        match component {
            Utf8Component::Normal(_) => has_component = true,
            Utf8Component::Prefix(_)
            | Utf8Component::RootDir
            | Utf8Component::CurDir
            | Utf8Component::ParentDir => {
                bail!("note path must be relative and contain only normal components");
            }
        }
    }

    if !has_component {
        bail!("note path must contain at least one component");
    }

    Ok(())
}

impl NoteStore {
    pub fn new(root: Utf8PathBuf) -> Self {
        Self { root }
    }

    pub fn write_note(&self, folder: &str, file_name: &str, body: &str) -> Result<Utf8PathBuf> {
        validate_note_relative_path(folder)?;
        validate_note_relative_path(file_name)?;

        let relative_path = Utf8Path::new(folder).join(file_name);
        validate_note_relative_path(relative_path.as_str())?;

        let dir = self.root.join(folder);
        fs::create_dir_all(dir.as_std_path())?;
        let path = self.root.join(relative_path);
        fs::write(path.as_std_path(), body)?;
        Ok(path)
    }

    pub fn read_note(&self, relative_path: &str) -> Result<String> {
        validate_note_relative_path(relative_path)?;

        Ok(fs::read_to_string(
            self.root.join(relative_path).as_std_path(),
        )?)
    }

    pub fn relative_path<'a>(&self, path: &'a Utf8Path) -> Result<&'a str> {
        let relative_path = path
            .strip_prefix(&self.root)
            .map_err(|_| anyhow!("note path is outside note root"))?;
        validate_note_relative_path(relative_path.as_str())?;
        Ok(relative_path.as_str())
    }
}
