use std::path::PathBuf;
use std::collections::{HashMap, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};


pub fn get_cache() -> PathBuf {
    if let Some(cache_dir) = dirs::cache_dir() {
        cache_dir.join("werve").join("cache.txt")
    } else {
        // Fallback to default cachedir
        PathBuf::from(".config/werve/cache.txt")
    }
}

pub fn read_cache() -> HashMap<String, i32> {
    let path = get_cache();
    let mut counts = HashMap::new();
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            *counts.entry(line).or_insert(0) += 1;
        }
    }
    counts
}

pub fn update_cache(name: &str) {
    let path = get_cache();
    // Read existing lines
    let mut lines: VecDeque<String> = VecDeque::new();
    if let Ok(file) = File::open(&path) {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            lines.push_back(line);
        }
    }
    // Add new launch
    lines.push_back(name.to_string());
    // Keep only last 100
    while lines.len() > 100 {
        lines.pop_front();
    }
    // Write back
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    if let Ok(mut file) = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
    {
        for line in lines {
            writeln!(file, "{}", line).ok();
        }
    }
}