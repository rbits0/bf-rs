use std::{error::Error, io::{self, Read}, collections::HashMap};
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
    Break,
}


const VALID_CHARS: [char; 8]= ['[', ']', '<', '>', '+', '-', '.', ','];


fn parse_string(code: &str, breakpoints: bool) -> Vec<Instruction> {
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
            '@' => if breakpoints { Some(Instruction::Break) } else { None },
            _ => None,
        }
    }).collect()
}

pub fn parse_string_macros(code: &str, breakpoints: bool) -> Result<Vec<Instruction>, Box<dyn Error>> {
    // Process brackets first

    let mut split_string: Vec<String> = Vec::new();
    let mut remaining_string = code;
    let mut macro_strings: HashMap<String, String> = HashMap::new();
    
    while !remaining_string.is_empty() {
        match remaining_string.find('{') {
            Some(i) => {
                split_string.push(remaining_string[..i].to_string());
                remaining_string = &remaining_string[(i + 1)..];
                
                // Find closing bracket
                let Some(close_index) = remaining_string.find('}') else {
                    return Err("all curly brackets must be matched".into());
                };
                let macro_string = &remaining_string[..close_index];
                if macro_string.contains('{') {
                    return Err("macros in macros are not allowed".into());
                }
                
                // Find macro name
                let Some(macro_name) = split_string.last() else {
                    return Err("macros must have a name".into());
                };
                let Some(macro_name) = macro_name.split_whitespace().last() else {
                    return Err("macros must have a name".into());
                };
                
                // If macro_name contains any instruction, error
                if VALID_CHARS.iter().any(|c| macro_name.contains(*c)) {
                    return Err("macro name cannot contain instructions".into());
                }
                
                // @macro_name<space> so I can easily find and replace
                macro_strings.insert("@".to_string() + macro_name + "@", remaining_string[..close_index].to_string());
                remaining_string = &remaining_string[(close_index + 1)..]
            },
            None => {
                if remaining_string.contains('}') {
                    return Err("all curly brackets must be matched".into());
                }
                split_string.push(remaining_string.to_string());
                break;
            }
        }
    }
    

    // Replace all macro calls with the macro code
    // First do the macros
    let mut remaining_macros = macro_strings.clone();
    let mut processed_macros: HashMap<String, String> = HashMap::new();
    while !remaining_macros.is_empty() {
        let macro_names: Vec<&String> = remaining_macros.keys().collect();

        // Process the ones that don't call unprocessed macros
        // If all the unprocessed macros call another unprocessed macro,
        // that means they're recursive
        let mut to_remove: Vec<String> = Vec::new();
        for (macro_name, macro_code) in remaining_macros.iter().filter(
            |(_, macro_code)| macro_names.iter().all(|name| !macro_code.contains(*name))
        ) {
            let mut new_code = macro_code.to_string();
            for (macro_name2, macro_code2) in &processed_macros {
                new_code = new_code.replace(macro_name2, macro_code2);
            }
            
            processed_macros.insert(macro_name.to_string(), new_code);
            to_remove.push(macro_name.to_string())
        }
        
        if to_remove.is_empty() {
            return Err("recursive macros are not allowed".into());
        }
        
        for macro_name in to_remove {
            remaining_macros.remove(&macro_name);
        }
    }

    // Then do the non-macro code
    for (macro_name, macro_string) in processed_macros {
        for code_string in &mut split_string {
            *code_string = code_string.replace(&macro_name, &macro_string);
        }
    }
    

    // Parse all strings, and join them into one vec
    Ok(split_string.iter().flat_map(|s| parse_string(s, breakpoints)).collect())
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
        Instruction::Output => '.',
        Instruction::Break => '@',
    }
}

pub fn run(code: &str, breakpoints: bool, macros: bool, debug_mode: DebugMode) -> Result<(), Box<dyn Error>> {
    let instructions = {
        if macros {
            parse_string_macros(code, breakpoints)?
        } else {
            parse_string(code, breakpoints)
        }
    };

    // Location of the instruction pointer
    let mut i: usize = 0;
    // Location of the data pointer
    let mut pointer: usize = 0;
    // Using u8, but max will be 127
    let mut data: Vec<u8> = vec![0];
    
    while i < instructions.len() {
        let instruction = &instructions[i];
        let mut is_break = false;

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
                println!();

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
            },
            Instruction::Break => {
                is_break = true;
            }
        }
        

        if (debug_mode == DebugMode::Step || debug_mode == DebugMode::Verbose) && !is_break {
            print!("{}:", instruction_to_char(instruction));
            for x in &data {
                print!(" {}", x);
            }
            println!();
        }
        
        if debug_mode == DebugMode::Step || is_break {
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
            parse_string("a<+<c<]", true),
            vec![Instruction::Left, Instruction::Increment, Instruction::Left, Instruction::Left, Instruction::Close]
        );
    }
    
    #[test]
    fn forward_match() {
        assert_eq!(
            Ok(5),
            find_matching_bracket(&parse_string("-[-]-]]--", true), true)
        );
    }
    
    #[test]
    fn backward_match() {
        assert_eq!(
            Ok(3),
            find_matching_bracket(&parse_string("--[[-[-]-", true), false)
        );
    }
    
    #[test]
    fn macro_parse_test() {
        use Instruction::*;
        let code = ">+,@test@.@test@,
test {
    [+<]
}";
        let instructions = vec![
            Right,
            Increment,
            Input,
            Open,
            Increment,
            Left,
            Close,
            Output,
            Open,
            Increment,
            Left,
            Close,
            Input
        ];
        
        assert_eq!(parse_string_macros(code, true).unwrap(), instructions)
    }

    #[test]
    fn break_parse_test() {
        use Instruction::*;

        let code = "<>@+@abc++@ -";
        let instructions = vec![Left, Right, Break, Increment, Break, Increment, Increment, Break, Decrement];

        assert_eq!(parse_string_macros(code, true).unwrap(), instructions);
    }

    #[test]
    fn break_parse_disabled() {
        use Instruction::*;

        let code = "<>@+@abc++@ -";
        let instructions = vec![Left, Right, Increment, Increment, Increment, Decrement];

        assert_eq!(parse_string_macros(code, false).unwrap(), instructions);
    }

    #[test]
    fn macro_calls_macro() {
        use Instruction::*;

        let code = "@a@ .+ @b@
a {
    + @b@
}
b {
    -
}";
        let instructions = vec![
            Increment,
            Decrement,
            Output,
            Increment,
            Decrement,
        ];

        // dbg!(parse_string_macros(code, true).unwrap());

        assert_eq!(parse_string_macros(code, true).unwrap(), instructions)
    }
}