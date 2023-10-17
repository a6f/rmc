// review moved code

use levenshtein::levenshtein;
use similar::{ChangeTag, TextDiff};
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

fn hunks(filename: impl AsRef<Path>) -> Vec<String> {
    let file = File::open(filename).unwrap();
    let lines = BufReader::new(file).lines();
    let mut hunks = Vec::<String>::new();
    let mut h = String::new();
    for line in lines {
        let line = line.unwrap();
        let first = line.chars().next().unwrap_or(' ');
        // This is a very poor approximation of matching one "item" of formatted Rust.
        // Note that this discards leading comments and use directives.
        if h.is_empty() {
            if "/u} ".contains(first) {
                continue;
            }
            h.push_str(&line);
            h.push('\n');
        } else {
            h.push_str(&line);
            h.push('\n');
            if first == '}' {
                hunks.push(std::mem::take(&mut h));
            }
        }
    }
    hunks
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let old: Vec<String> = args.iter().cloned().take_while(|a| a != &"--").collect();
    let new: Vec<String> = args
        .iter()
        .cloned()
        .skip_while(|a| a != &"--")
        .skip(1)
        .collect();
    println!("comparing old files {old:?} to new files {new:?}");
    let old: HashSet<String> = old.iter().map(hunks).flatten().collect();
    let new: HashSet<String> = new.iter().map(hunks).flatten().collect();
    println!("total old hunks: {}", old.len());
    println!("total new hunks: {}", new.len());
    println!("common hunks: {}", old.intersection(&new).count());
    // TODO: mark identical hunks as matched to reduce expensive LCS computations
    let mut distances: Vec<(&str, &str, usize)> = Vec::new();
    for h in &old {
        for j in &new {
            distances.push((h, j, levenshtein(h, j)));
        }
    }
    distances.sort_by(|a, b| a.2.cmp(&b.2));
    let mut matched: HashSet<usize> = HashSet::new();
    for (h, j, distance) in distances.iter() {
        let hid = h.as_ptr() as usize;
        let jid = j.as_ptr() as usize;
        if matched.contains(&hid) || matched.contains(&jid) {
            continue;
        }
        matched.insert(hid);
        matched.insert(jid);
        println!("matched {} to {} with {} edits", h.len(), j.len(), distance);
        println!("{0:->72}", "");
        let diff = TextDiff::from_lines(h.to_owned(), j.to_owned());
        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            print!("{}{}", sign, change);
        }
        println!("{0:->72}\n", "");
    }
    println!("unmatched old hunks:");
    for h in old {
        let hid = h.as_ptr() as usize;
        if !matched.contains(&hid) {
            println!("{0:->72}\n{h}{0:->72}\n", "");
        }
    }
    println!("unmatched new hunks:");
    for h in new {
        let hid = h.as_ptr() as usize;
        if !matched.contains(&hid) {
            println!("{0:->72}\n{h}{0:->72}\n", "");
        }
    }
}
