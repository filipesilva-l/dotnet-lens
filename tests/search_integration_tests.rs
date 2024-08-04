use core::panic;
use std::fs;
use std::path::PathBuf;

use tempfile::tempdir;

extern crate dotnet_lens;

use dotnet_lens::search::search_projects;

#[test]
fn test_search_csproj_files_and_ignore() {
    // given
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::create_dir_all(dir_path.join("src")).unwrap();
    fs::create_dir_all(dir_path.join("bin")).unwrap();
    fs::create_dir_all(dir_path.join("obj")).unwrap();
    fs::File::create(dir_path.join("project1.csproj")).unwrap();
    fs::File::create(dir_path.join("src/project2.csproj")).unwrap();
    fs::File::create(dir_path.join("src/project3.fsproj")).unwrap();
    fs::File::create(dir_path.join("bin/ignored_project.csproj")).unwrap();
    fs::File::create(dir_path.join("obj/ignored_project.csproj")).unwrap();

    // when
    let result = search_projects(&dir_path);

    // then
    match result {
        Ok(files) => {
            let mut paths: Vec<PathBuf> = vec![
                dir_path.join("project1.csproj"),
                dir_path.join("src/project2.csproj"),
                dir_path.join("src/project3.fsproj"),
            ];
            paths.sort();
            let mut result_files = files;
            result_files.sort();
            assert_eq!(result_files, paths);
        }
        Err(e) => panic!("Test failed with error: {:?}", e),
    }

    dir.close().unwrap();
}

#[test]
fn test_no_csproj_files() {
    // given
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    fs::create_dir_all(dir_path.join("src")).unwrap();
    fs::create_dir_all(dir_path.join("bin")).unwrap();
    fs::create_dir_all(dir_path.join("obj")).unwrap();
    fs::File::create(dir_path.join("src/project.txt")).unwrap();

    // when
    let result = search_projects(&dir_path);

    // then
    match result {
        Ok(projects) => assert!(projects.is_empty()),
        _ => unreachable!(),
    }

    dir.close().unwrap();
}
