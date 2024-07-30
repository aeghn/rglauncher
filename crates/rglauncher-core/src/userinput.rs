use std::{
    cell::RefCell,
    sync::{Arc, RwLock},
};

#[derive(Clone, Debug)]
pub struct UserInput {
    pub input: String,
    cancel_signal: Arc<RwLock<bool>>,
}
impl PartialEq for UserInput {
    fn eq(&self, other: &Self) -> bool {
        self.input == other.input
    }
}

impl UserInput {
    pub fn new(input: &str) -> Self {
        UserInput {
            input: input.to_string(),
            cancel_signal: Arc::new(RwLock::new(false)),
        }
    }

    pub fn cancelled(&self) -> bool {
        *self.cancel_signal.read().unwrap()
    }

    pub fn cancel(&self) {
        if let Ok(mut cancel_signal) = self.cancel_signal.write() {
            *cancel_signal = true;
        }
    }
}
