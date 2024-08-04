use spex::{
    parsing::XmlReader,
    xml::{Element, XmlDocument},
};
use std::{
    io::{self, Read},
    path::{Path, PathBuf},
};
use thiserror::Error;

use crate::{PackageReference, Project, ProjectLanguage, ProjectReference};

/// Parses a .NET project file and extracts project information.
///
/// This function reads the provided .NET project file and extracts its name,
/// language, target framework, project references, and package references.
///
/// # Arguments
///
/// * `reader` - A reader that provides the content of the project file.
/// * `path` - The path to the project file. It can be any type that implements `AsRef<Path>`.
///
/// # Returns
///
/// This function returns a `Result`:
/// * `Ok(Project)` - A `Project` struct containing the parsed project information.
/// * `Err(ParseError)` - An error if the file cannot be parsed or if it is not a valid project file.
///
/// # Errors
///
/// This function returns a `ParseError` in the following cases:
/// * If the path is a directory.
/// * If the file is not a recognized project file type (.csproj, .fsproj, .vbproj).
/// * If the file does not have a name.
/// * If there is an error reading or deserializing the file.
///
/// # Examples
///
/// ```no_run
/// use dotnet_lens::parser::parse;
/// use std::fs::File;
/// use std::path::Path;
///
/// let path = Path::new("path/to/project.csproj");
/// let file = File::open(path).unwrap();
/// let project = parse(file, path).unwrap();
/// println!("Parsed project: {:?}", project);
/// ```
pub fn parse<R, P>(reader: R, path: P) -> Result<Project, ParseError>
where
    R: Read,
    P: AsRef<Path>,
{
    let path = path.as_ref();
    if path.is_dir() {
        return Err(ParseError::PathIsNotAFile);
    }

    let language = path.extension().and_then(ProjectLanguage::from_extension);
    if language.is_none() {
        return Err(ParseError::FileIsNotAProject);
    }

    let name = Project::get_project_name(path).ok_or(ParseError::FileDoesNotHaveAName)?;

    let mut project = Project {
        name,
        language: language.unwrap(),
        path: path.to_owned(),
        target_framework: None,
        project_references: vec![],
        package_references: vec![],
    };

    fill_project_based_on_xml(&mut project, XmlReader::parse_auto(reader)?)?;

    Ok(project)
}

fn fill_project_based_on_xml(
    project: &mut Project,
    document: XmlDocument,
) -> Result<(), ParseError> {
    for element in document.root().elements() {
        match element.name().local_part() {
            "PropertyGroup" => handle_property_group(project, element)?,
            "ItemGroup" => handle_item_group(project, element)?,
            _ => (),
        }
    }

    Ok(())
}

fn handle_property_group(project: &mut Project, element: &Element) -> Result<(), ParseError> {
    // currently, the target framework is the only information that we look in the
    // PropertyGroup tag
    if project.target_framework.is_some() {
        return Ok(());
    }

    project.target_framework = element
        .opt("TargetFramework")
        .text()?
        .map(|target| target.to_string());

    Ok(())
}

fn handle_item_group(project: &mut Project, element: &Element) -> Result<(), ParseError> {
    for item in element.elements() {
        match item.name().local_part() {
            "ProjectReference" => {
                let attr_content = item
                    .att_req("Include")
                    .map_err(|_| ParseError::DeserializationError)?
                    .replace("\\", "/");

                let path = PathBuf::from(attr_content);

                let name =
                    Project::get_project_name(&path).ok_or(ParseError::FileDoesNotHaveAName)?;

                project
                    .project_references
                    .push(ProjectReference::new(name, path));
            }
            "PackageReference" => {
                let name = item
                    .att_req("Include")
                    .map_err(|_| ParseError::DeserializationError)?
                    .to_string();

                let version = item
                    .att_req("Version")
                    .map_err(|_| ParseError::DeserializationError)?
                    .to_string();

                project
                    .package_references
                    .push(PackageReference::new(name, version));
            }
            _ => (),
        }
    }

    Ok(())
}

/// Represents errors that can occur during project file parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    /// An error occurred during deserialization.
    #[error("there was an error while deserializing the file")]
    DeserializationError,
    /// An I/O error occurred while reading the file.
    #[error("there was an error while reading the file")]
    IoError(#[from] io::Error),
    /// The provided path is a directory, not a file.
    #[error("the path is a directory")]
    PathIsNotAFile,
    /// The file is not a recognized project file type (.csproj, .fsproj, .vbproj).
    #[error("the file is not a project (.csproj, .fsproj, .vbproj)")]
    FileIsNotAProject,
    /// The file does not have a name.
    #[error("the file does not have a name")]
    FileDoesNotHaveAName,
}

impl From<spex::common::XmlError> for ParseError {
    fn from(_: spex::common::XmlError) -> Self {
        Self::DeserializationError
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use io::Cursor;

    use crate::PackageReference;

    use super::*;

    #[test]
    pub fn parse_valid_csproj() {
        // given
        let content = r#"
<Project Sdk="Microsoft.NET.Sdk">

  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net8.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
  </PropertyGroup>

  <PropertyGroup>
    <Nullable>enable</Nullable>
  </PropertyGroup>

  <ItemGroup>
    <PackageReference Include="Microsoft.Extensions.Configuration" Version="8.0.0" />
  </ItemGroup>

  <ItemGroup>
    <PackageReference Include="Microsoft.Extensions.Hosting" Version="8.0.0" />
  </ItemGroup>

  <ItemGroup>
    <ProjectReference Include="..\FsharpConsole\FsharpConsole.fsproj" />
  </ItemGroup>

</Project>
"#;

        let project_path: &Path = "./TestProject.csproj".as_ref();

        // when
        let parsed_project = parse(Cursor::new(content), project_path).unwrap();

        // then
        let expected_project = Project {
            name: "TestProject".to_string(),
            path: PathBuf::from(project_path),
            language: ProjectLanguage::CSharp,
            target_framework: Some("net8.0".to_string()),
            project_references: vec![ProjectReference {
                name: "FsharpConsole".to_string(),
                path: PathBuf::from("../FsharpConsole/FsharpConsole.fsproj"),
            }],
            package_references: vec![
                PackageReference {
                    name: "Microsoft.Extensions.Configuration".to_string(),
                    version: "8.0.0".to_string(),
                },
                PackageReference {
                    name: "Microsoft.Extensions.Hosting".to_string(),
                    version: "8.0.0".to_string(),
                },
            ],
        };

        assert_eq!(parsed_project, expected_project);
    }

    #[test]
    pub fn parse_valid_fsproj() {
        // given
        let content = r#"
<Project Sdk="Microsoft.NET.Sdk">

  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>

  <ItemGroup>
    <Compile Include="Program.fs" />
  </ItemGroup>

  <ItemGroup>
    <PackageReference Include="Microsoft.Extensions.Configuration" Version="8.0.0" />
  </ItemGroup>

  <ItemGroup>
    <ProjectReference Include="..\VbConsole\VbConsole.vbproj" />
  </ItemGroup>

</Project>
"#;

        let project_path: &Path = "./TestProject.fsproj".as_ref();

        // when
        let parsed_project = parse(Cursor::new(content), project_path).unwrap();

        // then
        let expected_project = Project {
            name: "TestProject".to_string(),
            path: PathBuf::from(project_path),
            language: ProjectLanguage::FSharp,
            target_framework: Some("net8.0".to_string()),
            project_references: vec![ProjectReference {
                name: "VbConsole".to_string(),
                path: PathBuf::from("../VbConsole/VbConsole.vbproj"),
            }],
            package_references: vec![PackageReference {
                name: "Microsoft.Extensions.Configuration".to_string(),
                version: "8.0.0".to_string(),
            }],
        };

        assert_eq!(parsed_project, expected_project);
    }

    #[test]
    pub fn parse_valid_vbproj() {
        // given
        let content = r#"
<Project Sdk="Microsoft.NET.Sdk">

  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <RootNamespace>VbConsole</RootNamespace>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>

  <ItemGroup>
    <PackageReference Include="Microsoft.Extensions.Configuration" Version="8.0.0" />
  </ItemGroup>

  <ItemGroup>
    <ProjectReference Include="..\FsharpConsole\FsharpConsole.fsproj" />
  </ItemGroup>

</Project>
"#;

        let project_path: &Path = "./TestProject.vbproj".as_ref();

        // when
        let parsed_project = parse(Cursor::new(content), project_path).unwrap();

        // then
        let expected_project = Project {
            name: "TestProject".to_string(),
            path: PathBuf::from(project_path),
            language: ProjectLanguage::VB,
            target_framework: Some("net8.0".to_string()),
            project_references: vec![ProjectReference {
                name: "FsharpConsole".to_string(),
                path: PathBuf::from("../FsharpConsole/FsharpConsole.fsproj"),
            }],
            package_references: vec![PackageReference {
                name: "Microsoft.Extensions.Configuration".to_string(),
                version: "8.0.0".to_string(),
            }],
        };

        assert_eq!(parsed_project, expected_project);
    }

    #[test]
    pub fn invalid_xml() {
        // given
        let content = r#"
<Project Sdk="Microsoft.NET.Sdk">

  <PropertyGroup>
    <OutputType>Exe</OutputType>

  <ItemGroup>
    <PackageReference Include="Microsoft.Extensions.Configuration" Version="8.0.0" />
  </ItemGroup>

  <ItemGroup>
    <ProjectReference Include="..\FsharpConsole\FsharpConsole.fsproj" />
  </ItemGroup>

</Project>
"#;

        let project_path: &Path = "./TestProject.vbproj".as_ref();

        // when
        let parsed_project = parse(Cursor::new(content), project_path);

        // then
        if let Err(error) = parsed_project {
            assert!(matches!(error, ParseError::IoError(_)));

            return;
        }

        unreachable!()
    }

    #[test]
    pub fn missing_field() {
        // given
        let content = r#"
<Project Sdk="Microsoft.NET.Sdk">

  <ItemGroup>
    <PackageReference Include="Microsoft.Extensions.Configuration" />
  </ItemGroup>

  <ItemGroup>
    <ProjectReference Include="..\FsharpConsole\FsharpConsole.fsproj" />
  </ItemGroup>

</Project>
"#;

        let project_path: &Path = "./TestProject.vbproj".as_ref();

        // when
        let parsed_project = parse(Cursor::new(content), project_path);

        // then
        if let Err(error) = parsed_project {
            assert!(matches!(error, ParseError::DeserializationError));

            return;
        }

        unreachable!()
    }

    #[test]
    pub fn path_is_not_a_file() {
        // given
        let content = "";

        let project_path: &Path = "./".as_ref();

        // when
        let parsed_project = parse(Cursor::new(content), project_path);

        // then
        if let Err(error) = parsed_project {
            assert!(matches!(error, ParseError::PathIsNotAFile));

            return;
        }

        unreachable!()
    }

    #[test]
    pub fn path_is_not_a_project() {
        // given
        let content = "";

        let project_path: &Path = "./TestProject.txt".as_ref();

        // when
        let parsed_project = parse(Cursor::new(content), project_path);

        // then
        if let Err(error) = parsed_project {
            assert!(matches!(error, ParseError::FileIsNotAProject));

            return;
        }

        unreachable!()
    }

    #[test]
    pub fn path_does_not_have_a_name() {
        // given
        let content = "";

        let project_path: &Path = "./.csproj".as_ref();

        // when
        let parsed_project = parse(Cursor::new(content), project_path);

        // then
        if let Err(error) = parsed_project {
            assert!(matches!(error, ParseError::FileIsNotAProject));

            return;
        }

        unreachable!()
    }
}
