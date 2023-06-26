use std::{error::Error, io::{self, Read}};
use clap::{Parser, ValueEnum};


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


#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum DebugMode {
    None,
    
    /// Print memory and instructions
    Verbose,
    
    /// Verbose, but pause at every instruction
    Step,
}



#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    Increment,
    Decrement,
    Left,
    Right,
    Open,
    Close,
    Input,
    Output,
}


const valid_chars: Vec<char> = vec!['[', ']', '<', '>', '+', '-', '.', ','];


fn parse_string_basic(code: &str) -> Vec<Instruction> {
    // Split at '@' so I can see whether they are macros or breakpoints
    code.chars().filter_map({
        |x| match x {
            '+' => Some(Instruction::Increment),
            '-' => Some(Instruction::Decrement),
            '<' => Some(Instruction::Left),
            '>' => Some(Instruction::Right),
            '[' => Some(Instruction::Open),
            ']' => Some(Instruction::Close),
            ',' => Some(Instruction::Input),
            '.' => Some(Instruction::Output),
            _ => None,
        }
    }).collect()
}

pub fn parse_string(code: &str) -> Result<Vec<Instruction>, Box<dyn Error>> {
    // Process brackets first
    let mut split_string: Vec<&str> = Vec::new();
    let mut remaining_string = code;
    
    while remaining_string.len() > 0 {
        match remaining_string.find('{') {
            Some(i) => {
                split_string.push(&remaining_string[..i]);
                remaining_string = &remaining_string[(i + 1)..];
                
                let Some(close_index) = remaining_string.find('}') else {
                    return Err("all curly brackets must be matched".into());
                };
                let macro_string = &remaining_string[..close_index];
                if macro_string.contains('{') {
                    return Err("macros in macros are not allowed".into());
                }
                
                let Some(macro_name) = split_string.last() else {
                    return Err("macros must have a name".into());
                };
                let Some(macro_name) = macro_name.split_whitespace().last() else {
                    return Err("macros must have a name".into());
                };
                
                // If macro_name contains anything in valid_chars
                if valid_chars.iter().any(|c| macro_name.contains(*c)) {
                    return Err("macro name cannot contain instructions".into());
                }
            }
        }

    }


    Vec::new()
}


pub fn instruction_to_char(instruction: &Instruction) -> char {
    match instruction {
        Instruction::Increment => '+',
        Instruction::Decrement => '-',
        Instruction::Left => '<',
        Instruction::Right => '>',
        Instruction::Open => '[',
        Instruction::Close => ']',
        Instruction::Input => ',',
        Instruction::Output => '.'
    }
}

pub fn run(code: &str, breakpoints: bool, debug_mode: DebugMode) -> Result<(), Box<dyn Error>> {
    let instructions = parse_string(code)?;
    // Location of the instruction pointer
    let mut i: usize = 0;
    // Location of the data pointer
    let mut pointer: usize = 0;
    // Using u8, but max will be 127
    let mut data: Vec<u8> = vec![0];
    
    while i < instructions.len() {
        let instruction = &instructions[i];

        match instruction {
            Instruction::Increment => {
                if data[pointer] == 127 {
                    data[pointer] = 0;
                } else {
                    data[pointer] += 1;
                }
            },
            Instruction::Decrement => {
                if data[pointer] == 0 {
                    data[pointer] = 127;
                } else {
                    data[pointer] -= 1;
                }
            },
            Instruction::Left => {
                pointer = pointer.saturating_sub(1);
            },
            Instruction::Right => {
                pointer += 1;
                if pointer >= data.len() {
                    data.push(0);
                }
            },
            Instruction::Open => {
                if data[pointer] == 0 {
                    let remaining_instructions = &instructions[(i + 1)..];
                    i = find_matching_bracket(remaining_instructions, true)? + i + 1;
                }
            },
            Instruction::Close => {
                if data[pointer] != 0 {
                    let remaining_instructions = &instructions[..i];
                    i = find_matching_bracket(remaining_instructions, false)?;
                }
            },
            Instruction::Input => {
                let mut input: [u8; 1] = [0];
                io::stdin().read(&mut input).map_err(|_| "failed to read input")?;
                if input[0] > 127 {
                    return Err("input must be an ASCII character".into());
                }
                data[pointer] = input[0];
            },
            Instruction::Output => {
                let output = data[pointer] as char;
                print!("{}", output);
            }
        }
        

        if debug_mode == DebugMode::Step || debug_mode == DebugMode::Verbose {
            print!("{}:", instruction_to_char(instruction));
            for x in &data {
                print!(" {}", x);
            }
            println!();
        }
        
        if debug_mode == DebugMode::Step {
            io::stdin().read_line(&mut String::new())?;
        }

        i += 1;
    }
    
    println!();
    Ok(())
}


pub fn find_matching_bracket(instructions: &[Instruction], forwards: bool) -> Result<usize, &'static str> {
    let mut level = 0;
    if forwards {
        for (i, instruction) in instructions.iter().enumerate() {
            match instruction {
                Instruction::Open => {
                    level += 1;
                },
                Instruction::Close => {
                    if level > 0 {
                        level -= 1
                    } else {
                        return Ok(i);
                    }
                },
                _ => (),
            }
        }
    } else {
        for (i, instruction) in instructions.iter().enumerate().rev() {
            match instruction {
                Instruction::Open => {
                    if level > 0 {
                        level -= 1
                    } else {
                        return Ok(i);
                    }
                },
                Instruction::Close => {
                    level += 1;
                },
                _ => (),
            }
        }
    }
    
    Err("all brackets must have matching brackets")
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_string_test() {
        assert_eq!(
            parse_string("a<+<c<]"),
            vec![Instruction::Left, Instruction::Increment, Instruction::Left, Instruction::Left, Instruction::Close]
        );
    }
    
    #[test]
    fn forward_match() {
        assert_eq!(
            Ok(5),
            find_matching_bracket(&parse_string("-[-]-]]--"), true)
        );
    }
    
    #[test]
    fn backward_match() {
        assert_eq!(
            Ok(3),
            find_matching_bracket(&parse_string("--[[-[-]-"), false)
        );
    }
}