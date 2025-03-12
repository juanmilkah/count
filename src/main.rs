use std::collections::HashMap;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{env, fs};

use anyhow::Context;

/// File contents information
#[derive(Debug, Default)]
struct FileStats {
    /// Total number of lines
    lines: u64,

    /// Total number of actual lines of code
    code: u64,

    /// Total number of commented lines
    comments: u64,

    /// Total number of blank lines
    blanks: u64,
}

struct Aggregate {
    /// The Language based HashMap
    /// FileStats and File Count for a specific language
    map: HashMap<String, (FileStats, u64)>,
}

impl Aggregate {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn add(&mut self, lang: String, stats: FileStats) {
        let lang_stats = self.map.entry(lang).or_default();
        lang_stats.0.blanks += stats.blanks;
        lang_stats.0.comments += stats.comments;
        lang_stats.0.code += stats.code;
        lang_stats.0.lines += stats.lines;
        lang_stats.1 += 1;
    }
}

impl FileStats {
    fn new() -> Self {
        Self::default()
    }
}

fn read_file_content(filepath: &Path) -> anyhow::Result<String> {
    let file = fs::OpenOptions::new()
        .read(true)
        .open(filepath)
        .context(format!("open file :{:?}", filepath))?;

    let mut reader = BufReader::new(file);
    let mut content = String::new();

    reader
        .read_to_string(&mut content)
        .context("read file content")?;
    Ok(content)
}

fn read_dir_recursively(dir_path: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir_path).context(format!("read_dir files: {:?}", dir_path))? {
        let entry = entry?;
        let file = entry.path();
        if file.is_dir() {
            let mut subdir_files = read_dir_recursively(&file)?;
            files.append(&mut subdir_files);
        } else {
            files.push(file);
        }
    }

    Ok(files)
}

fn process_markdown_file(filepath: &Path) -> anyhow::Result<FileStats> {
    let content = read_file_content(filepath)?;

    let mut stats = FileStats::new();

    for line in content.lines() {
        if line.is_empty() {
            stats.blanks += 1;
            stats.lines += 1;
            continue;
        }
        stats.code += 1;
        stats.lines += 1;
    }

    Ok(stats)
}

fn main() -> anyhow::Result<()> {
    // for file in directory
    // match according to extension
    // files to skip:
    // starts with "."
    // git files
    // executables && binaries
    let args = env::args().skip(1).collect::<Vec<String>>();

    if args.is_empty() {
        eprintln!("No filepath provided!");
        exit(1);
    }

    let mut files = Vec::new();
    let mut global_stats = Aggregate::new();

    // process args
    for filepath in args {
        let filepath = Path::new(&filepath);
        if filepath.is_dir() {
            let mut sub_dir_files = read_dir_recursively(filepath)?;
            files.append(&mut sub_dir_files);
        } else {
            files.push(filepath.to_path_buf());
        }
    }

    // make this parallel later on
    for file in &files {
        let (lang, stats) = if let Some(ext) = file.extension() {
            match ext.to_str() {
                Some("md") => (Some("Markdown"), process_markdown_file(file)?),
                _ => (None, FileStats::new()),
            }
        } else {
            // Files that start with "." fall in this else statement
            (None, FileStats::new())
        };

        if let Some(lang) = lang {
            global_stats.add(lang.to_string(), stats);
        }
    }

    {
        for (lang, stats) in &global_stats.map {
            println!("LANGUAGE  FILES    CODE    COMMENTS   BLANKS  TOTAL LINES");
            println!(
                "{lang}:   {}  {}  {}  {}   {}",
                stats.1, stats.0.code, stats.0.comments, stats.0.blanks, stats.0.lines
            );
        }
        let total_files = global_stats.map.keys().count();
        println!("TOTAL FILES:  {total_files}");
    }

    Ok(())
}
