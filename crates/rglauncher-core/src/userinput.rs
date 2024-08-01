use crate::misc::KTicket;

#[derive(Clone, Debug)]
pub struct UserInput {
    pub input: String,
    ticket: KTicket,
}
impl PartialEq for UserInput {
    fn eq(&self, other: &Self) -> bool {
        self.input == other.input
    }
}

impl UserInput {
    pub fn new(input: &str, ticket: &KTicket) -> Self {
        UserInput {
            input: input.to_string(),
            ticket: ticket.clone(),
        }
    }

    pub fn cancelled(&self) -> bool {
        !self.ticket.is_valid()
    }

    pub fn get_ticket(&self) -> KTicket {
        self.ticket.clone()
    }
}
