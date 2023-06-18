use std::{env, fs, error::Error};
use bf_rs::*;

fn main() -> Result<(), Box<dyn Error>> {
    let Some(filepath) = env::args().nth(1) else {
        return Err("A");
    };
    let code_string = fs::read_to_string(filepath);

    run(&code_string);
    
    Ok(())
}
