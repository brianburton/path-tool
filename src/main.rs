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
use std::collections::HashSet;
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

fn main() {
    let cli = Cli::parse();
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
        exec_print(path);
    } else {
        println!("{}", to_string(&path));
    }
}

fn exec_print(current: Vec<String>) {
    for dir in current {
        println!("{}", dir);
    }
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
    let mut i = path.len();
    while i > 0 {
        i -= 1;
        if path[i] == dir {
            path.remove(i);
        }
    }
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
    let mut new_path = Vec::new();
    for dir in path.iter() {
        if let Ok(true) = is_valid(dir) {
            new_path.push(dir.to_string());
        }
    }
    new_path
}

fn normalize(path: Vec<String>) -> Vec<String> {
    let mut uniques = HashSet::new();
    let mut new_path = Vec::new();
    for dir in path.iter() {
        if let Ok(Some(canonical)) = canonicalize(dir) {
            if uniques.insert(canonical.to_string()) {
                new_path.push(canonical);
            }
        }
    }
    new_path
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
    for p in path.iter() {
        if p == dir {
            return;
        }
    }
    path.push(dir.to_string());
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
    if !fs::metadata(Path::new(path))?.is_dir() {
        Ok(None)
    } else {
        let canonical = fs::canonicalize(Path::new(path))?;
        let canonical = canonical.to_str().map(String::from);
        let canonical = canonical.or_else(|| Some(path.to_string()));
        Ok(canonical)
    }
}
