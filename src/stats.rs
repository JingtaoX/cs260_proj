use std::fs::File;
use std::io::{self, BufRead, Result};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Stats {
    pub field_num: u32,
    pub function_returning_value_num: u32,
    pub func_param_num: u32,
    pub local_var_num: u32,
    pub block_num: u32,
    pub instr_num: u32,
    pub terminal_num: u32,
    pub int_type_num: u32,
    pub struct_type_num: u32,
    pub pointer_to_int_num: u32,
    pub pointer_to_struct_num: u32,
    pub pointer_to_function_num: u32,
    pub pointer_to_pointer_num: u32,
}

impl Stats {
    pub fn new()-> Stats {
        Stats {
            field_num: 0,
            function_returning_value_num: 0,
            func_param_num: 0,
            local_var_num: 0,
            block_num: 0,
            instr_num: 0,
            terminal_num: 0,
            int_type_num: 0,
            struct_type_num: 0,
            pointer_to_int_num: 0,
            pointer_to_struct_num: 0,
            pointer_to_function_num: 0,
            pointer_to_pointer_num: 0,
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Stats> {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);

        let mut stats = Stats::new();  // Assuming Stats::new() initializes all fields to 0



        for line in reader.lines() {
            let line = line?;
            if let Some(number) = line.split(": ").nth(1).and_then(|s| s.parse::<u32>().ok()) {
                if line.contains("Number of fields across all struct types") {
                    stats.field_num = number;
                } else if line.contains("Number of functions that return a value") {
                    stats.function_returning_value_num = number;
                } else if line.contains("Number of function parameters") {
                    stats.func_param_num = number;
                } else if line.contains("Number of local variables") {
                    stats.local_var_num = number;
                } else if line.contains("Number of basic blocks") {
                    stats.block_num = number;
                } else if line.contains("Number of instructions") {
                    stats.instr_num = number;
                } else if line.contains("Number of terminals") {
                    stats.terminal_num = number;
                } else if line.contains("Number of locals and globals with int type") {
                    stats.int_type_num = number;
                } else if line.contains("Number of locals and globals with struct type") {
                    stats.struct_type_num = number;
                } else if line.contains("Number of locals and globals with pointer to int type") {
                    stats.pointer_to_int_num = number;
                } else if line.contains("Number of locals and globals with pointer to struct type") {
                    stats.pointer_to_struct_num = number;
                } else if line.contains("Number of locals and globals with pointer to function type") {
                    stats.pointer_to_function_num = number;
                } else if line.contains("Number of locals and globals with pointer to pointer type") {
                    stats.pointer_to_pointer_num = number;
                }
            }
        }

        Ok(stats)
    }
}
