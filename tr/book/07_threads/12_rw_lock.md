# Okuyucular ve Yazıcılar {#readers-and-writers}

Yeni `TicketStore`'umuz çalışıyor, ama okuma performansı pek iyi değil: belirli bir ticket'ı aynı anda
yalnızca bir istemci okuyabilir, çünkü `Mutex<T>` okuyucular ile yazıcılar arasında ayrım yapmaz.

Bu sorunu farklı bir kilitleme primitive'i kullanarak çözebiliriz: `RwLock<T>`.\
`RwLock<T>`, **read-write lock** (okuma-yazma kilidi) ifadesinin kısaltmasıdır. Veriye **birden fazla okuyucunun** aynı anda erişmesine izin verir,
ama aynı anda yalnızca bir yazıcıya.

`RwLock<T>`'nin kilit almak için iki metodu vardır: `read` ve `write`.\
`read`, veriyi okumanı sağlayan bir guard döndürür; `write` ise veriyi değiştirmeni sağlayan bir guard döndürür.

```rust
use std::sync::RwLock;

// Bir okuma-yazma kilidiyle korunan bir tam sayı
let lock = RwLock::new(0);

// RwLock üzerinde bir okuma kilidi al
let guard1 = lock.read().unwrap();

// İlki hâlâ etkinken
// **ikinci** bir okuma kilidi al
let guard2 = lock.read().unwrap();
```

## Trade-off'lar (Ödünleşimler) {#trade-offs}

İlk bakışta `RwLock<T>` üzerine düşünmeye bile gerek olmayan bir seçim gibi görünüyor: `Mutex<T>`'nin işlevselliğinin bir üst kümesini sunuyor.
`RwLock<T>` kullanabiliyorken neden `Mutex<T>` kullanasın ki?

İki temel neden var:

- Bir `RwLock<T>`'yi kilitlemek, bir `Mutex<T>`'yi kilitlemekten daha maliyetlidir.\
  Bunun nedeni, `RwLock<T>`'nin etkin okuyucu ve yazıcıların sayısını takip etmek zorunda olması, `Mutex<T>`'nin ise
  yalnızca kilidin tutulup tutulmadığını takip etmesinin yeterli olmasıdır.
  Okuyucu sayısı yazıcılardan fazlaysa bu performans ek yükü sorun değildir, ama iş yükü
  yazma ağırlıklıysa `Mutex<T>` daha iyi bir seçim olabilir.
- `RwLock<T>` **yazıcı açlığına** yol açabilir.\
  Kilidi almayı bekleyen her zaman okuyucular varsa, yazıcılar hiçbir zaman çalışma fırsatı bulamayabilir.\
  `RwLock<T>`, okuyuculara ve yazıcılara kilide erişimin hangi sırayla verileceği konusunda hiçbir garanti vermez.
  Bu, altta yatan işletim sisteminin implement ettiği politikaya bağlıdır; bu politika da yazıcılara karşı adil olmayabilir.

Bizim durumumuzda iş yükünün okuma ağırlıklı olmasını bekleyebiliriz (çünkü çoğu istemci ticket'ları değiştirmeyecek, okuyacak);
dolayısıyla `RwLock<T>` iyi bir seçimdir.
