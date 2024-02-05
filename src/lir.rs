use std::collections::HashMap;
// use std::fs::File;
use serde::{Deserialize, Serialize};
use serde_json as json;
use crate::stats::Stats;
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Debug)]
pub struct Program {
    pub structs: HashMap<String, Vec<Field>>,
    pub globals: Vec<Variable>,
    pub functions: HashMap<String, Function>, // function definitions
    pub externs: HashMap<String, Type>,       // external function declarations
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone)]
pub struct Field {
    name: String,
    typ: Type,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone)]
pub enum Type {
    Int,                         // "Int"
    Struct(String),              // {"Struct": "xxx"}
    Function(Box<FunctionType>), // {"Function": "xxx"}
    Pointer(Box<Type>),          // {"Pointer": "xxx"}
}

impl Type {
    // check if the type is can finally reach local int as pointer or struct with pointer
    pub fn is_pointer_to_int_helper(&self, has_pt_before: bool) -> bool {
        match self {
            Type::Pointer(t) => {
                match **t {
                    Type::Int => true,
                    _ => t.is_pointer_to_int_helper(true),
                    // Type::Struct(t) => has_pt_before,
                    // _ => false,
                }
            }
            Type::Int => has_pt_before,
            // todo: adpt struct
            // Type::Struct(s) => has_pt_before,
            _ => false,
        }
    }

    pub fn is_pointer_to_int(&self) -> bool {
        self.is_pointer_to_int_helper(false)
    }

}

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, )]
pub struct Variable {
    // it could be as parameter, local variable, or global variable
    pub name: String,
    pub typ: Type,
    pub scope: Option<String>,
}

impl PartialOrd for Variable {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl Ord for Variable {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Function {
    pub id: String,
    pub ret_ty: Option<Type>,
    pub params: Vec<Variable>,
    pub locals: Vec<Variable>,
    pub body: HashMap<String, Block>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone)]
pub struct FunctionType {
    pub ret_ty: Option<Type>,
    pub param_ty: Vec<Type>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone)]
pub struct Block {
    /*
     {
        "id": "bb12",
        "insts": [],
        "term": {...}
    },
     */
    pub id: String,
    pub insts: Vec<Instruction>,
    pub term: Terminal,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone)]
pub enum Instruction {
    // 10 kinds of instructions
    AddrOf {
        // {"AddrOf": {"lhs": "xxx", "rhs": "xxx"}}
        lhs: Variable,
        rhs: Variable,
    },
    Alloc {
        // {"Alloc": {"lhs": "xxx", "num": "xxx", "id": "xxx"}}
        lhs: Variable,
        num: Operand,
        id: Variable,
    },
    Copy {
        // {"Copy": {"lhs": "xxx", "op": "xxx"}}
        lhs: Variable,
        op: Operand,
    },
    Gep {
        // get-element-pointer, {"Gep": {"lhs": "xxx", "src": "xxx", "idx": "xxx"}}
        lhs: Variable,
        src: Variable,
        idx: Operand,
    },
    Arith {
        // {"Arith": {"lhs": "xxx", "aop": "xxx", "op1": "xxx", "op2": "xxx"}}
        lhs: Variable,
        aop: ArithOp,
        op1: Operand,
        op2: Operand,
    }, //
    Load {
        // {"Load": {"lhs": "xxx", "src": "xxx"}
        lhs: Variable,
        src: Variable,
    },
    Store {
        // {"Store": {"dst": "xxx", "op": "xxx"}}
        dst: Variable,
        op: Operand,
    },
    Gfp {
        // {"Gfp": {"lhs": "xxx", "src": "xxx", "field": "xxx"}}
        lhs: Variable,
        src: Variable,
        field: Variable,
    },
    Cmp {
        // {"Cmp": {"lhs": "xxx", "rop": "xxx", "op1": "xxx", "op2": "xxx"}}
        lhs: Variable,
        rop: RelaOp,
        op1: Operand,
        op2: Operand,
    },
    CallExt {
        // {"CallExt": {"lhs": "xxx", "ext_callee": "xxx", "args": ["xxx", "xxx"]}}
        lhs: Option<Variable>,
        ext_callee: String,
        args: Vec<Operand>,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone)]
pub enum ArithOp {
    // arithmetic operators
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone)]
pub enum RelaOp {
    // relational operators
    Neq,
    Eq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone)]
pub enum Terminal {
    // a terminal signals the end of a basic block and is one of
    Jump(String), // {"Jump": "bb1"}
    Branch {
        // {"Branch": {"cond": "xxx", "tt": "xxx", "ff": "xxx"}}
        cond: Operand,
        tt: String,
        ff: String,
    },
    Ret(Option<Operand>), // {"Ret": "xxx"}
    CallDirect {
        // {"CallDirect": {"lhs": "xxx", "callee": "xxx", "args": ["xxx", "xxx"], "next_bb": "xxx"}}
        lhs: Option<Variable>,
        callee: String,
        args: Vec<Operand>,
        next_bb: String,
    },
    CallIndirect {
        // {"CallIndirect": {"lhs": "xxx", "callee": "xxx", "args": ["xxx", "xxx"], "next_bb": "xxx"}}
        lhs: Option<Variable>,
        callee: Variable,
        args: Vec<Operand>,
        next_bb: String,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone)]
pub enum Operand {
    // an operand is either a variable or a constant
    Var(Variable),
    CInt(i32),
}

impl Program {
    pub fn new() -> Program {
        Program {
            structs: HashMap::new(),
            globals: Vec::new(),
            functions: HashMap::new(),
            externs: HashMap::new(),
        }
    }

    pub fn parse_json(string: &str) -> Program {
        json::from_str(string).unwrap()
    }

    pub fn as_json(&self) -> String {
        json::to_string(&self).unwrap()
    }

    pub fn get_stats(&self) -> Stats {
        let mut stats = Stats::new();

        stats.field_num = self.structs.values()
            .map(|s| s.len())
            .sum::<usize>() as u32;

        stats.function_returning_value_num = self.functions.values()
            .filter(|f| f.ret_ty.is_some())
            .count() as u32;

        stats.func_param_num = self.functions.values()
            .map(|f| f.params.len())
            .sum::<usize>() as u32;

        stats.local_var_num = self.functions.values()
            .map(|f| f.locals.len())
            .sum::<usize>() as u32;

        stats.block_num = self.functions.values()
            .map(|f| f.body.len())
            .sum::<usize>() as u32;

        stats.instr_num = self.functions.values()
            .flat_map(|f| f.body.values())
            .map(|b| b.insts.len())
            .sum::<usize>() as u32;

        stats.terminal_num = self.functions.values()
            .flat_map(|f| f.body.values())
            .count() as u32;


        stats.int_type_num = //self.get_local_num_by_type(Type::Int);
            self.functions.values()
                .flat_map(|f| &f.locals)
                .filter(|v| matches!(&v.typ, Type::Int))
                .count() as u32;

        stats.int_type_num += self.globals.iter()
            .filter(|v| matches!(v.typ, Type::Int))
            .count() as u32;

        stats.struct_type_num = self.functions.values()
                .flat_map(|f| &f.locals)
                .filter(|v| matches!(&v.typ, Type::Struct(_)))
                .count() as u32;

        stats.struct_type_num += self.globals.iter()
            .filter(|v| matches!(v.typ, Type::Struct(_)))
            .count() as u32;

        stats.pointer_to_int_num = self.functions.values()
            .flat_map(|f| &f.locals)
            .filter(|v| match &v.typ {
                Type::Pointer(t) => matches!(**t, Type::Int),
                _ => false,
            })
            .count() as u32;

        stats.pointer_to_int_num += self.globals.iter()
            .filter(|v| match &v.typ {
                Type::Pointer(t) => matches!(**t, Type::Int),
                _ => false,
            })
            .count() as u32;

        stats.pointer_to_struct_num = self.functions.values()
            .flat_map(|f| &f.locals)
            .filter(|v| match &v.typ {
                Type::Pointer(t) => matches!(**t, Type::Struct(_)),
                _ => false,
            })
            .count() as u32;

        stats.pointer_to_struct_num += self.globals.iter()
            .filter(|v| match &v.typ {
                Type::Pointer(t) => matches!(**t, Type::Struct(_)),
                _ => false,
            })
            .count() as u32;

        stats.pointer_to_function_num = self.functions.values()
            .flat_map(|f| &f.locals)
            .filter(|v| match &v.typ {
                Type::Pointer(t) => matches!(**t, Type::Function(_)),
                _ => false,
            })
            .count() as u32;

        stats.pointer_to_function_num += self.globals.iter()
            .filter(|v| match &v.typ {
                Type::Pointer(t) => matches!(**t, Type::Function(_)),
                _ => false,
            })
            .count() as u32;

        stats.pointer_to_pointer_num = self.functions.values()
            .flat_map(|f| &f.locals)
            .filter(|v| match &v.typ {
                Type::Pointer(t) => matches!(**t, Type::Pointer(_)),
                _ => false,
            })
            .count() as u32;

        stats.pointer_to_pointer_num += self.globals.iter()
            .filter(|v| match &v.typ {
                Type::Pointer(t) => matches!(**t, Type::Pointer(_)),
                _ => false,
            })
            .count() as u32;

        stats
    }
    // fn get_local_num_by_type(&self, typ: Type) -> u32 {
    //     self.functions.values()
    //         .flat_map(|f| &f.locals)
    //         .filter(|v| matches!(&v.typ, typ))
    //         .count() as u32
    // }
    // fn get_global_num_by_type(&self, typ: Type) -> u32 {
    //     self.globals.iter()
    //         .filter(|v| matches!(&v.typ, typ))
    //         .count() as u32
    // }
}