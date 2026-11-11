// TODO: Bu egzersizi bitirdiğini düşündüğünde `ready` içinde `move_forward`'u `true` yap.
//  Çözümünü doğrulatmak için bir eğitmen çağırmaktan çekinme!
use channels::data::TicketDraft;
use channels::{launch, Command};
use std::time::Duration;
use ticket_fields::test_helpers::{ticket_description, ticket_title};

#[test]
fn a_thread_is_spawned() {
    let sender = launch();
    std::thread::sleep(Duration::from_millis(200));

    sender
        .send(Command::Insert(TicketDraft {
            title: ticket_title(),
            description: ticket_description(),
        }))
        // Thread artık çalışmıyorsa bu panic'leyecek,
        // çünkü kanal kapanmış olacak.
        .expect("Did you actually spawn a thread? The channel is closed!");
}

#[test]
fn ready() {
    // Bu egzersizde otomatik olarak kontrol edebileceğimiz çok az şey var,
    // çünkü sunucumuz hiçbir **okuma** eylemi sunmuyor.
    // Eklemelerin gerçekten gerçekleşip gerçekleşmediğini ve doğru bir şekilde
    // gerçekleşip gerçekleşmediğini bilmenin hiçbir yolu yok.
    let move_forward = false;

    assert!(move_forward);
}
