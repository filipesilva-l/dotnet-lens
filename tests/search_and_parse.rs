extern crate dotnet_lens;

use std::path::PathBuf;

use dotnet_lens::{search, Project, ProjectLanguage, ProjectReference};

#[test]
fn search_and_parse_files() {
    let path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("tests/search_and_parse_data/");

    let projects_paths = search::search_projects(&path).unwrap();
    let expected_paths = vec![
        path.join("CsharpConsole/CsharpConsole.csproj"),
        path.join("VbConsole/VbConsole.vbproj"),
        path.join("FsharpConsole/FsharpConsole.fsproj"),
        path.join("CsharpClassLib/CsharpClassLib.csproj"),
    ];

    assert_eq!(projects_paths, expected_paths);

    let csharp_console = Project::new(&projects_paths[0]).unwrap();

    assert_eq!(csharp_console.name(), &"CsharpConsole".to_string());
    assert_eq!(csharp_console.language(), ProjectLanguage::CSharp);
    assert_eq!(csharp_console.path(), &expected_paths[0]);
    assert_eq!(
        csharp_console.target_framework(),
        Some("net8.0".to_string()).as_ref()
    );
    assert_eq!(
        csharp_console.project_references(),
        &vec![ProjectReference::new(
            "CsharpClassLib".into(),
            "../CsharpClassLib/CsharpClassLib.csproj".into()
        )]
    );
    assert_eq!(csharp_console.package_references(), &vec![]);
}
