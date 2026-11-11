//! TODO: `example` fonksiyonundaki deyimleri **yeniden sıralayarak** kodun
//!  derlenmesini sağla. `spawner` fonksiyonunu ya da `example` içindeki her
//!  satırın ne yaptığını değiştirmene izin yok.
//!   Gerekirse mevcut deyimleri `{}` bloklarının içine sarmalayabilirsin.
use std::rc::Rc;
use tokio::task::yield_now;

fn spawner() {
    tokio::spawn(example());
}

async fn example() {
    let non_send = Rc::new(1);
    yield_now().await;
    println!("{}", non_send);
}
