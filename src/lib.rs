//! dotnet-lens is a library for listing dependencies between .NET projects and packages.
//!
//! This library provides functionality to parse .NET project files (`.csproj`, `.fsproj`, `.vbproj`)
//! and extract information about project dependencies, including project references and package references.
//!
//! ## Overview
//!
//! The main components of this library include:
//! - `Project`: A struct representing a .NET project, including its language, name, path, target framework,
//!   project references, and package references.
//! - `ProjectLanguage`: An enum representing the language of the project based on the file extension.
//! - `ProjectReference`: A struct representing a reference to another project.
//! - `PackageReference`: A struct representing a reference to a NuGet package.
//!
//! ## Modules
//!
//! - `parser`: A module for parsing .NET project files and extracting dependency information.
//! - `search`: A module for searching .NET project files in a directory.
//!
//! ## Features
//! - `serde`: Adds support for serde serialization and deserialization for the Project struct and
//!    adjacent types
//!
//! ## Examples
//!
//! Here is a brief example demonstrating how to use the `Project` struct and its methods:
//!
//! ```no_run
//! use dotnet_lens::{Project, search};
//!
//! let projects_paths = search::search_projects(&"path/to/repository")?;
//!
//! for path in projects_paths {
//!     let project = Project::new(path)?;
//!
//!     for package_reference in project.package_references() {
//!         println!("{}: {}", package_reference.name(), package_reference.version());
//!     }
//! }
//!
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::{
    ffi::OsStr,
    fs::File,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

use parser::ParseError;

pub mod parser;
pub mod search;

/// List of valid extensions: "csproj", "fsproj", "vbproj".
pub const VALID_EXTENSIONS: [&str; 3] = ["csproj", "fsproj", "vbproj"];

/// Represents a .NET project.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Project {
    name: String,
    language: ProjectLanguage,
    path: PathBuf,
    target_framework: Option<String>,
    project_references: Vec<ProjectReference>,
    package_references: Vec<PackageReference>,
}

impl Project {
    /// Creates a new `Project` instance by parsing a .NET project file.
    ///
    /// This function opens the file specified by the `path`, reads its contents, and parses it
    /// into a `Project` instance using the `parser::parse` function.
    ///
    /// # Arguments
    ///
    /// * `path` - A path to the .NET project file that implements the `AsRef<Path>` trait.
    ///
    /// # Returns
    ///
    /// A `Result<Self, ParseError>` where `Ok(Self)` contains the parsed `Project` instance, and
    /// `Err(ParseError)` contains the error if the file could not be read or parsed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The file could not be opened.
    /// - The file could not be parsed as a .NET project file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dotnet_lens::{Project, parser::ParseError};
    /// use std::path::Path;
    ///
    /// match Project::new(Path::new("path/to/MyProject.csproj")) {
    ///     Ok(project) => {
    ///         println!("Project parsed successfully: {:?}", project);
    ///     },
    ///     Err(e) => {
    ///         eprintln!("Failed to parse project: {:?}", e);
    ///     }
    /// }
    /// ```
    pub fn new<P>(path: P) -> Result<Self, ParseError>
    where
        P: AsRef<Path>,
    {
        let file_reader = File::open(path.as_ref())?;

        parser::parse(file_reader, path)
    }

    /// Returns the name of the project based on the file name of the provided path.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to a path that implements the `AsRef<Path>` trait.
    ///
    /// # Returns
    ///
    /// An `Option<String>` containing the project name if it could be extracted, otherwise `None`.
    pub fn get_project_name<P>(path: P) -> Option<String>
    where
        P: AsRef<Path>,
    {
        path.as_ref()
            .file_stem()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
    }

    /// Returns the name of the project.
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Returns the language of the project.
    pub fn language(&self) -> ProjectLanguage {
        self.language
    }

    /// Returns the path of the project.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Returns the target framework of the project, if any.
    pub fn target_framework(&self) -> Option<&String> {
        self.target_framework.as_ref()
    }

    /// Returns a reference to the list of project references.
    pub fn project_references(&self) -> &Vec<ProjectReference> {
        &self.project_references
    }

    /// Adds a new project reference to the list of project references.
    pub fn add_project_reference(&mut self, value: ProjectReference) {
        self.project_references.push(value);
    }

    /// Returns a reference to the list of package references.
    pub fn package_references(&self) -> &Vec<PackageReference> {
        &self.package_references
    }

    /// Adds a new package reference to the list of package references.
    pub fn add_package_reference(&mut self, value: PackageReference) {
        self.package_references.push(value);
    }
}

/// Represents the language of a .NET project based on the file extension.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ProjectLanguage {
    CSharp,
    FSharp,
    VB,
}

impl ProjectLanguage {
    /// Determines the project language from the file extension.
    ///
    /// The extension must be provided without a "." (ex: "csproj", "fsproj", "vbproj").
    ///
    /// # Arguments
    ///
    /// * `extension` - A reference to an `OsStr` representing the file extension.
    ///
    /// # Returns
    ///
    /// An `Option<ProjectLanguage>` containing the project language if it could be determined, otherwise `None`.
    pub fn from_extension(extension: &OsStr) -> Option<Self> {
        match extension.as_bytes() {
            b"csproj" => Some(Self::CSharp),
            b"fsproj" => Some(Self::FSharp),
            b"vbproj" => Some(Self::VB),
            _ => None,
        }
    }
}

/// Represents a reference to another .NET project.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProjectReference {
    name: String,
    path: PathBuf,
}

impl ProjectReference {
    /// Creates a new `ProjectReference` instance with the specified name and path.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the referenced project.
    /// * `path` - The path to the referenced project file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dotnet_lens::ProjectReference;
    /// use std::path::PathBuf;
    ///
    /// let project_ref = ProjectReference::new("OtherProject".to_string(), PathBuf::from("path/to/OtherProject.csproj"));
    /// println!("Project Name: {}", project_ref.name());
    /// println!("Project Path: {:?}", project_ref.path());
    /// ```
    pub fn new(name: String, path: PathBuf) -> Self {
        Self { name, path }
    }

    /// Returns the name of the referenced project.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dotnet_lens::ProjectReference;
    /// use std::path::PathBuf;
    ///
    /// let project_ref = ProjectReference::new("OtherProject".to_string(), PathBuf::from("path/to/OtherProject.csproj"));
    /// println!("Project Name: {}", project_ref.name());
    /// ```
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Returns the path to the referenced project file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dotnet_lens::ProjectReference;
    /// use std::path::PathBuf;
    ///
    /// let project_ref = ProjectReference::new("OtherProject".to_string(), PathBuf::from("path/to/OtherProject.csproj"));
    /// println!("Project Path: {:?}", project_ref.path());
    /// ```
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Represents a reference to a NuGet package.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PackageReference {
    name: String,
    version: String,
}

impl PackageReference {
    /// Creates a new `PackageReference` instance with the specified name and version.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the package.
    /// * `version` - The version of the package.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dotnet_lens::PackageReference;
    ///
    /// let package_ref = PackageReference::new("MyPackage".to_string(), "1.0.0".to_string());
    /// println!("Package Name: {}", package_ref.name());
    /// println!("Package Version: {}", package_ref.version());
    /// ```
    pub fn new(name: String, version: String) -> Self {
        Self { name, version }
    }

    /// Returns the name of the package.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dotnet_lens::PackageReference;
    ///
    /// let package_ref = PackageReference::new("MyPackage".to_string(), "1.0.0".to_string());
    /// println!("Package Name: {}", package_ref.name());
    /// ```
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Returns the version of the package.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dotnet_lens::PackageReference;
    ///
    /// let package_ref = PackageReference::new("MyPackage".to_string(), "1.0.0".to_string());
    /// println!("Package Version: {}", package_ref.version());
    /// ```
    pub fn version(&self) -> &String {
        &self.version
    }
}
