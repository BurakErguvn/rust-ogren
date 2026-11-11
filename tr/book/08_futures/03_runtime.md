# Runtime Mimarisi {#runtime-architecture}

Şimdiye kadar async runtime'lardan soyut bir kavram olarak bahsettik.
Biraz daha derine inip nasıl implement edildiklerine bakalım—birazdan göreceğin gibi,
bunun kodumuz üzerinde bir etkisi var.

## Çeşitler {#flavors}

`tokio` iki farklı runtime _çeşidi_ (flavor) ile gelir.

Runtime'ını `tokio::runtime::Builder` aracılığıyla yapılandırabilirsin:

- `Builder::new_multi_thread` sana **çok thread'li bir `tokio` runtime'ı** verir
- `Builder::new_current_thread` ise çalıştırma için **mevcut thread**'e (iş parçacığına) dayanır.

`#[tokio::main]` varsayılan olarak çok thread'li bir runtime döndürür,
`#[tokio::test]` ise varsayılan olarak bir mevcut thread runtime'ı kullanır.

### Mevcut Thread Runtime'ı {#current-thread-runtime}

Mevcut thread runtime'ı, adından da anlaşılacağı gibi, task'leri (görevleri) zamanlamak ve çalıştırmak için
yalnızca başlatıldığı işletim sistemi thread'ine dayanır.\
Mevcut thread runtime'ını kullanırken **eşzamanlılığın** olur ama **paralelliğin** olmaz:
asenkron task'ler dönüşümlü olarak çalıştırılır, ama herhangi bir anda en fazla bir task
çalışıyor olur.

### Çok Thread'li Runtime {#multithreaded-runtime}

Çok thread'li runtime kullanırken ise herhangi bir anda _paralel olarak_ çalışan
en fazla `N` task olabilir; burada `N`, runtime'ın kullandığı thread sayısıdır.
Varsayılan olarak `N`, mevcut CPU çekirdeği sayısına eşittir.

Dahası var: `tokio` **work-stealing** (iş çalma) yapar.\
Bir thread boştaysa öylece beklemez: ya global bir kuyruktan ya da başka bir thread'in
yerel kuyruğundan çalarak, çalıştırılmaya hazır yeni bir task bulmaya
çalışır.\
Uygulaman thread'ler arasında mükemmel şekilde dengelenmemiş iş yükleriyle uğraştığında
work-stealing, özellikle tail latency'ler (kuyruk gecikmeleri) üzerinde, ciddi performans
avantajları sağlayabilir.

## Etkiler {#implications}

`tokio::spawn` çeşitten bağımsızdır: çok thread'li runtime üzerinde de, mevcut thread
runtime'ı üzerinde de çalışır. Bunun dezavantajı, imzanın en kötü durumu
(yani çok thread'li olanı) varsayması ve buna göre kısıtlanmış olmasıdır:

```rust
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{ /* */ }
```

Şimdilik `Future` trait'ini görmezden gelip geri kalanına odaklanalım.\
`spawn`, tüm girdilerinin `Send` olmasını ve `'static` lifetime'ına (yaşam süresine) sahip olmasını istiyor.

`'static` kısıtı, `std::thread::spawn` üzerindeki `'static` kısıtıyla aynı gerekçeye
dayanır: spawn edilen task, spawn edildiği bağlamdan daha uzun yaşayabilir;
bu nedenle spawn eden bağlam yok edildikten sonra serbest bırakılabilecek
herhangi bir yerel veriye bağımlı olmamalıdır.

```rust
fn spawner() {
    let v = vec![1, 2, 3];
    // Bu çalışmaz, çünkü `&v` yeterince
    // uzun yaşamaz.
    tokio::spawn(async { 
        for x in &v {
            println!("{x}")
        }
    })
}
```

Öte yandan `Send`, `tokio`'nun work-stealing stratejisinin doğrudan bir sonucudur:
`A` thread'inde spawn edilen bir task, `B` thread'i boştaysa sonunda oraya taşınabilir;
thread sınırlarını aştığımız için de bu bir `Send` bound'u gerektirir.

```rust
fn spawner(input: Rc<u64>) {
    // Bu da çalışmaz, çünkü
    // `Rc`, `Send` değildir.
    tokio::spawn(async move {
        println!("{}", input);
    })
}
```
