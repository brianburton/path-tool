use clap::{Parser, Subcommand};
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

    /// Normalize directory names in path
    #[arg(short, long, default_value_t = false)]
    normalize: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
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
    let current = parse_path(&(env::var("PATH").unwrap_or_default()));
    let mut path = match cli.command {
        Commands::Print => exec_print(&current),
        Commands::New { directories } => exec_new(&directories),
        Commands::Add { directories } => exec_add(&current, &directories),
        Commands::Append { directories } => exec_append(&current, &directories),
    };
    if cli.filter {
        filter(&mut path);
    }
    if cli.normalize {
        normalize(&mut path);
    }
    if !path.is_empty() {
        println!("{}", to_string(&path));
    }
}

fn exec_print(current: &[String]) -> Vec<String> {
    for dir in current {
        println!("{}", dir);
    }
    vec![]
}

fn exec_new(directories: &[String]) -> Vec<String> {
    let mut path = Vec::new();
    for arg in directories {
        let parsed = parse_path(arg);
        add_all_last(&mut path, &parsed);
    }
    path
}

fn exec_add(current: &[String], directories: &[String]) -> Vec<String> {
    let mut path = Vec::new();
    for arg in directories {
        let parsed = parse_path(arg);
        add_all_last(&mut path, &parsed);
    }
    add_all_unique(&mut path, current);
    path
}

fn exec_append(current: &[String], directories: &[String]) -> Vec<String> {
    let mut path = Vec::new();
    add_all_unique(&mut path, current);
    for arg in directories {
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

fn filter(path: &mut Vec<String>) {
    let mut i = path.len();
    while i > 0 {
        i -= 1;
        if is_invalid(&path[i]).unwrap_or(false) {
            path.remove(i);
        }
    }
}

fn normalize(path: &mut [String]) {
    for p in path.iter_mut() {
        if let Some(s) = expand(p.as_str()) {
            if let Ok(d) = fs::canonicalize(s.as_str()) {
                if let Some(c) = d.to_str() {
                    *p = String::from(c);
                }
            }
        }
    }
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

fn is_invalid(path: &str) -> Option<bool> {
    let d = fs::metadata(Path::new(&path)).ok()?;
    if d.is_symlink() {
        let link_path = fs::read_link(path).ok()?;
        return is_invalid(link_path.to_str()?);
    }
    Some(d.is_dir())
}

fn expand(path: &str) -> Option<String> {
    let d = fs::metadata(Path::new(&path)).ok()?;
    if d.is_symlink() {
        let link_path = fs::read_link(path).ok()?;
        return expand(link_path.to_str()?);
    }
    if d.is_dir() {
        Some(path.to_string())
    } else {
        None
    }
}
