use std::collections::HashMap;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{env, fs};

use anyhow::Context;

/// File contents information for a single file
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

/// Information about a specific language
#[derive(Debug, Default)]
struct LanguageStats {
    /// Accumulated statistics across all files
    stats: FileStats,

    /// Number of files for this language
    file_count: u64,
}

/// Manages all statistics for the program
#[derive(Debug)]
struct StatisticsManager {
    /// Language-specific statistics
    language_stats: HashMap<String, LanguageStats>,

    /// File extension to language mapping
    extension_map: HashMap<String, String>,

    /// Processors for different file types
    processors: HashMap<String, fn(&Path) -> anyhow::Result<FileStats>>,
}

impl FileStats {
    fn add(&mut self, stats: FileStats) {
        self.blanks += stats.blanks;
        self.comments += stats.comments;
        self.lines += stats.lines;
        self.code += stats.code;
    }
}

impl LanguageStats {
    fn add(&mut self, stats: FileStats) {
        self.stats.add(stats);
        self.file_count += 1;
    }
}

impl StatisticsManager {
    fn new() -> Self {
        let mut manager = Self {
            language_stats: HashMap::new(),
            extension_map: HashMap::new(),
            processors: HashMap::new(),
        };

        //extensions
        manager
            .extension_map
            .insert("md".to_string(), "Markdown".to_string());

        //processors
        manager
            .processors
            .insert("md".to_string(), process_markdown_file);

        manager
    }

    fn process_file(&mut self, filepath: &Path) -> anyhow::Result<()> {
        if let Some(ext) = filepath.extension().and_then(|e| e.to_str()) {
            if let Some(processor) = self.processors.get(ext) {
                let stats = processor(filepath)?;
                if let Some(lang) = self.extension_map.get(ext) {
                    let lang_stats = self.language_stats.entry(lang.to_string()).or_default();
                    lang_stats.add(stats);
                }
            }
        }

        Ok(())
    }

    fn print_statistics(&self) {
        println!("LANGUAGE  FILES    CODE    COMMENTS   BLANKS  TOTAL LINES");
        println!("{}", "*".repeat(58));

        for (lang, stats) in &self.language_stats {
            println!(
                "{lang}:   {}  {}  {}  {}   {}",
                stats.file_count,
                stats.stats.code,
                stats.stats.comments,
                stats.stats.blanks,
                stats.stats.lines
            );
        }

        println!("TOTAL FILES:  {}", self.total_files());
    }

    fn total_files(&self) -> u64 {
        self.language_stats
            .values()
            .map(|stats| stats.file_count)
            .sum()
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

    let mut stats = FileStats::default();

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
    let mut global_stats = StatisticsManager::new();

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
        global_stats.process_file(file)?;
    }

    global_stats.print_statistics();

    Ok(())
}
