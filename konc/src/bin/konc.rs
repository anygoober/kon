use std::{error::Error, fs, path::PathBuf, process::exit};

use bumpalo::Bump;
use konc::parser as konc_parser;
use pico_args::Arguments;

#[cfg(false)]
fn create_module() {
    use qbe::{Function, Instr, Linkage, Module, Type, Value};
    let mut module = Module::new();

    let mut func = Function::new(
        Linkage::private(),
        "add",
        vec![
            (Type::Word, Value::Temporary("a".to_string())),
            (Type::Word, Value::Temporary("b".to_string())),
        ],
        Some(Type::Word),
    );

    let block = func.add_block("start");

    block.assign_instr(
        Value::Temporary("sum".to_string()),
        Type::Word,
        Instr::Add(
            Value::Temporary("a".to_string()),
            Value::Temporary("b".to_string()),
        ),
    );

    block.add_instr(Instr::Ret(Some(Value::Temporary("sum".to_string()))));
    module.add_function(func);

    println!("{}", module);
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        println!("usage: konc <path>");
        exit(0);
    }

    let file: PathBuf = args.free_from_str()?;
    let content = fs::read_to_string(file)?;

    let bump = Bump::new();
    let ast = konc_parser::parse(&content, &bump);

    println!("{ast:#?}");

    Ok(())
}
