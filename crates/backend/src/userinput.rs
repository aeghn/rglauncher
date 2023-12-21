use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;

#[derive(Clone, Debug)]
pub struct UserInput {
    pub window_id: i32,
    pub input: String,
    cancel_signal: Arc<AtomicBool>
}
impl PartialEq for UserInput {
    fn eq(&self, other: &Self) -> bool {
        self.window_id == other.window_id && self.input == other.input
    }
}

impl UserInput {
    pub fn new(input: &str) -> Self {
        UserInput {
            window_id: 0,
            input: input.to_string(),
            cancel_signal: Arc::new(AtomicBool::from(false)),
        }
    }

    pub fn cancelled(&self) -> bool {
        return self.cancel_signal.load(SeqCst)
    }

    pub fn cancel(&mut self) {
        self.cancel_signal.store(true, SeqCst)
    }
}
