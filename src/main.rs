
#[macro_use]
extern crate structopt;

extern crate human_size;
extern crate rand;
extern crate yaml_rust;

use std::iter::empty;
use std::str::FromStr;
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs::File;
use std::io::Result;
use std::io::prelude::*;
use std::path::PathBuf;

use human_size::{Size, Multiple};
use rand::{thread_rng, seq};
use structopt::StructOpt;
use yaml_rust::{Yaml, YamlLoader};

/// Filmilhoes - random pick files
#[derive(StructOpt, Debug)]
#[structopt(name = "Filmilhoes")]
struct Opt {
    /// Directory to analyse
    #[structopt(name = "DIRECTORY", parse(from_os_str))]
    directory: PathBuf,

    /// Set max number of files to pick
    #[structopt(short = "n", long = "number")]
    n: Option<usize>,
    
    /// Minimum size of files to pick
    #[structopt(short = "s", long = "size")]
    min_size: Option<Size>,

    /// Directories to exclude
    #[structopt(short = "x", long = "exclude", parse(from_os_str))]
    exclude: Vec<OsString>,
}

struct Settings {
    directory: PathBuf,
    n: usize,
    min_size: Option<u64>,
    exclude: HashSet<OsString>
}

const DEFAULT_YAML: &'static str = "files: 1";

fn get_settings() -> Settings {
    let opt = Opt::from_args();

    let yaml_path = opt.directory.join(".filmilhoes.yml");
    let yamls = if let Ok(mut f) = File::open(yaml_path) {
        let mut buffer = String::new();
        f.read_to_string(&mut buffer).expect("Failed to read yaml file!");
        YamlLoader::load_from_str(&buffer).expect("Invalid yaml file!")
    } else {
        YamlLoader::load_from_str(&DEFAULT_YAML).unwrap()
    };

    let yaml = &yamls[0];

    Settings {
        directory: opt.directory,
        n: opt.n.unwrap_or(
            match yaml["files"] {
                Yaml::Integer(i) if i > 0 => i as usize,
                Yaml::BadValue            => 1usize,
                _                         => panic!("YAML: 'files' must be a positive number")
            }),
        min_size: opt.min_size.or(
            match yaml["minsize"] {
                Yaml::String(ref size) => Some(Size::from_str(size).expect("YAML: 'minsize' must be a file size")),
                Yaml::BadValue     => None,
                _                  => panic!("YAML: 'minsize' must be a file size")
            })
            .map(| s | s.into_bytes() as u64),
        exclude: opt.exclude.into_iter()
            .chain(
                match yaml["exclude"] {
                    Yaml::Array(ref array) => array.iter().map(|e| {
                        match e {
                            &Yaml::String(ref s) => OsString::from(s),
                            _ => panic!("YAML: Invalid 'exclude' item")
                        }
                    }).collect::<Vec<_>>(),
                    Yaml::BadValue   => empty::<OsString>().collect::<Vec<_>>(),
                    _ => panic!("YAML: 'exclude' is a list of strings")
                }
            )
            .collect()
    }
}

fn main() {
    let settings = get_settings();
    let files = get_files(&settings);

    let mut rng = thread_rng();
    let sample = if files.len() >= settings.n {
        seq::sample_slice(&mut rng, &files, settings.n)
    } else {
        files
    };

    for file in sample.iter() {
        let name = file.file_name().unwrap().to_string_lossy();
        let size = get_file_len(file).unwrap_or(0);
        let readable_size = format_size(size);
        println!(" {} ║ {}", readable_size, name);
    }
}

static SCALES: &'static [Multiple] = &[Multiple::Byte, Multiple::Kibibyte, Multiple::Mebibyte, Multiple::Gigibyte, Multiple::Tebibyte];

fn format_size(size: u64) -> String {
    let mut scale = 0usize;
    let base = 1024f64;
    let mut scalled = size as f64;

    while scalled > base {
        scalled /= base;
        scale += 1;
    }

    format!("{:7.2} {:>3}", scalled, SCALES[scale])
}

fn get_files(settings: &Settings) -> Vec<PathBuf> {
    let directory = &settings.directory;

    if !directory.is_dir() {
        return Vec::new();
    }
    
    let mut vec = Vec::with_capacity(1000);
    {
        let r = if let Some(size) = settings.min_size {
            visit_dirs(directory, &settings.exclude, &mut |file_path| {
                let len = get_file_len(&file_path).expect("Failed to get file length");
                if len >= size {
                    vec.push(file_path);
                }
            })
        } else {
            visit_dirs(directory, &settings.exclude, &mut |file_path| vec.push(file_path))
        };
        r.expect("Failed to search directory");
    }
    vec
}

fn get_file_len(path: &PathBuf) -> Result<u64> {
    let metadata = path.metadata()?;
    Ok(metadata.len())
}

fn visit_dirs(dir: &PathBuf, exclude: &HashSet<OsString>, cb: &mut FnMut(PathBuf)) -> Result<()> {
    if dir.is_dir() {
        for entry in dir.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name();
                if !exclude.contains(&name) {
                    visit_dirs(&path, exclude, cb)?;
                }
            } else {
                cb(path);
            }
        }
    }
    Ok(())
}