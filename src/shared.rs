pub struct UserInput {
    pub input: String
}

impl UserInput {
    pub fn new(input: &str) -> Self {
        UserInput{input: input.to_string()}
    }

    pub fn clone(&self) -> Self {
        UserInput{
            input: self.input.to_string()
        }
    }
}