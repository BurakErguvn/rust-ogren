// TODO: `TicketStore::add_ticket`'ın imzasını, `impl Trait` söz dizimi yerine generic bir tür
//  parametresi kullanacak şekilde yeniden düzenle.

use ticket_fields::{TicketDescription, TicketTitle};

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

    // `ticket` için tür parametresi olarak `Into<Ticket>` kullanmak, metodun başarısız olamayan
    // bir dönüşümle `Ticket`'a dönüştürülebilen her türü kabul etmesini sağlar.
    // Bu, çağrı noktasından `.into()`'nun yarattığı söz dizimi gürültüsünü kaldırdığı için metodun
    // kullanımını daha hoş hale getirebilir. Yine de derleyici hata mesajlarının kalitesini düşürebilir.
    pub fn add_ticket(&mut self, ticket: impl Into<Ticket>) {
        self.tickets.push(ticket.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ticket_fields::test_helpers::{ticket_description, ticket_title};

    struct TicketDraft {
        pub title: TicketTitle,
        pub description: TicketDescription,
    }

    impl From<TicketDraft> for Ticket {
        fn from(draft: TicketDraft) -> Self {
            Self {
                title: draft.title,
                description: draft.description,
                status: Status::ToDo,
            }
        }
    }

    #[test]
    fn generic_add() {
        let mut store = TicketStore::new();
        // `add_ticket` argüman konumunda `impl Trait` söz dizimini kullanırsa bu derlenmez.
        store.add_ticket::<TicketDraft>(TicketDraft {
            title: ticket_title(),
            description: ticket_description(),
        });
    }
}
