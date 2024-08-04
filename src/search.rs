use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::VALID_EXTENSIONS;

const BLOCKED_DIRS: [&str; 3] = ["bin", ".git", "obj"];

/// Searches recursively for project files in the given directory.
///
/// This function traverses the directory tree starting from the specified path,
/// looking for files with an extension contained in the const `VALID_EXTENSIONS`.
/// It skips the directories `bin`, `.git`, and `obj` for performance reasons.
///
/// # Arguments
///
/// * `path` - A reference to a path where the search should begin. It can be any type that implements `AsRef<Path>`.
///
/// # Returns
///
/// This function returns a `Result`:
/// * `Ok(Vec<PathBuf>)` - A vector of paths to found `.csproj` files if any are found.
/// * `Err(io::Error)` - An error if there is an issue reading the directory.
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
/// use dotnet_lens::search::search_projects;
///
/// let path = Path::new("/path/to/your/repository");
/// match search_projects(&path) {
///     Ok(files) => println!("Found csproj files: {:?}", files),
///     Err(e) => eprintln!("Error: {:?}", e),
/// }
/// ```
pub fn search_projects<P>(path: &P) -> Result<Vec<PathBuf>, io::Error>
where
    P: AsRef<Path>,
{
    let mut results = Vec::new();

    let path = path.as_ref();

    for entry in fs::read_dir(path)? {
        let entry = entry?;

        let file_type = entry.file_type()?;

        let entry_path = entry.path();

        if file_type.is_dir() && !BLOCKED_DIRS.iter().any(|dir| entry_path.ends_with(dir)) {
            results.append(&mut search_projects(&entry_path)?);

            continue;
        }

        if let Some(entry_extension) = entry_path.extension() {
            if VALID_EXTENSIONS.iter().any(|ext| *ext == entry_extension) {
                results.push(entry_path);
            }
        }
    }

    Ok(results)
}
