#[derive(Clone, Debug)]
pub struct UserInput {
    pub input: String,
}

impl UserInput {
    pub fn new(input: &str) -> Self {
        UserInput {
            input: input.to_string(),
        }
    }
}
