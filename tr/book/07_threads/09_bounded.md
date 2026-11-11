# Sınırlı ve Sınırsız Kanallar {#bounded-vs-unbounded-channels}

Şimdiye kadar sınırsız kanallar kullanıyorduk.\
İstediğin kadar mesaj gönderebilirsin ve kanal onları alacak şekilde büyür.\
Çok üreticili, tek tüketicili bir senaryoda bu sorun yaratabilir: üreticiler mesajları
tüketicinin işleyebileceğinden daha hızlı kuyruğa eklerse kanal
büyümeye devam eder ve potansiyel olarak kullanılabilir belleğin tamamını tüketir.

Önerimiz, bir üretim sisteminde sınırsız bir kanalı **asla** kullanmamandır.\
Kuyruğa eklenebilecek mesaj sayısına her zaman bir **sınırlı kanal** kullanarak
bir üst sınır koymalısın.

## Sınırlı Kanallar {#bounded-channels}

Sınırlı bir kanalın sabit bir kapasitesi vardır.\
Sıfırdan büyük bir kapasiteyle `sync_channel` çağırarak bir tane oluşturabilirsin:

```rust
use std::sync::mpsc::sync_channel;

let (sender, receiver) = sync_channel(10);
```

`receiver` öncekiyle aynı türe sahiptir: `Receiver<T>`.\
`sender` ise bir `SyncSender<T>` instance'ıdır (örnek).

### Mesaj Göndermek {#sending-messages}

Bir `SyncSender` üzerinden mesaj göndermek için iki farklı metodun var:

- `send`: kanalda yer varsa mesajı kuyruğa ekler ve `Ok(())` döndürür.\
  Kanal doluysa bloklar ve yer açılana kadar bekler.
- `try_send`: kanalda yer varsa mesajı kuyruğa ekler ve `Ok(())` döndürür.\
  Kanal doluysa `Err(TrySendError::Full(value))` döndürür; burada `value`, gönderilemeyen mesajdır.

Kullanım senaryona bağlı olarak birini ya da diğerini kullanmak isteyebilirsin.

### Backpressure {#backpressure}

Sınırlı kanal kullanmanın asıl avantajı, bir tür **backpressure** (geri basınç) sağlamalarıdır.\
Tüketici yetişemiyorsa üreticileri yavaşlamaya zorlarlar.
Backpressure daha sonra sistem boyunca iletilebilir; potansiyel olarak tüm mimariyi etkileyerek
son kullanıcıların sistemi isteklere boğmasını engeller.
