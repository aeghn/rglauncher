use std::sync::{atomic::AtomicUsize, Arc};


#[derive(Clone, Debug)]
pub struct KTicket {
    cursor: usize,
    holder: Arc<AtomicUsize>,
}

impl KTicket {
    pub fn create() -> KTicket {
        Self {
            cursor: 0,
            holder: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn create_next(&self) -> KTicket {
        let c = self
            .holder
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            .saturating_add(1);

        Self {
            cursor: c,
            holder: self.holder.clone(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.holder.load(std::sync::atomic::Ordering::Relaxed) == self.cursor
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn cursor_eq(&self, ticket: &KTicket) -> bool {
        self.cursor == ticket.cursor
    }
}
