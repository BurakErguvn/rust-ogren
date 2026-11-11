# Task'leri (Görevleri) Spawn Etmek {#spawning-tasks}

Önceki egzersize yazdığın çözüm aşağı yukarı şöyle görünmeli:

```rust
pub async fn echo(listener: TcpListener) -> Result<(), anyhow::Error> {
    loop {
        let (mut socket, _) = listener.accept().await?;
        let (mut reader, mut writer) = socket.split();
        tokio::io::copy(&mut reader, &mut writer).await?;
    }
}
```

Fena değil!\
Gelen iki bağlantı arasında uzun bir süre geçerse `echo` fonksiyonu boşta bekler
(çünkü `TcpListener::accept` asenkron bir fonksiyondur); bu da executor'ın bu arada
başka task'leri çalıştırmasına olanak tanır.

Ama birden çok task'in eşzamanlı olarak çalışmasını gerçekten nasıl sağlayabiliriz?\
Asenkron fonksiyonlarımızı her zaman sonuna kadar çalıştırırsak (`.await` kullanarak), aynı anda
hiçbir zaman birden fazla task çalışıyor olmaz.

İşte burada `tokio::spawn` fonksiyonu devreye girer.

## `tokio::spawn` {#tokiospawn}

`tokio::spawn`, bir task'i **tamamlanmasını beklemeden** executor'a devretmeni sağlar.\
`tokio::spawn`'ı her çağırdığında `tokio`'ya, spawn edilen task'i arka planda,
onu spawn eden task ile **eşzamanlı olarak** çalıştırmaya devam etmesini söylüyorsun.

Birden çok bağlantıyı eşzamanlı olarak işlemek için onu şöyle kullanabilirsin:

```rust
use tokio::net::TcpListener;

pub async fn echo(listener: TcpListener) -> Result<(), anyhow::Error> {
    loop {
        let (mut socket, _) = listener.accept().await?;
        // Bağlantıyı ele almak için bir arka plan task'i spawn et,
        // böylece ana task yeni bağlantıları hemen
        // kabul etmeye başlayabilir
        tokio::spawn(async move {
            let (mut reader, mut writer) = socket.split();
            tokio::io::copy(&mut reader, &mut writer).await?;
        });
    }
}
```

### Asenkron Bloklar {#asynchronous-blocks}

Bu örnekte `tokio::spawn`'a bir **asenkron blok** verdik: `async move { /* */ }`
Asenkron bloklar, ayrı bir async fonksiyon tanımlamak zorunda kalmadan bir kod
bölgesini asenkron olarak işaretlemenin hızlı bir yoludur.

### `JoinHandle` {#joinhandle}

`tokio::spawn` bir `JoinHandle` döndürür.\
Spawn edilen thread'ler (iş parçacıkları) için `join` kullandığımız gibi, arka plandaki
task'i `.await` etmek için de `JoinHandle`'ı kullanabilirsin.

```rust
pub async fn run() {
    // Telemetri verisini uzak bir sunucuya
    // göndermek için bir arka plan task'i spawn et
    let handle = tokio::spawn(emit_telemetry());
    // Bu arada başka faydalı işler yap
    do_work().await;
    // Ama telemetri verisi başarıyla teslim edilene
    // kadar çağırana dönme
    handle.await;
}

pub async fn emit_telemetry() {
    // [...]
}

pub async fn do_work() {
    // [...]
}
```

### Panic Sınırı {#panic-boundary}

`tokio::spawn` ile spawn edilen bir task panic'lerse, panic executor tarafından yakalanır.\
İlgili `JoinHandle`'ı `.await` etmezsen panic, spawn edene iletilmez.
`JoinHandle`'ı `.await` etsen bile panic otomatik olarak iletilmez.
Bir `JoinHandle`'ı await etmek, hata türü [`JoinError`](https://docs.rs/tokio/latest/tokio/task/struct.JoinError.html)
olan bir `Result` döndürür. Ardından `JoinError::is_panic`'i çağırarak task'in panic'leyip panic'lemediğini kontrol edebilir ve
panic ile ne yapacağını seçebilirsin—loglayabilir, görmezden gelebilir ya da iletebilirsin.

```rust
use tokio::task::JoinError;

pub async fn run() {
    let handle = tokio::spawn(work());
    if let Err(e) = handle.await {
        if let Ok(reason) = e.try_into_panic() {
            // Task panic'ledi
            // Panic'in unwinding'ine devam ediyoruz,
            // böylece onu mevcut task'e iletiyoruz
            panic::resume_unwind(reason);
        }
    }
}

pub async fn work() {
    // [...]
}
```

### `std::thread::spawn` Ve `tokio::spawn` {#stdthreadspawn-vs-tokiospawn}

`tokio::spawn`'ı, `std::thread::spawn`'ın asenkron kardeşi olarak düşünebilirsin.

Önemli bir farka dikkat et: `std::thread::spawn` ile kontrolü işletim sistemi scheduler'ına (zamanlayıcısına) devrediyorsun.
Thread'lerin nasıl zamanlandığı senin kontrolünde değildir.

`tokio::spawn` ile ise tamamen kullanıcı alanında (user space) çalışan bir async executor'a
devrediyorsun. Bir sonraki adımda hangi task'in çalışacağına dair kararda alttaki işletim sistemi
scheduler'ı yer almaz. Artık bu karar, kullanmayı seçtiğimiz executor aracılığıyla bizim elimizde.
