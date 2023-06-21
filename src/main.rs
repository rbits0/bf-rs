use std::{fs, error::Error};
use bf_rs::*;
use clap::{Parser, ValueEnum};




fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    
    let filepath = args.filepath;
    let code_string = fs::read_to_string(filepath)?;
    
    run(&code_string, args.breakpoints, args.debug_mode)?;
    
    Ok(())
}
