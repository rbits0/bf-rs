use std::{error::Error, io::{self, Read}};

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


pub fn parse_string(code: &str) -> Vec<Instruction> {
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

pub fn run(code: &str) -> Result<(), Box<dyn Error>>{
    let instructions = parse_string(code);
    // Location of the instruction pointer
    let mut i: usize = 0;
    // Location of the data pointer
    let mut pointer: usize = 0;
    // Using u8, but max will be 127
    let mut data: Vec<u8> = vec![0];
    
    while i < instructions.len() {
        match instructions[i] {
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
                pointer.saturating_sub(1);
            },
            Instruction::Right => {
                pointer += 1;
                if pointer >= data.len() {
                    data.push(0);
                }
            },
            Instruction::Open => {
                let remaining_instructions = &instructions[(pointer + 1)..];
                pointer = find_matching_bracket(remaining_instructions, true)? + pointer + 1;
            },
            Instruction::Close => {
                let remaining_instructions = &instructions[..pointer];
                pointer = find_matching_bracket(remaining_instructions, false)?;
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

        panic!("reached end of loop");
    }
    
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