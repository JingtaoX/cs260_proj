pub mod lir;
pub mod stats;

use std::fs;

use std::process::exit;
use crate::lir::{*};
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
enum IntConstAbsVal {
    Top,
    Bottom,
    IntConst(i32),
}

impl IntConstAbsVal {
    fn as_string(&self) -> String {
        match self {
            IntConstAbsVal::Top => "Top".to_string(),
            IntConstAbsVal::Bottom => "Bottom".to_string(),
            IntConstAbsVal::IntConst(i) => i.to_string(),
        }
    }
    fn arith(op1: &IntConstAbsVal, op2: &IntConstAbsVal, aop: &ArithOp) -> IntConstAbsVal {
        match (op1, op2) {
            (IntConstAbsVal::IntConst(i), IntConstAbsVal::IntConst(j)) => {
                match aop {
                    ArithOp::Add => IntConstAbsVal::IntConst(i + j),
                    ArithOp::Subtract => IntConstAbsVal::IntConst(i - j),
                    ArithOp::Multiply => IntConstAbsVal::IntConst(i * j),
                    ArithOp::Divide => {
                        if j == &0 {
                            // bot?
                            IntConstAbsVal::Top
                        } else {
                            IntConstAbsVal::IntConst(i / j)
                        }
                    }
                }
            }
            (IntConstAbsVal::Bottom, _) |
            (_, IntConstAbsVal::Bottom) => IntConstAbsVal::Bottom,
            _ => IntConstAbsVal::Top,
        }
    }
    fn cmp(op1: &IntConstAbsVal, op2: &IntConstAbsVal, rop: &RelaOp) -> IntConstAbsVal {
        // println!("op1: {:?}, op2: {:?}, rop: {:?}", op1, op2, rop);
        match (op1, op2) {
            (IntConstAbsVal::IntConst(i), IntConstAbsVal::IntConst(j)) => {
                match rop {
                    RelaOp::Eq => if i == j { IntConstAbsVal::IntConst(1) } else { IntConstAbsVal::IntConst(0) },
                    RelaOp::Neq => if i != j { IntConstAbsVal::IntConst(1) } else { IntConstAbsVal::IntConst(0) },
                    RelaOp::Less => if i < j { IntConstAbsVal::IntConst(1) } else { IntConstAbsVal::IntConst(0) },
                    RelaOp::LessEq => if i <= j { IntConstAbsVal::IntConst(1) } else { IntConstAbsVal::IntConst(0) },
                    RelaOp::Greater => if i > j { IntConstAbsVal::IntConst(1) } else { IntConstAbsVal::IntConst(0) },
                    RelaOp::GreaterEq => if i >= j { IntConstAbsVal::IntConst(1) } else { IntConstAbsVal::IntConst(0) },
                }
            }
            // (IntConstAbsVal::Bottom, IntConstAbsVal::Bottom) => IntConstAbsVal::Top,
            (_, IntConstAbsVal::Bottom) |
            (IntConstAbsVal::Bottom, _) => IntConstAbsVal::Bottom,
            _ => IntConstAbsVal::Top,
        }
    }
}

impl IntConstAbsVal {
    fn join(a: &IntConstAbsVal, b: &IntConstAbsVal) -> IntConstAbsVal {
        match (a, b) {
            (IntConstAbsVal::Top, _) => IntConstAbsVal::Top,
            (_, IntConstAbsVal::Top) => IntConstAbsVal::Top,
            (IntConstAbsVal::Bottom, _) => b.clone(),
            (_, IntConstAbsVal::Bottom) => a.clone(),
            (IntConstAbsVal::IntConst(i), IntConstAbsVal::IntConst(j)) => {
                if i == j {
                    IntConstAbsVal::IntConst(*i)
                } else {
                    IntConstAbsVal::Top
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct AbstractStore {
    store: HashMap<String, IntConstAbsVal>,
}

impl AbstractStore {
    fn new() -> AbstractStore {
        AbstractStore {
            store: HashMap::new(),
        }
    }

    fn insert(&mut self, bb: String, value: IntConstAbsVal) {
        self.store.insert(bb, value);
    }

    // change self according to join, return true if self is changed
    fn join(&mut self, store_to_join: &AbstractStore) -> bool {
        let mut changed = false;
        for (bb, value_to_join) in &store_to_join.store {
            let old_value =
                self.store.entry(bb.clone()).or_insert(IntConstAbsVal::Bottom);
            let new_value = IntConstAbsVal::join(value_to_join, old_value);
            if new_value != *old_value {
                *old_value = new_value;
                changed = true;
            }
        }
        changed
    }

    fn resolve_operand(&self, operand: &Operand) -> IntConstAbsVal {
        match operand {
            Operand::Var(v) => self.store.get(&v.name).cloned().unwrap_or(IntConstAbsVal::Bottom),
            Operand::CInt(i) => IntConstAbsVal::IntConst(*i),
        }
    }

    fn execute(&mut self, inst: &Instruction) {
        match inst {
            Instruction::Copy { lhs, op } => {
                if lhs.typ != Type::Int { return; }
                let op = self.resolve_operand(op);
                self.insert(lhs.name.clone(), op);
            }
            Instruction::Arith { lhs, op1, op2, aop } => {
                let mut exist_not_int_op = false;
                if let Operand::Var(v) = op1 {
                    if v.typ != Type::Int { exist_not_int_op = true; }
                }
                if let Operand::Var(v) = op2 {
                    if v.typ != Type::Int { exist_not_int_op = true; }
                }
                if exist_not_int_op {
                    self.insert(lhs.name.clone(), IntConstAbsVal::Top);
                }
                else {
                    let op1 = self.resolve_operand(op1);
                    let op2 = self.resolve_operand(op2);
                    let result = IntConstAbsVal::arith(&op1, &op2, aop);
                    self.insert(lhs.name.clone(), result);
                }

            }
            Instruction::Cmp { lhs, op1, op2, rop } => {
                // println!("lhs: {:?}, op1: {:?}, op2: {:?}, rop: {:?}", lhs, op1, op2, rop);
                let mut exist_not_int_op = false;
                if let Operand::Var(v) = op1 {
                    if v.typ != Type::Int { exist_not_int_op = true; }
                }
                if let Operand::Var(v) = op2 {
                    if v.typ != Type::Int { exist_not_int_op = true; }
                }
                if exist_not_int_op {
                    self.insert(lhs.name.clone(), IntConstAbsVal::Top);
                }
                else {
                    let op1 = self.resolve_operand(op1);
                    let op2 = self.resolve_operand(op2);
                    let result = IntConstAbsVal::cmp(&op1, &op2, rop);
                    self.insert(lhs.name.clone(), result);
                }
            }
            Instruction::Load { lhs, src:_src } => {
                if lhs.typ != Type::Int { return; }
                self.insert(lhs.name.clone(), IntConstAbsVal::Top);
            }
            Instruction::Store { dst, op } => unsafe {
                if dst.typ != Type::Int { return; }
                let mut tmp_store = AbstractStore::new();
                let op = self.resolve_operand(op);
                ADDR_TAKEN_INTS.iter().for_each(|v| {
                    if v.name == dst.name {
                        tmp_store.insert(v.name.clone(), op.clone());
                    }
                });
                self.join(&tmp_store);
            }
            Instruction::CallExt { lhs, args, .. } => {
                solve_call(lhs, args, self);
            }
            Instruction::AddrOf { .. } |
            Instruction::Alloc { .. } |
            Instruction::Gep { .. } |
            Instruction::Gfp { .. } => {}
            // _ => {
            //     let info = format!("unsupported instruction: {:?}", inst);
            //     panic!("{info}");
            // }
        }
    }
}

fn solve_call(lhs: &Option<Variable>, args: &Vec<Operand>, store: &mut AbstractStore) {
    unsafe {
        GLOBAL_INTS.iter().for_each(|v| {
            store.insert(v.name.clone(), IntConstAbsVal::Top);
        });
        if let Some(lhs) = lhs {
            if lhs.typ == Type::Int {
                store.insert(lhs.name.clone(), IntConstAbsVal::Top);
            }
        }
        let any_arg_reaches_int = args.iter().any(|arg| {
            if let Operand::Var(v) = arg {
                v.typ.is_pointer_to_int()
            } else {
                false
            }
        });
        if any_arg_reaches_int || GLOBAL_PTR_TO_INTS {
            ADDR_TAKEN_INTS.iter().for_each(|v| {
                store.insert(v.name.clone(), IntConstAbsVal::Top);
            });
        }
    }
}
// working list algorithm for int const analysis
fn int_const_analysis(program: &Program, function_name: &str) -> HashMap<String, AbstractStore> {
    let function = program.functions.get(function_name).unwrap();
    let mut bb2store: HashMap<String, AbstractStore> = HashMap::new();
    let mut bb2store_post: HashMap<String, AbstractStore> = HashMap::new();
    let mut working_list = VecDeque::new();
    working_list.push_back("entry".to_string());

    let mut initial_store = AbstractStore::new();
    // init global ints as top
    program.globals.iter().for_each(|g| {
        if g.typ == Type::Int {
            initial_store.insert(g.name.clone(), IntConstAbsVal::Top);
        }
    });
    // init parameters as top
    function.params.iter().for_each(|p| {
        if p.typ == Type::Int {
            initial_store.insert(p.name.clone(), IntConstAbsVal::Top);
        }
    });
    bb2store.insert("entry".to_string(), initial_store);

    while !working_list.is_empty() {
        let bb_name = working_list.pop_front().unwrap();
        let bb = function.body.get(&bb_name).unwrap();
        let mut current_store = bb2store.entry(bb_name.clone()).or_insert(
            AbstractStore::new(),
        ).clone();
        // println!("working on {}", bb_name);
        // execute the instructions in the basic block to update the store
        bb.insts.iter().for_each(|inst| current_store.execute(inst));

        // work on terminals
        // collect the target basic blocks
        let target_bb: Vec<String> = match &bb.term {
            Terminal::Branch { tt, ff, cond } => {
                let cond = current_store.resolve_operand(cond);
                match cond {
                    IntConstAbsVal::IntConst(0) => vec![ff.clone()],
                    IntConstAbsVal::IntConst(_) => vec![tt.clone()],
                    IntConstAbsVal::Bottom => vec![],
                    IntConstAbsVal::Top => vec![tt.clone(), ff.clone()],
                }
            }
            Terminal::Jump(target) => vec![target.clone()],
            Terminal::CallDirect { next_bb, lhs, args,  .. } |
            Terminal::CallIndirect { next_bb, lhs, args, .. } => {
                solve_call(lhs, args, &mut current_store);
                vec![next_bb.clone()]
            },
            Terminal::Ret(_) => vec![],
        };

        bb2store_post.insert(bb_name.clone(), current_store.clone());

        // join the current store with the store of the target basic blocks
        // if changed then add the target basic blocks to the working list
        for target in target_bb.iter() { // or just `for target in &target_bb` to borrow each String
            let changed =
                bb2store.entry(target.clone()).or_insert_with(|| AbstractStore::new()).join(&current_store);
            if changed {
                working_list.push_back(target.clone());
            }
        }
    }

    bb2store_post
}


fn print_store(store: &HashMap<String, AbstractStore>) {
    let mut blocks: Vec<&String> = store.keys().collect();
    blocks.sort();
    for block in blocks {
        println!("{}:", block);
        // print the store after sorting the keys
        let mut keys: Vec<&String> = store[block].store.keys().collect();
        keys.sort();
        for key in keys {
            let val = store[block].store[key].as_string();
            if val != "Bottom" {
                println!("{} -> {}", key, val);
            }
        }
        println!();
    }
}

static mut GLOBAL_INTS: Vec<Variable> = vec![];
static mut ADDR_TAKEN_INTS: Vec<Variable> = vec![];
static mut GLOBAL_PTR_TO_INTS: bool = false;
unsafe fn global_init(program: &Program, function_name: &str) {
    // collect all global variables of type int
    program.globals.iter().for_each(|g| {
        if g.typ == Type::Int {
            GLOBAL_INTS.push(g.clone());
        }
    });
    // collect all variables that are taken the address of
    program.functions.get(function_name).unwrap().body.iter().for_each(|(_, bb)| {
        bb.insts.iter().for_each(|inst| {
            match inst {
                Instruction::AddrOf { lhs:_, rhs } => {
                    if rhs.typ == Type::Int {
                        unsafe {
                            ADDR_TAKEN_INTS.push(rhs.clone());
                        }
                    }
                }
                _ => {}
            }
        });
    });
    // check if some global variables are pointers that reach int
    program.globals.iter().for_each(|g| {
        if g.typ.is_pointer_to_int() {
            unsafe {
                GLOBAL_PTR_TO_INTS = true;
            }
        }
    });
    // remove duplicates of addr_taken_ints
    ADDR_TAKEN_INTS.sort();
    ADDR_TAKEN_INTS.dedup();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // two argument: the first is the file path, the second is the name of function to analysis
    if args.len() != 3 {
        eprintln!("Usage: {} <file> <function>", args[0]);
        exit(1);
    }
    let function_name = &args[2];
    // read the file into a string
    let file_path = &args[1];
    let file_content = fs::read_to_string(file_path).unwrap();
    // parse the string into a Program
    let program = Program::parse_json(&file_content);
    unsafe { global_init(&program, function_name); }
    // let store = int_const_analysis(&program, function_name);
    //
    // print_store(&store);
}
