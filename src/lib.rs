#[derive(Debug, PartialEq)]
enum Instruction {
    Increment,
    Decrement,
    Left,
    Right,
    Open,
    Close,
    Input,
    Output,
}


fn parse_string(code: &str) -> Vec<Instruction> {
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

fn run(code: &str) {

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_string_test() {
        assert_eq!(
            vec![Instruction::Left, Instruction::Increment, Instruction::Left, Instruction::Left, Instruction::Close],
            parse_string("a<+<c<]")
        );
    }
}