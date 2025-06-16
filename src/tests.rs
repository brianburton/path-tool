// Utility to edit, filter, and print unix PATH-like strings.
// Copyright (C) 2025  Brian Burton
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::*;
use std::env::set_var;

const TEST_ROOT: &str = "test_dirs";

fn dir(s: &str) -> String {
    format!("{}/{}", TEST_ROOT, s)
}

fn err_message(e: anyhow::Error) -> String {
    format!("{:?}", e)
}

fn normal_dir(s: &str) -> String {
    let mut prefix = fs::canonicalize(Path::new(TEST_ROOT))
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(prefix.len() > 0);
    prefix += "/";
    prefix += s;
    prefix
}

// Determines the canonical path of TEST_ROOT and then removes
// it from the start of the given directory path.
// Intended for use in a test so makes assumptions about
// unwrap being safe.
fn rm_prefix(path: String) -> String {
    let prefix = normal_dir("");
    assert!(
        path.starts_with(prefix.as_str()),
        "path did not start with {}: {}",
        prefix,
        path
    );
    path.strip_prefix(prefix.as_str()).unwrap().to_string()
}

// Determines the canonical path of TEST_ROOT and then removes
// it from the start of the given directory path.
// Intended for use in a test so makes assumptions about
// unwrap being safe.
fn rm_prefix_opt(dir: Option<String>) -> Option<String> {
    dir.map(|path| rm_prefix(path))
}

fn strings(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| String::from(*s)).collect()
}

#[test]
fn test_remove() {
    let mut v = strings(&["a", "b", "b"]);
    let unchanged = v.clone();

    remove(&mut v, "x");
    assert_eq!(v, unchanged);

    remove(&mut v, "a");
    assert_eq!(v, strings(&["b", "b"]));

    let mut v = unchanged.clone();
    remove(&mut v, "b");
    assert_eq!(v, strings(&["a"]));
}

#[test]
fn test_is_valid() {
    assert_eq!(is_valid(":").map_err(err_message), Ok(false));
    assert_eq!(is_valid(TEST_ROOT).ok(), Some(true));
    assert_eq!(is_valid(dir("a").as_str()).ok(), Some(true));
    assert_eq!(is_valid(dir("b/bb").as_str()).ok(), Some(true));
    assert_eq!(is_valid(dir("z").as_str()).ok(), Some(false));
    assert_eq!(is_valid(dir("a/keepme.txt").as_str()).ok(), Some(false));
    assert_eq!(is_valid(dir("la").as_str()).ok(), Some(true));
    assert_eq!(is_valid(dir("laa").as_str()).ok(), Some(true));
    assert_eq!(is_valid(dir("broken").as_str()).ok(), Some(false));
    assert_eq!(is_valid(dir("broken2").as_str()).ok(), Some(false));
}

#[test]
fn test_canonicalize() {
    assert_eq!(
        canonicalize(dir("a").as_str())
            .map_err(err_message)
            .map(rm_prefix_opt),
        Ok(Some("a".to_string()))
    );
    assert_eq!(
        canonicalize(dir("b").as_str())
            .map_err(err_message)
            .map(rm_prefix_opt),
        Ok(Some("b".to_string()))
    );
    assert_eq!(
        canonicalize(dir("b/bb").as_str())
            .map_err(err_message)
            .map(rm_prefix_opt),
        Ok(Some("b/bb".to_string()))
    );
    assert_eq!(
        canonicalize(dir("la").as_str())
            .map_err(err_message)
            .map(rm_prefix_opt),
        Ok(Some("a".to_string()))
    );
    assert_eq!(
        canonicalize(dir("laa").as_str())
            .map_err(err_message)
            .map(rm_prefix_opt),
        Ok(Some("a".to_string()))
    );
    assert_eq!(
        canonicalize(dir("broken").as_str()).map_err(err_message),
        Ok(None)
    );
    assert_eq!(
        canonicalize(dir("broken2").as_str()).map_err(err_message),
        Ok(None)
    );
}

#[test]
fn test_filter() {
    assert_eq!(
        filter(vec![
            dir("laa"),
            dir("b/bb"),
            dir("c"),
            dir("broken2"),
            dir("b/bb"),
            dir("z"),
            dir("b")
        ])
        .into_iter()
        .collect::<Vec<String>>(),
        vec![dir("laa"), dir("b/bb"), dir("c"), dir("b")]
    );
}

#[test]
fn test_normalize() {
    assert_eq!(
        normalize(vec![dir("a")])
            .into_iter()
            .map(rm_prefix)
            .collect::<Vec<String>>(),
        vec!["a".to_string()]
    );

    // should be unique in the normalized path
    assert_eq!(
        normalize(vec![dir("a"), dir("la")])
            .into_iter()
            .map(rm_prefix)
            .collect::<Vec<String>>(),
        vec!["a".to_string()]
    );
    assert_eq!(
        normalize(vec![dir("laa"), dir("a")])
            .into_iter()
            .map(rm_prefix)
            .collect::<Vec<String>>(),
        vec!["a".to_string()]
    );

    assert_eq!(
        normalize(vec![
            dir("laa"),
            dir("b/bb"),
            dir("c"),
            dir("broken2"),
            dir("b/bb"),
            dir("z"),
            dir("b")
        ])
        .into_iter()
        .map(rm_prefix)
        .collect::<Vec<String>>(),
        vec![
            "a".to_string(),
            "b/bb".to_string(),
            "c".to_string(),
            "b".to_string(),
        ]
    );
}

#[test]
fn test_add_unique() {
    let mut path: Vec<String> = Vec::new();

    add_unique(&mut path, "");
    assert_eq!(path, Vec::<String>::new());

    add_unique(&mut path, "a");
    add_unique(&mut path, "b");
    add_unique(&mut path, "c");
    assert_eq!(path, vec!["a", "b", "c"]);

    add_unique(&mut path, "c");
    add_unique(&mut path, "b");
    add_unique(&mut path, "a");
    assert_eq!(path, vec!["a", "b", "c"]);
}

#[test]
fn test_parse_path() {
    assert_eq!(parse_path(""), Vec::<String>::new());
    assert_eq!(parse_path("::"), Vec::<String>::new());
    assert_eq!(parse_path(":/foo::/bar:"), vec!["/foo", "/bar"]);
    assert_eq!(parse_path("/foo:/bar:/baz"), vec!["/foo", "/bar", "/baz"]);
    assert_eq!(
        parse_path("/foo:/bar:/foo:/baz:/bar"),
        vec!["/foo", "/bar", "/baz"]
    );
    assert_eq!(
        parse_raw_path("/foo:/bar:/foo:/baz:/bar"),
        vec!["/foo", "/bar", "/foo", "/baz", "/bar"]
    );
}

#[test]
fn test_print() {
    let env_var = "TEST_PATH_PRINT".to_string();
    let base_cli = Cli {
        env: env_var.to_owned(),
        command: Commands::Print,
        ..Cli::default()
    };
    let path = vec![dir("b"), dir("c"), dir("z")].join(":");
    let cli = base_cli.clone();
    unsafe {
        set_var(env_var.to_owned(), path);
    }
    let mut buf = Vec::new();
    main_logic(cli, &mut buf).unwrap();
    assert_eq!(
        String::from_utf8(buf).unwrap(),
        format!("{}\n{}\n{}\n", dir("b"), dir("c"), dir("z"))
    );

    let cli = Cli {
        filter: true,
        ..base_cli.clone()
    };
    let mut buf = Vec::new();
    main_logic(cli, &mut buf).unwrap();
    assert_eq!(
        String::from_utf8(buf).unwrap(),
        format!("{}\n{}\n", dir("b"), dir("c"))
    );

    let cli = Cli {
        normalize: true,
        ..base_cli.clone()
    };
    let mut buf = Vec::new();
    main_logic(cli, &mut buf).unwrap();
    assert_eq!(
        String::from_utf8(buf).unwrap(),
        format!("{}\n{}\n", normal_dir("b"), normal_dir("c"))
    );
}

#[test]
fn test_new() {
    let env_var = "TEST_PATH_NEW".to_string();
    let base_cli = Cli {
        env: env_var.to_owned(),
        command: Commands::New {
            directories: vec![
                dir("la"),
                dir("b"),
                dir("a"),
                dir("c"),
                dir("z"),
                dir("x"),
                dir("b/bb"),
            ],
        },
        ..Cli::default()
    };
    let cli = base_cli.clone();
    let mut buf = Vec::new();
    main_logic(cli, &mut buf).unwrap();
    assert_eq!(
        String::from_utf8(buf).unwrap(),
        vec![
            dir("la"),
            dir("b"),
            dir("a"),
            dir("c"),
            dir("z"),
            dir("x"),
            dir("b/bb")
        ]
        .join(":")
            + "\n"
    );

    let cli = Cli {
        filter: true,
        ..base_cli.clone()
    };
    let mut buf = Vec::new();
    main_logic(cli, &mut buf).unwrap();
    assert_eq!(
        String::from_utf8(buf).unwrap(),
        vec![dir("la"), dir("b"), dir("a"), dir("c"), dir("b/bb")].join(":") + "\n"
    );

    let cli = Cli {
        normalize: true,
        ..base_cli.clone()
    };
    let mut buf = Vec::new();
    main_logic(cli, &mut buf).unwrap();
    assert_eq!(
        String::from_utf8(buf).unwrap(),
        vec![
            normal_dir("a"),
            normal_dir("b"),
            normal_dir("c"),
            normal_dir("b/bb")
        ]
        .join(":")
            + "\n"
    );
}

#[test]
fn test_add() {
    let env_var = "TEST_PATH_ADD".to_string();
    let base_cli = Cli {
        env: env_var.to_owned(),
        command: Commands::Add {
            directories: vec![dir("la"), dir("x")],
        },
        ..Cli::default()
    };
    let path = vec![dir("b"), dir("a"), dir("c"), dir("z")].join(":");
    let cli = base_cli.clone();
    unsafe {
        set_var(env_var.to_owned(), path);
    }
    let mut buf = Vec::new();
    main_logic(cli, &mut buf).unwrap();
    assert_eq!(
        String::from_utf8(buf).unwrap(),
        vec![dir("la"), dir("x"), dir("b"), dir("a"), dir("c"), dir("z")].join(":") + "\n"
    );

    let cli = Cli {
        filter: true,
        ..base_cli.clone()
    };
    let mut buf = Vec::new();
    main_logic(cli, &mut buf).unwrap();
    assert_eq!(
        String::from_utf8(buf).unwrap(),
        vec![dir("la"), dir("b"), dir("a"), dir("c")].join(":") + "\n"
    );

    let cli = Cli {
        normalize: true,
        ..base_cli.clone()
    };
    let mut buf = Vec::new();
    main_logic(cli, &mut buf).unwrap();
    assert_eq!(
        String::from_utf8(buf).unwrap(),
        vec![normal_dir("a"), normal_dir("b"), normal_dir("c")].join(":") + "\n"
    );
}

#[test]
fn test_append() {
    let env_var = "TEST_PATH_APPEND".to_string();
    let base_cli = Cli {
        env: env_var.to_owned(),
        command: Commands::Append {
            directories: vec![dir("la"), dir("x")],
        },
        ..Cli::default()
    };
    let path = vec![dir("b"), dir("a"), dir("c"), dir("z")].join(":");
    let cli = base_cli.clone();
    unsafe {
        set_var(env_var.to_owned(), path);
    }
    let mut buf = Vec::new();
    main_logic(cli, &mut buf).unwrap();
    assert_eq!(
        String::from_utf8(buf).unwrap(),
        vec![dir("b"), dir("a"), dir("c"), dir("z"), dir("la"), dir("x")].join(":") + "\n"
    );

    let cli = Cli {
        filter: true,
        ..base_cli.clone()
    };
    let mut buf = Vec::new();
    main_logic(cli, &mut buf).unwrap();
    assert_eq!(
        String::from_utf8(buf).unwrap(),
        vec![dir("b"), dir("a"), dir("c"), dir("la")].join(":") + "\n"
    );

    let cli = Cli {
        normalize: true,
        ..base_cli.clone()
    };
    let mut buf = Vec::new();
    main_logic(cli, &mut buf).unwrap();
    assert_eq!(
        String::from_utf8(buf).unwrap(),
        vec![normal_dir("b"), normal_dir("a"), normal_dir("c")].join(":") + "\n"
    );
}

#[test]
fn test_get_invalid_dirs() {
    let path = vec![dir("laa"), dir("broken"), dir("a"), dir("c"), dir("z")].join(":");
    assert_eq!(
        get_invalid_dirs(path.as_str()),
        vec!(dir("broken"), dir("z"))
    );
}

#[test]
fn test_get_duplicate_dirs() {
    let path = vec![
        dir("laa"),
        dir("broken"),
        dir("a"),
        dir("c"),
        dir("laa"),
        dir("a"),
    ]
    .join(":");
    assert_eq!(
        get_duplicate_dirs(path.as_str()),
        vec!(dir("laa"), dir("a"))
    );
}

#[test]
fn test_get_shadowed() {
    let path = vec![dir("a"), dir("b"), dir("c")].join(":");
    assert_eq!(
        get_shadowed(path.as_str()).unwrap(),
        vec![
            (
                dir("b"),
                vec![Shadow::new(dir("a"), "keepme.txt".to_string()),]
            ),
            (
                dir("c"),
                vec![
                    Shadow::new(dir("a"), "keepme.txt".to_string()),
                    Shadow::new(dir("b"), "x".to_string()),
                ]
            ),
        ]
    );
}
