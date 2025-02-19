use std::sync::{atomic::AtomicUsize, Arc};

use chin_tools::SharedStr;

#[derive(Clone, Debug)]
pub struct Signal {
    pub ticket: usize,
    pub dealer: Arc<AtomicUsize>,
}

impl Default for Signal {
    fn default() -> Self {
        Self::new()
    }
}

impl Signal {
    pub fn new() -> Self {
        Self {
            ticket: 0,
            dealer: AtomicUsize::new(0).into(),
        }
    }

    pub fn deal(&self) -> Self {
        self.dealer
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let next = self.dealer.load(std::sync::atomic::Ordering::Relaxed);
        Self {
            ticket: next,
            dealer: self.dealer.clone(),
        }
    }

    pub fn valid(&self) -> bool {
        self.ticket == self.dealer.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[derive(Clone, Debug)]
pub struct UserInput {
    pub input: SharedStr,
    signal: Signal,
}

impl PartialEq for UserInput {
    fn eq(&self, other: &Self) -> bool {
        self.input == other.input
    }
}

impl Eq for UserInput {}

impl UserInput {
    pub fn new(input: &str, signal: &Signal) -> Self {
        UserInput {
            input: input.into(),
            signal: signal.deal(),
        }
    }

    pub fn cancelled(&self) -> bool {
        !self.signal.valid()
    }

    pub fn cancel(&self) {
        self.signal.deal();
    }
}

#[cfg(test)]
mod tests {
    use super::Signal;

    #[test]
    fn test() {
        let signal = Signal::new();
        println!("{:?}", signal);
        let next = signal.deal();
        println!("{:?}", signal);
        println!("{:?}", next);
    }
}
