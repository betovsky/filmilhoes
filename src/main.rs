
#[macro_use]
extern crate structopt;

extern crate human_size;

use std::fs::DirEntry;
use std::io::Result;
use std::path::PathBuf;
use human_size::Size;
use structopt::StructOpt;

/// Filmilhoes - random pick files
#[derive(StructOpt, Debug)]
#[structopt(name = "Filmilhoes")]
struct Opt {
    /// Directory to analyse
    #[structopt(name = "DIRECTORY", parse(from_os_str))]
    directory: PathBuf,

    /// Set max number of files to pick
    #[structopt(short = "n", long = "number", default_value = "5")]
    n: usize,
    
    /// Minimum size of files to pick
    #[structopt(short = "s", long = "size", default_value = "100MB")]
    min_size: Size
}


fn main() {
    let opt = Opt::from_args();

    let min_size = opt.min_size.into_bytes() as u64;
    let files = get_files(&opt.directory, min_size);

    for file in files.iter().take(opt.n) {
        let name = file.strip_prefix(&opt.directory).unwrap_or(file);
        let size = get_file_len(file).unwrap_or(0);
        let readable_size = format_size(size);
        println!("File: {}   (Size: {})", name.display(), readable_size);
    }

}

static SCALES: &'static [&str] = &["B", "KB", "MB", "GB", "TB"];

fn format_size(size: u64) -> String {
    let mut scale = 0usize;
    let base = 1024f64;
    let mut scalled = size as f64;

    while scalled > base {
        scalled /= base;
        scale += 1;
    }

    format!("{:.2} {}", scalled, SCALES[scale])
}

fn get_files(directory: &PathBuf, min_size: u64) -> Vec<PathBuf> {
    if !directory.is_dir() {
        return Vec::new();
    }
    
    let mut vec = Vec::with_capacity(1000);
    {
        let mut check_file = |file_entry: &DirEntry| {
            if let Ok(metadata) = file_entry.metadata() {
                if metadata.len() >= min_size {
                    vec.push(file_entry.path())
                }
            }
        };
        visit_dirs(directory, &mut check_file).expect("Failed to search directory");
    }
    vec
}

fn get_file_len(path: &PathBuf) -> Result<u64> {
    let metadata = path.metadata()?;
    Ok(metadata.len())
}

fn visit_dirs(dir: &PathBuf, cb: &mut FnMut(&DirEntry)) -> Result<()> {
    if dir.is_dir() {
        for entry in dir.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}