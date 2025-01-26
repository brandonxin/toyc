use std::env;
use std::fs::File;

use clap::Parser;

mod aarch64;
mod ast;
mod frontend;
mod ir;

/// Yet Another Toy Compiler
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// file...
    files: Vec<String>,

    #[arg(long = "dump-ir")]
    /// Dump intermediate representation only
    dump_ir: bool,

    #[arg(long = "no-regalloc")]
    /// Disable register allocation
    no_regalloc: bool,
}

fn get_exec_name() -> String {
    match env::current_exe() {
        Ok(path) => path.file_name().unwrap().to_str().unwrap().to_string(),
        Err(_) => "a.out".to_string(),
    }
}

fn compile(opt: &Args, file: &str) {
    let src = File::open(file).unwrap();

    let mut parser = frontend::Parser::<frontend::Utf8Decoder<_>, _>::new(src);
    let mut unit = ast::Module::new();
    parser.parse(&mut unit);

    let ir_module = ir::Module::new();
    let ir_codegen = ir::Codegen::new(&ir_module);
    ir_codegen.visit_unit(&unit);

    if opt.dump_ir {
        let out = File::create(format!("{}.ir", file)).unwrap();
        let mut out = std::io::BufWriter::new(out);
        ir_module.dump(&mut out).unwrap();
        return;
    }

    let aarch64_module = aarch64::Module::new();
    let mut codegen = aarch64::Codegen::new(&aarch64_module);
    codegen.visit_unit(&ir_module, opt.no_regalloc);

    let out = File::create(format!("{}.s", file)).unwrap();
    let mut out = std::io::BufWriter::new(out);
    codegen.unit().dump(&mut out).unwrap();
}

fn main() {
    let cli = Args::parse();

    if cli.files.is_empty() {
        println!("{}: error: no input files", get_exec_name());
        return;
    }

    for file in cli.files.iter() {
        compile(&cli, file);
    }
}
