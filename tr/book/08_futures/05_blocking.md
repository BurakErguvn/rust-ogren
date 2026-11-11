# Runtime'ı Bloklama {#dont-block-the-runtime}

Yield point'lere geri dönelim.\
Thread'lerin (iş parçacıklarının) aksine, **Rust task'leri (görevleri) preempt edilemez (kesintiye uğratılamaz)**.

`tokio`, kendi başına bir task'i duraklatıp onun yerine başka birini çalıştırmaya karar veremez.
Kontrol executor'a **yalnızca** task yield ettiğinde geri döner—yani
`Future::poll` `Poll::Pending` döndürdüğünde ya da `async fn` söz konusuysa
bir future'ı `.await` ettiğinde.

Bu durum runtime'ı bir riske açık hale getirir: bir task hiç yield etmezse runtime asla
başka bir task'i çalıştıramaz. Buna **runtime'ı bloklamak** denir.

## Bloklama Nedir? {#what-is-blocking}

Ne kadar uzun, fazla uzundur? Bir task, sorun haline gelmeden önce yield etmeden
ne kadar zaman geçirebilir?

Bu; runtime'a, uygulamaya, o anda süren task'lerin sayısına ve
başka pek çok etkene bağlıdır. Ama genel bir pratik kural olarak, yield point'ler arasında
100 mikrosaniyeden az zaman geçirmeye çalış.

## Sonuçlar {#consequences}

Runtime'ı bloklamak şunlara yol açabilir:

- **Deadlock**'lar (kilitlenmeler): yield etmeyen task, başka bir task'in tamamlanmasını
  bekliyorsa ve o task da ilkinin yield etmesini bekliyorsa, bir deadlock oluşur.
  Runtime diğer task'i farklı bir thread üzerinde zamanlayamadığı sürece
  hiçbir ilerleme sağlanamaz.
- **Açlık**: diğer task'ler çalışamayabilir ya da uzun bir gecikmeden sonra
  çalışabilir; bu da düşük performansa, örneğin yüksek tail latency'lere (kuyruk gecikmelerine) yol açabilir.

## Bloklama Her Zaman Bariz Değildir {#blocking-is-not-always-obvious}

Bazı işlem türlerinden async kodda genellikle kaçınılmalıdır, örneğin:

- Senkron I/O. Ne kadar süreceğini öngöremezsin ve büyük olasılıkla
  100 mikrosaniyeden uzun sürer.
- Maliyetli, CPU'ya bağlı hesaplamalar.

Ancak ikinci kategori her zaman bariz değildir. Örneğin birkaç elemanlı bir vektörü
sıralamak sorun değildir; vektörde milyarlarca eleman varsa bu değerlendirme
değişir.

## Bloklamadan Nasıl Kaçınılır {#how-to-avoid-blocking}

Peki, bloklayıcı sayılan ya da sayılma riski taşıyan bir işlemi gerçekleştirmek
_zorundaysan_, runtime'ı bloklamaktan nasıl kaçınırsın?\
İşi farklı bir thread'e taşıman gerekir. Runtime thread'leri denen thread'leri,
yani `tokio`'nun task'leri çalıştırmak için kullandığı thread'leri kullanmak istemezsin.

`tokio` bu amaç için **blocking pool** adında özel bir thread pool sağlar.
`tokio::task::spawn_blocking` fonksiyonunu kullanarak blocking pool üzerinde senkron
bir işlem spawn edebilirsin. `spawn_blocking`, işlem tamamlandığında işlemin sonucunu
veren bir future döndürür.

```rust
use tokio::task;

fn expensive_computation() -> u64 {
    // [...]
}

async fn run() {
    let handle = task::spawn_blocking(expensive_computation);
    // Bu arada başka şeyler yap
    let result = handle.await.unwrap();
}
```

Blocking pool uzun ömürlüdür. `spawn_blocking`, doğrudan `std::thread::spawn`
ile yeni bir thread oluşturmaktan daha hızlı olmalıdır,
çünkü thread'in başlatılmasının maliyeti birden çok çağrıya yayılır (amorti edilir).

## İleri Okuma {#further-reading}

- Konuyla ilgili [Alice Ryhl'ın blog yazısına](https://ryhl.io/blog/async-what-is-blocking/)
  göz at.
