pub fn split_cmd_to_args(input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_word = String::new();
    let mut in_quotes = false;
    let mut in_single_quotes = false;
    let mut escape_next = false;

    for c in input.chars() {
        if escape_next {
            current_word.push(c);
            escape_next = false;
            continue;
        }

        match c {
            ' ' if !in_quotes && !in_single_quotes => {
                if !current_word.is_empty() {
                    result.push(current_word.clone());
                    current_word.clear();
                }
            }
            '\\' => {
                escape_next = true;
            }
            '"' => {
                if !in_single_quotes {
                    if in_quotes {
                        result.push(current_word.clone());
                        current_word.clear();
                    }
                    in_quotes = !in_quotes;
                } else {
                    current_word.push(c);
                }
            }
            '\'' => {
                if !in_quotes {
                    if in_single_quotes {
                        result.push(current_word.clone());
                        current_word.clear();
                    }
                    in_single_quotes = !in_single_quotes;
                } else {
                    current_word.push(c);
                }
            }
            _ => {
                current_word.push(c);
            }
        }
    }

    if !current_word.is_empty() {
        result.push(current_word);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let vvv =
            |vec: Vec<&'static str>| -> Vec<String> { vec.iter().map(|s| s.to_string()).collect() };

        // assert_eq!(parse_cmd_string("who are you"), vvv(vec!["who","are","you"]));
        assert_eq!(
            split_cmd_to_args("who \"are\" you"),
            vvv(vec!["who", "are", "you"])
        );
        assert_eq!(
            split_cmd_to_args("who 'are' you"),
            vvv(vec!["who", "are", "you"])
        );
        assert_eq!(
            split_cmd_to_args("who 'a\"re' you"),
            vvv(vec!["who", "a\"re", "you"])
        );
        assert_eq!(
            split_cmd_to_args("who 'a\"r\\\\e' you"),
            vvv(vec!["who", "a\"r\\e", "you"])
        );
    }
}
