use anyhow::{anyhow, bail, Result};
use camino::{Utf8Component, Utf8Path, Utf8PathBuf};
use std::fs;

/// Filesystem-backed store for compact markdown notes.
pub struct NoteStore {
    root: Utf8PathBuf,
}

/// Validates that a note path is relative and only contains normal components.
///
/// # Arguments
///
/// * `path` - Slash-separated relative path.
///
/// # Returns
///
/// `Ok(())` when the path is a safe relative note path.
///
/// # Errors
///
/// Returns an error if the path is empty, has empty components, contains `\`,
/// or includes `.` / `..` / root / prefix components.
///
/// # Examples
///
/// ```
/// # use knowledge_core::notes::validate_note_relative_path;
/// validate_note_relative_path("lessons/slug.md")?;
/// assert!(validate_note_relative_path("../escape.md").is_err());
/// # Ok::<(), anyhow::Error>(())
/// ```
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
    /// Creates a note store rooted at `root`.
    ///
    /// # Arguments
    ///
    /// * `root` - Root directory that contains all note files.
    ///
    /// # Returns
    ///
    /// New `NoteStore` instance.
    pub fn new(root: Utf8PathBuf) -> Self {
        Self { root }
    }

    /// Writes a note under `folder/file_name`.
    ///
    /// # Arguments
    ///
    /// * `folder` - Relative folder under the note root.
    /// * `file_name` - Relative note file name.
    /// * `body` - Note content to write.
    ///
    /// # Returns
    ///
    /// Absolute UTF-8 path to the written note.
    ///
    /// # Errors
    ///
    /// Returns an error if path validation fails, directories cannot be
    /// created, or file writing fails.
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

    /// Reads a note by relative path.
    ///
    /// # Arguments
    ///
    /// * `relative_path` - Relative path within the note root.
    ///
    /// # Returns
    ///
    /// Full note contents.
    ///
    /// # Errors
    ///
    /// Returns an error if the path is invalid or the file cannot be read.
    pub fn read_note(&self, relative_path: &str) -> Result<String> {
        validate_note_relative_path(relative_path)?;

        Ok(fs::read_to_string(
            self.root.join(relative_path).as_std_path(),
        )?)
    }

    /// Converts an absolute path under the note root to a validated relative path.
    ///
    /// # Arguments
    ///
    /// * `path` - Absolute path that should reside within this note root.
    ///
    /// # Returns
    ///
    /// Relative note path slice.
    ///
    /// # Errors
    ///
    /// Returns an error if `path` is outside the root or fails relative-path
    /// validation.
    pub fn relative_path<'a>(&self, path: &'a Utf8Path) -> Result<&'a str> {
        let relative_path = path
            .strip_prefix(&self.root)
            .map_err(|_| anyhow!("note path is outside note root"))?;
        validate_note_relative_path(relative_path.as_str())?;
        Ok(relative_path.as_str())
    }
}
