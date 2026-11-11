use ticket_fields::{TicketDescription, TicketTitle};

// TODO: Ticket store'umuzun taslağını çizmeye başlayalım!
//  İlk görev: `TicketStore`'un içerdiği tüm ticket'lar üzerinde bir `for` döngüsüyle
//  gezinilebilmesi için `TicketStore` üzerinde `IntoIterator`'ı implement et.
//
// İpucu: bu durumda `Iterator` trait'ini implement etmen gerekmemeli.
//   Yinelemeyi `TicketStore`'daki `Vec<Ticket>` alanına *devretmek* istiyorsun.
//   `into_iter`'den döndürülecek doğru türü bulmak için standart kütüphanenin `Vec`
//   dokümantasyonuna bak.
#[derive(Clone)]
pub struct TicketStore {
    tickets: Vec<Ticket>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ticket {
    pub title: TicketTitle,
    pub description: TicketDescription,
    pub status: Status,
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Status {
    ToDo,
    InProgress,
    Done,
}

impl TicketStore {
    pub fn new() -> Self {
        Self {
            tickets: Vec::new(),
        }
    }

    pub fn add_ticket(&mut self, ticket: Ticket) {
        self.tickets.push(ticket);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ticket_fields::test_helpers::{ticket_description, ticket_title};

    #[test]
    fn add_ticket() {
        let mut store = TicketStore::new();

        let ticket = Ticket {
            title: ticket_title(),
            description: ticket_description(),
            status: Status::ToDo,
        };
        store.add_ticket(ticket);

        let ticket = Ticket {
            title: ticket_title(),
            description: ticket_description(),
            status: Status::InProgress,
        };
        store.add_ticket(ticket);

        let tickets: Vec<_> = store.clone().into_iter().collect();
        assert_eq!(tickets, store.tickets);
    }
}
