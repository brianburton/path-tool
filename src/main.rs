use clap::{Parser, Subcommand};
use std::env;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

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
    match cli.command {
        Commands::Print => exec_print(&current),
        Commands::New { directories } => exec_new(&directories),
        Commands::Add { directories } => exec_add(&current, &directories),
        Commands::Append { directories } => exec_append(&current, &directories),
    }
}

fn exec_print(current: &[String]) {
    for dir in current {
        println!("{}", dir);
    }
}

fn exec_new(directories: &[String]) {
    let mut path = Vec::new();
    for arg in directories {
        let parsed = parse_path(arg);
        add_all_last(&mut path, &parsed);
    }
    println!("{}", to_string(&path));
}

fn exec_add(current: &[String], directories: &[String]) {
    let mut path = Vec::new();
    for arg in directories {
        let parsed = parse_path(arg);
        add_all_last(&mut path, &parsed);
    }
    add_all_unique(&mut path, current);
    println!("{}", to_string(&path));
}

fn exec_append(current: &[String], directories: &[String]) {
    let mut path = Vec::new();
    add_all_unique(&mut path, current);
    for arg in directories {
        let parsed = parse_path(arg);
        add_all_last(&mut path, &parsed);
    }
    println!("{}", to_string(&path));
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
