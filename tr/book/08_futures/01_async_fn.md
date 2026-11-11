# Asenkron Fonksiyonlar {#asynchronous-functions}

Şu ana kadar yazdığın tüm fonksiyonlar ve metotlar eager (hemen çalışan) idi.\
Sen onları çağırana kadar hiçbir şey olmuyordu. Ama çağırdığında sonuna kadar
çalışıyorlardı: işlerinin **tamamını** yapıyor, ardından çıktılarını döndürüyorlardı.

Bazen bu istenmeyen bir durumdur.\
Örneğin bir HTTP sunucusu yazıyorsan çok fazla **bekleme** olabilir: istek
gövdesinin gelmesini beklemek, veritabanının yanıt vermesini beklemek, alt akıştaki bir
servisin cevap vermesini beklemek vb.

Beklerken başka bir şey yapabilseydin ne olurdu?\
Bir hesaplamanın ortasında vazgeçmeyi seçebilseydin ne olurdu?\
Başka bir task'e (göreve) mevcut olandan daha fazla öncelik vermeyi seçebilseydin ne olurdu?

İşte burada **asenkron fonksiyonlar** devreye girer.

## `async fn` {#async-fn}

Asenkron bir fonksiyon tanımlamak için `async` anahtar kelimesini kullanırsın:

```rust
use tokio::net::TcpListener;

// Bu fonksiyon asenkron
async fn bind_random() -> TcpListener {
    // [...]
}
```

`bind_random`'ı sıradan bir fonksiyon gibi çağırırsan ne olur?

```rust
fn run() {
    // `bind_random`'ı çağır
    let listener = bind_random();
    // Şimdi ne olacak?
}
```

Hiçbir şey olmaz!\
Rust, `bind_random`'ı çağırdığında onu çalıştırmaya başlamaz;
(diğer dillerdeki deneyimine dayanarak bekleyebileceğin gibi) bir arka plan task'i olarak
bile başlatmaz.
Rust'taki asenkron fonksiyonlar **lazy**'dir (tembeldir): sen açıkça istemediğin sürece
hiçbir iş yapmazlar.
Rust'ın terminolojisiyle, `bind_random`'ın bir **future** döndürdüğünü söyleriz; future, daha
sonra tamamlanabilecek bir hesaplamayı temsil eden bir türdür. Bunlara future denir,
çünkü `Future` trait'ini implement ederler; bu arayüzü bu bölümün ilerleyen kısımlarında
ayrıntılı olarak inceleyeceğiz.

## `.await` {#await}

Asenkron bir fonksiyondan iş yapmasını istemenin en yaygın yolu
`.await` anahtar kelimesini kullanmaktır:

```rust
use tokio::net::TcpListener;

async fn bind_random() -> TcpListener {
    // [...]
}

async fn run() {
    // `bind_random`'ı çağır ve tamamlanmasını bekle
    let listener = bind_random().await;
    // Artık `listener` hazır
}
```

`.await`, asenkron fonksiyon sonuna kadar çalışmadan kontrolü çağırana geri
vermez—örneğin yukarıdaki örnekte `TcpListener` oluşturulana kadar.

## Runtime'lar {#runtimes}

Kafan karıştıysa, karışmakta haklısın!\
Az önce asenkron fonksiyonların avantajının, işlerinin **tamamını** tek seferde
yapmamaları olduğunu söyledik. Ardından, asenkron fonksiyon sonuna kadar çalışmadan
dönmeyen `.await`'i tanıttık. Çözmeye çalıştığımız sorunu az önce yeniden ortaya
çıkarmadık mı? Bunun ne anlamı var?

Tam olarak değil! `.await`'i çağırdığında perde arkasında pek çok şey olur!\
Kontrolü bir **async runtime**'a, diğer adıyla bir **async executor**'a yield ediyorsun.
Sihrin gerçekleştiği yer executor'lardır: devam eden tüm asenkron **task**'lerini
yönetmekten onlar sorumludur. Özellikle iki farklı hedef arasında denge kurarlar:

- **İlerleme**: task'lerin ilerleyebildikleri her an ilerlemesini sağlarlar.
- **Verimlilik**: bir task bir şey bekliyorsa, bu arada başka bir task'in çalışabilmesini
  sağlamaya çalışarak mevcut kaynakları tam olarak kullanırlar.

### Varsayılan Runtime Yok {#no-default-runtime}

Rust, asenkron programlamaya yaklaşımında oldukça benzersizdir: varsayılan bir
runtime yoktur. Standart kütüphane bir runtime ile gelmez. Kendi runtime'ını
getirmen gerekir!

Çoğu durumda ekosistemde mevcut seçeneklerden birini seçeceksin.
Bazı runtime'lar geniş bir kullanım alanına uygun olacak şekilde tasarlanmıştır ve çoğu uygulama için sağlam bir seçenektir.
`tokio` ve `async-std` bu kategoriye girer. Diğer runtime'lar belirli kullanım
senaryoları için optimize edilmiştir—örneğin gömülü sistemler için `embassy`.

Bu kurs boyunca, Rust'ta genel amaçlı asenkron programlama için en popüler runtime olan
`tokio`'ya güveneceğiz.

### `#[tokio::main]` {#tokiomain}

Çalıştırılabilir programının giriş noktası olan `main` fonksiyonu senkron bir fonksiyon olmalıdır.
Seçtiğin async runtime'ı kurman ve başlatman gereken yer orasıdır.

Çoğu runtime bunu kolaylaştırmak için bir makro sağlar. `tokio` için bu makro `tokio::main`'dir:

```rust
#[tokio::main]
async fn main() {
    // Async kodun buraya gelir
}
```

bu da şuna genişler:

```rust
fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(
        // Async fonksiyonun buraya gelir
        // [...]
    );
}
```

### `#[tokio::test]` {#tokiotest}

Aynısı testler için de geçerlidir: testler senkron fonksiyonlar olmalıdır.\
Her test fonksiyonu kendi thread'inde (iş parçacığında) çalıştırılır ve testlerinde async kod
çalıştırman gerekiyorsa bir async runtime kurmak ve başlatmak senin
sorumluluğundadır.\
`tokio` bunu kolaylaştırmak için bir `#[tokio::test]` makrosu sağlar:

```rust
#[tokio::test]
async fn my_test() {
    // Async test kodun buraya gelir
}
```
