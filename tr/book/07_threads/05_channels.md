# Kanallar {#channels}

Şimdiye kadar spawn ettiğimiz tüm thread'ler (iş parçacıkları) oldukça kısa ömürlüydü.\
Biraz girdi al, bir hesaplama yap, sonucu döndür, kapan.

Ticket yönetim sistemimiz için farklı bir şey yapmak istiyoruz:
bir istemci-sunucu mimarisi.

Durumumuzu, yani saklanan ticket'ları yönetmekten sorumlu **uzun süre çalışan tek bir sunucu thread'imiz**
olacak.

Ardından **birden fazla istemci thread'imiz** olacak.\
Her istemci, durumunu değiştirmek (ör. yeni bir ticket eklemek)
ya da bilgi almak (ör. bir ticket'ın durumunu öğrenmek) için durum tutan thread'e
**komutlar** ve **sorgular** gönderebilecek.\
İstemci thread'leri eşzamanlı olarak çalışacak.

## İletişim {#communication}

Şimdiye kadar yalnızca çok sınırlı bir üst-alt iletişimimiz oldu:

- Spawn edilen thread, üst bağlamdan veri ödünç aldı/tüketti
- Spawn edilen thread, join edildiğinde üst thread'e veri döndürdü

Bu, istemci-sunucu tasarımı için yeterli değil.\
İstemcilerin, sunucu thread'i başlatıldıktan _sonra_ ona veri gönderebilmesi
ve ondan veri alabilmesi gerekir.

Bu sorunu **kanallar** kullanarak çözebiliriz.

## Kanallar {#channels-1}

Rust'ın standart kütüphanesi, `std::sync::mpsc` modülünde **çok üreticili, tek tüketicili** (mpsc)
kanallar sunar.\
İki kanal çeşidi vardır: sınırlı ve sınırsız. Şimdilik sınırsız sürümle devam edeceğiz,
ama artılarını ve eksilerini daha sonra tartışacağız.

Kanal oluşturma şöyle görünür:

```rust
use std::sync::mpsc::channel;

let (sender, receiver) = channel();
```

Bir gönderici ve bir alıcı elde edersin.\
Kanala veri itmek için gönderici üzerinde `send` çağırırsın.\
Kanaldan veri çekmek için alıcı üzerinde `recv` çağırırsın.

### Birden Fazla Gönderici {#multiple-senders}

`Sender` klonlanabilir: birden fazla gönderici oluşturabiliriz (ör. her istemci thread'i
için bir tane) ve hepsi aynı kanala veri iter.

`Receiver` ise klonlanabilir değildir: belirli bir kanal için yalnızca tek bir alıcı
olabilir.

**mpsc** (multi-producer single-consumer, yani çok üreticili tek tüketicili) tam olarak bu anlama gelir!

### Mesaj Türü {#message-type}

Hem `Sender` hem `Receiver`, `T` tür parametresi üzerinde generic'tir.\
Bu, kanalımızda dolaşabilen _mesajların_ türüdür.

Bir `u64`, bir struct, bir enum vb. olabilir.

### Hatalar {#errors}

Hem `send` hem `recv` başarısız olabilir.\
Alıcı drop edildiyse `send` bir hata döndürür.\
Tüm göndericiler drop edildiyse ve kanal boşsa `recv` bir hata döndürür.

Başka bir deyişle, kanal fiilen kapandığında `send` ve `recv` hata verir.
