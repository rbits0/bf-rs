use std::{fs, error::Error};
use bf_rs::*;
use clap::Parser;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    pub filepath: String,

    #[arg(short, long, value_enum, default_value_t=DebugMode::None)]
    pub debug_mode: DebugMode,
    
    /// Enable breakpoints
    #[arg(short='b', long)]
    pub breakpoints: bool,

    /// Enable macros
    #[arg(short='m', long)]
    pub macros: bool,
}



fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    
    let filepath = args.filepath;
    let code_string = fs::read_to_string(filepath)?;
    
    run(&code_string, args.breakpoints, args.macros, args.debug_mode)?;
    
    Ok(())
}
