use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use std::fs;

pub struct NoteStore {
    root: Utf8PathBuf,
}

impl NoteStore {
    pub fn new(root: Utf8PathBuf) -> Self {
        Self { root }
    }

    pub fn write_note(&self, folder: &str, file_name: &str, body: &str) -> Result<Utf8PathBuf> {
        let dir = self.root.join(folder);
        fs::create_dir_all(dir.as_std_path())?;
        let path = dir.join(file_name);
        fs::write(path.as_std_path(), body)?;
        Ok(path)
    }

    pub fn read_note(&self, relative_path: &str) -> Result<String> {
        Ok(fs::read_to_string(
            self.root.join(relative_path).as_std_path(),
        )?)
    }

    pub fn relative_path<'a>(&self, path: &'a Utf8Path) -> &'a str {
        path.strip_prefix(&self.root).unwrap().as_str()
    }
}
