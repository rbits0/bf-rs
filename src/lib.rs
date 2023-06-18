const VALID_CHARS = ['a', 'b',];


fn remove_comments(code: &str) -> String {
    code.chars().filter(|char| ).collect()
}

fn run(code: &str) {

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_comments_test() {
        assert_eq!("<<<", remove_comments("a<b<c<d"))
    }
}