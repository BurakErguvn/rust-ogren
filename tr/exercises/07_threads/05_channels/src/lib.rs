use std::sync::mpsc::{Receiver, Sender};

pub mod data;
pub mod store;

pub enum Command {
    Insert(todo!()),
}

// Sunucu thread'ini spawn ederek sistemi başlat.
// Bir `Sender` instance'ı döndürür; bu instance daha sonra
// bir ya da daha fazla istemci tarafından sunucuyla etkileşim kurmak için kullanılabilir.
pub fn launch() -> Sender<Command> {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || server(receiver));
    sender
}

// TODO: Sunucu görevi **asla** durmamalı.
//  Bir döngüye gir: kanalda bir komutun belirmesini
//  bekle, ardından onu çalıştır, ardından bir sonraki
//  komutu beklemeye başla.
pub fn server(receiver: Receiver<Command>) {}
