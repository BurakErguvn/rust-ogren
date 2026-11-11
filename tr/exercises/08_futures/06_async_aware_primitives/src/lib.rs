/// TODO: aşağıdaki kod deadlock'a girecek, çünkü std'nin async-aware olmayan
///  kanallarını kullanıyor.
///  Kodu `tokio`'nun kanal primitive'ini kullanacak şekilde yeniden yaz (evet,
///  test koduna da dokunman gerekecek).
///
/// Deadlock'a yol açabilecek olay serisini anlayabiliyor musun?
use std::sync::mpsc;

pub struct Message {
    payload: String,
    response_channel: mpsc::Sender<Message>,
}

/// Aldığı her mesaja `pong` ile yanıt verir ve çağıranla iletişime devam
/// etmek için yeni bir kanal kurar.
pub async fn pong(mut receiver: mpsc::Receiver<Message>) {
    loop {
        if let Ok(msg) = receiver.recv() {
            println!("Pong received: {}", msg.payload);
            let (sender, new_receiver) = mpsc::channel();
            msg.response_channel
                .send(Message {
                    payload: "pong".into(),
                    response_channel: sender,
                })
                .unwrap();
            receiver = new_receiver;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{pong, Message};
    use std::sync::mpsc;

    #[tokio::test]
    async fn ping() {
        let (sender, receiver) = mpsc::channel();
        let (response_sender, response_receiver) = mpsc::channel();
        sender
            .send(Message {
                payload: "pong".into(),
                response_channel: response_sender,
            })
            .unwrap();

        tokio::spawn(pong(receiver));

        let answer = response_receiver.recv().unwrap().payload;
        assert_eq!(answer, "pong");
    }
}
