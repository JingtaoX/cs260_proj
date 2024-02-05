pub mod lir;
pub mod stats;

use std::collections::HashMap;
use std::fs;
use std::io::{self, Result};
use std::path::{Path, PathBuf};
use std::process::exit;
use lir::Program;
use stats::*;

fn main() -> Result<()> {

    // assign0_tests()
    let mut a = HashMap::new();
    let mut b = vec![1, 2, 3];
    a.insert("1", b.clone());
    println!("{:?}", a);
    b.push(4);
    println!("{:?}", a);
    Ok(())
}
