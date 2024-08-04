dotnet-lens is a library for listing dependencies between .NET projects and packages.

This library provides functionality to parse .NET project files (`.csproj`, `.fsproj`, `.vbproj`)
and extract information about project dependencies, including project references and package references.

## Overview

The main components of this library include:
- `Project`: A struct representing a .NET project, including its language, name, path, target framework,
  project references, and package references.
- `ProjectLanguage`: An enum representing the language of the project based on the file extension.
- `ProjectReference`: A struct representing a reference to another project.
- `PackageReference`: A struct representing a reference to a NuGet package.

## Modules

- `parser`: A module for parsing .NET project files and extracting dependency information.
- `search`: A module for searching .NET project files in a directory.

## Features
- `serde`: Adds support for serde serialization and deserialization for the Project struct and
   adjacent types

## Examples

Here is a brief example demonstrating how to use the `Project` struct and its methods:

```rust
use dotnet_lens::{Project, search};

let projects_paths = search::search_projects(&"path/to/repository")?;

for path in projects_paths {
    let project = Project::new(path)?;

    for package_reference in project.package_references() {
        println!("{}: {}", package_reference.name(), package_reference.version());
    }
}
```
