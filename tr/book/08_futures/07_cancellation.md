# İptal {#cancellation}

Beklemedeki bir future drop edildiğinde ne olur?\
Runtime onu artık poll etmez, dolayısıyla future daha fazla ilerleme kaydetmez.
Başka bir deyişle, çalışması **iptal edilmiştir**.

Gerçek hayatta bu, çoğunlukla zaman aşımlarıyla (timeout) çalışırken olur.
Örneğin:

```rust
use tokio::time::timeout;
use tokio::sync::oneshot;
use std::time::Duration;

async fn http_call() {
    // [...]
}

async fn run() {
    // Future'ı, 10 milisaniye sonra dolacak şekilde ayarlanmış bir `Timeout` ile sarmala.
    let duration = Duration::from_millis(10);
    if let Err(_) = timeout(duration, http_call()).await {
        println!("Didn't receive a value within 10 ms");
    }
}
```

Zaman aşımı süresi dolduğunda, `http_call`'un döndürdüğü future iptal edilir.
`http_call`'un gövdesinin şu olduğunu düşünelim:

```rust
use std::net::TcpStream;

async fn http_call() {
    let (stream, _) = TcpStream::connect(/* */).await.unwrap();
    let request: Vec<u8> = /* */;
    stream.write_all(&request).await.unwrap();
}
```

Her yield point bir **iptal noktası** haline gelir.\
`http_call` runtime tarafından preempt edilemez (kesintiye uğratılamaz), bu yüzden ancak
`.await` aracılığıyla kontrolü executor'a geri yield ettikten sonra bir kenara atılabilir.
Bu özyinelemeli olarak geçerlidir—örneğin `stream.write_all(&request)`'in implementation'ında
büyük olasılıkla birden çok yield point vardır. `http_call`'un iptal edilmeden önce _kısmi_ bir
isteği gönderdiğini, böylece bağlantıyı drop ettiğini ve gövdeyi iletmeyi hiç
bitirmediğini görmek tamamen mümkündür.

## Temizlik {#clean-up}

Rust'ın iptal mekanizması oldukça güçlüdür—çağıranın, devam eden bir task'i (görevi), task'in
kendisinden hiçbir iş birliği gerektirmeden iptal etmesine olanak tanır.\
Aynı zamanda bu oldukça tehlikeli olabilir. İşlemi sonlandırmadan önce bazı temizlik
görevlerinin yapıldığından emin olmak için bir **kontrollü iptal**
gerçekleştirmek istenebilir.

Örneğin, bir SQL transaction'ı için şu hayali API'yi ele alalım:

```rust
async fn transfer_money(
    connection: SqlConnection,
    payer_id: u64,
    payee_id: u64,
    amount: u64
) -> Result<(), anyhow::Error> {
    let transaction = connection.begin_transaction().await?;
    update_balance(payer_id, amount, &transaction).await?;
    decrease_balance(payee_id, amount, &transaction).await?;
    transaction.commit().await?;
}
```

İptal durumunda, bekleyen transaction'ı askıda bırakmak yerine açıkça sonlandırmak
ideal olurdu.
Ne yazık ki Rust, bu tür **asenkron** temizlik işlemleri için kusursuz bir
mekanizma sağlamaz.

En yaygın strateji, gerekli temizlik işini zamanlamak için `Drop` trait'ine
güvenmektir. Bu şu yollarla yapılabilir:

- Runtime üzerinde yeni bir task spawn ederek
- Bir kanalın kuyruğuna bir mesaj ekleyerek
- Arka planda bir thread (iş parçacığı) spawn ederek

En iyi seçim bağlama göre değişir.

## Spawn Edilen Task'leri İptal Etmek {#cancelling-spawned-tasks}

Bir task'i `tokio::spawn` kullanarak spawn ettiğinde artık onu drop edemezsin;
task runtime'a aittir.\
Yine de gerekirse onu iptal etmek için `JoinHandle`'ını kullanabilirsin:

```rust
async fn run() {
    let handle = tokio::spawn(/* bir async task */);
    // Spawn edilen task'i iptal et
    handle.abort();
}
```

## İleri Okuma {#further-reading}

- İki farklı future'ı "yarıştırmak" için `tokio`'nun `select!` makrosunu kullanırken son derece dikkatli ol.
  **İptal güvenliğini** sağlayamıyorsan aynı task'i bir döngüde yeniden denemek tehlikelidir.
  Daha fazla ayrıntı için [`select!` dokümantasyonuna](https://tokio.rs/tokio/tutorial/select) göz at.\
  İki asenkron veri akışını (ör. bir soket ve bir kanal) dönüşümlü olarak işlemen gerekiyorsa, bunun yerine
  [`StreamExt::merge`](https://docs.rs/tokio-stream/latest/tokio_stream/trait.StreamExt.html#method.merge) kullanmayı tercih et.
- Bazı durumlarda bir [`CancellationToken`](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html),
  `JoinHandle::abort`'a tercih edilebilir.
