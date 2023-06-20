use std::{env, fs, error::Error};
use bf_rs::*;

fn main() -> Result<(), Box<dyn Error>> {
    let filepath = env::args().nth(1).ok_or("missing file path argument")?;
    let code_string = fs::read_to_string(filepath)?;

    run(&code_string);
    
    Ok(())
}
