use std::env;
use std::env::Args;
use std::iter::Skip;

fn main() {
    let current = parse_path(&(env::var("PATH").unwrap_or_default()));
    let args = env::args().skip(1);
    if args.len() == 0 {
        print_path(&current)
    } else {
        edit_path(&current, args);
    }
}

fn print_path(current: &Vec<String>) {
    for dir in current {
        println!("{}", dir);
    }
}

fn edit_path(current: &[String], args: Skip<Args>) {
    let mut inserted = false;
    let mut path = Vec::new();
    for arg in args {
        if arg == "PATH" {
            if !inserted {
                add_all(&mut path, current);
                inserted = true;
            }
        } else {
            add_last(&mut path, &arg)
        }
    }
    if !inserted {
        add_all(&mut path, current);
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

fn add_all(path: &mut Vec<String>, other: &[String]) {
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
