use std::{fs, error::Error, env};
use bf_rs::*;
use clap::{Parser, ValueEnum};


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    filepath: String,

    #[arg(short, long, value_enum, default_value_t=DebugMode::None)]
    debug_mode: DebugMode,
    
    /// Enable breakpoints
    #[arg(short='b', long)]
    breakpoints: bool,

    /// Enable macros
    #[arg(short='m', long)]
    macros: bool,
}


#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum DebugMode {
    None,
    
    /// Print memory and instructions
    Verbose,
    
    /// Verbose, but pause at every instruction
    Step,
}


fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    
    let filepath = args.filepath;
    let code_string = fs::read_to_string(filepath)?;
    
    run(&code_string)?;
    
    Ok(())
}
