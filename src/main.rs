pub mod lir;
pub mod stats;
mod constants;
mod test;
mod intervals;

use std::collections::HashMap;
use std::fs;
use std::io::{self, Result};
use std::path::{Path, PathBuf};
use std::process::exit;
use lir::Program;
use stats::*;

fn list_filenames_in_dir<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>> {
    let mut file_paths = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("lir") {
            file_paths.push(path);
        }
    }

    Ok(file_paths)
}

fn program_from_json_file<P: AsRef<Path>>(path: P) -> Result<Program> {
    let json_string = fs::read_to_string(path)?;
    let program = Program::parse_json(&json_string);
    Ok(program)
}

fn process_lir_files<P: AsRef<Path>>(dir_path: P) -> Result<()> {
    let mut files = list_filenames_in_dir(dir_path.as_ref())?;
    // Sort the files vector
    files.sort_by(|a, b| {
        a.file_stem().and_then(|s| s.to_str()).cmp(&b.file_stem().and_then(|s| s.to_str()))
    });
    // println!("{:?}", files);
    for path in files {
        let file_stem = path.file_stem().and_then(|s| s.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Invalid file stem"))?;
        println!("working on {}", file_stem);

        let json_path = path.with_extension("lir.json");
        let program = program_from_json_file(&json_path)?;
        println!("lir:\n{:?}", program.get_stats());

        let stats_path = path.with_extension("stats");
        let stats = Stats::from_file(&stats_path)?;
        println!("std:\n{:?}", stats);

        // Replace this with your actual comparison logic
        assert_eq!(stats, program.get_stats());
        println!("{} passed\n", file_stem);
    }

    Ok(())
}

fn assign0_tests() -> Result<()> {
    let dir_name = "./tests";
    process_lir_files(dir_name)
}

fn main() -> Result<()> {

    // assign0_tests()
    assign0_tests()
}
