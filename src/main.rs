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

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use itertools::Itertools;
use std::io::{Write, stdout};
use std::path::Path;
use std::{env, fs};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Name of path environment variable
    #[arg(short, long, default_value = "PATH")]
    env: String,

    /// Filter non-directories from path
    #[arg(short, long, default_value_t = false)]
    filter: bool,

    /// Print path one directory per line
    #[arg(short, long, default_value_t = false)]
    pretty: bool,

    /// Normalize directory names in path
    #[arg(short, long, default_value_t = false)]
    normalize: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Commands {
    /// Print the current PATH one directory per line
    Print,
    /// Build a new PATH from directories
    New { directories: Vec<String> },
    /// Add directories to front of PATH
    Add { directories: Vec<String> },
    /// Add directories to back of PATH
    Append { directories: Vec<String> },
}

fn main() -> Result<()> {
    main_logic(Cli::parse(), &mut stdout())
}

fn main_logic(cli: Cli, output: &mut impl Write) -> Result<()> {
    let current = parse_path(&(env::var(cli.env).unwrap_or_default()));
    let pretty = cli.pretty || cli.command == Commands::Print;
    let mut path = match cli.command {
        Commands::Print => current,
        Commands::New { directories } => exec_new(directories),
        Commands::Add { directories } => exec_add(&current, directories),
        Commands::Append { directories } => exec_append(&current, directories),
    };
    path = apply_filters(path, cli.filter, cli.normalize);
    if pretty {
        exec_print(path, output)
    } else {
        writeln!(output, "{}", to_string(&path)).with_context(|| "Failed to write output")
    }
}

fn exec_print(current: Vec<String>, output: &mut impl Write) -> Result<()> {
    for dir in current {
        writeln!(output, "{}", dir).with_context(|| format!("Failed to print {}", dir))?;
    }
    Ok(())
}

fn exec_new(directories: Vec<String>) -> Vec<String> {
    let mut path = Vec::new();
    for arg in directories.iter() {
        let parsed = parse_path(arg);
        add_all_last(&mut path, &parsed);
    }
    path
}

fn exec_add(current: &[String], directories: Vec<String>) -> Vec<String> {
    let mut path = Vec::new();
    for arg in directories.iter() {
        let parsed = parse_path(arg);
        add_all_last(&mut path, &parsed);
    }
    add_all_unique(&mut path, current);
    path
}

fn exec_append(current: &[String], directories: Vec<String>) -> Vec<String> {
    let mut path = Vec::new();
    add_all_unique(&mut path, current);
    for arg in directories.iter() {
        let parsed = parse_path(arg);
        add_all_last(&mut path, &parsed);
    }
    path
}

fn remove(path: &mut Vec<String>, dir: &str) {
    path.retain(|x| x != dir);
}

fn apply_filters(
    path: Vec<String>,
    filter_requested: bool,
    normalize_requested: bool,
) -> Vec<String> {
    if filter_requested {
        filter(path)
    } else if normalize_requested {
        normalize(path)
    } else {
        path
    }
}

fn filter(path: Vec<String>) -> Vec<String> {
    path.into_iter()
        .filter(|x| is_valid(x).ok() == Some(true))
        .unique()
        .collect::<Vec<String>>()
}

fn normalize(path: Vec<String>) -> Vec<String> {
    path.into_iter()
        .map(|x| canonicalize(&x).unwrap().unwrap_or_default())
        .filter(|x| !x.is_empty())
        .unique()
        .collect::<Vec<String>>()
}

fn add_last(path: &mut Vec<String>, dir: &str) {
    remove(path, dir);
    path.push(dir.to_string());
}

fn add_all_last(path: &mut Vec<String>, other: &[String]) {
    for dir in other.iter() {
        add_last(path, dir);
    }
}

fn add_all_unique(path: &mut Vec<String>, other: &[String]) {
    for dir in other.iter() {
        add_unique(path, dir);
    }
}

fn add_unique(path: &mut Vec<String>, dir: &str) {
    if !dir.is_empty() {
        for p in path.iter() {
            if p == dir {
                return;
            }
        }
        path.push(dir.to_string());
    }
}

fn parse_path(source: &str) -> Vec<String> {
    let mut path: Vec<String> = Vec::new();
    let mut remaining = source;
    while !remaining.is_empty() {
        match remaining.find(':') {
            Some(i) => {
                add_unique(&mut path, &remaining[..i]);
                remaining = &remaining[i + 1..];
            }
            None => {
                add_unique(&mut path, remaining);
                remaining = "";
            }
        }
    }
    path
}

fn to_string(path: &[String]) -> String {
    path.join(":")
}

fn is_valid(path: &str) -> Result<bool> {
    if !Path::new(path).exists() {
        Ok(false)
    } else {
        fs::metadata(Path::new(path))
            .map(|d| d.is_dir())
            .context("unable to read metadata")
    }
}

fn canonicalize(path: &str) -> Result<Option<String>> {
    if !is_valid(path)? {
        Ok(None)
    } else {
        let canonical = fs::canonicalize(Path::new(path))?;
        let canonical = canonical.to_str().map(String::from);
        let canonical = canonical.or_else(|| Some(path.to_string()));
        Ok(canonical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ROOT: &str = "test_dirs";

    fn dir(s: &str) -> String {
        format!("{}/{}", TEST_ROOT, s)
    }

    fn err_message(e: anyhow::Error) -> String {
        format!("{:?}", e)
    }

    // Determines the canonical path of TEST_ROOT and then removes
    // it from the start of the given directory path.
    // Intended for use in a test so makes assumptions about
    // unwrap being safe.
    fn rm_prefix(path: String) -> String {
        let mut prefix = fs::canonicalize(Path::new(TEST_ROOT))
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        assert!(prefix.len() > 0);
        prefix += "/";
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
    }

    #[test]
    fn test_print() {}
}
