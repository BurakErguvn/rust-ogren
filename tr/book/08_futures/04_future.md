# `Future` Trait'i {#the-future-trait}

## Yerel `Rc` Sorunu {#the-local-rc-problem}

`tokio::spawn`'ın imzasına geri dönelim:

```rust
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
{ /* */ }
```

`F` türünün `Send` olması _aslında_ ne anlama geliyor?\
Önceki kısımda gördüğümüz gibi bu, spawn eden ortamdan yakaladığı her değerin
`Send` olması gerektiği anlamına gelir. Ama iş bununla bitmiyor.

_Bir .await noktası boyunca tutulan_ her değer `Send` olmalıdır.\
Bir örneğe bakalım:

```rust
use std::rc::Rc;
use tokio::task::yield_now;

fn spawner() {
    tokio::spawn(example());
}

async fn example() {
    // Async fonksiyonun _içinde_ oluşturulan,
    // `Send` olmayan bir değer
    let non_send = Rc::new(1);
    
    // Hiçbir şey yapmayan bir `.await` noktası
    yield_now().await;

    // `Send` olmayan yerel değere `.await`'ten
    // sonra hâlâ ihtiyaç var
    println!("{}", non_send);
}
```

Derleyici bu kodu reddeder:

```text
error: future cannot be sent between threads safely
    |
5   |     tokio::spawn(example());
    |                  ^^^^^^^^^ 
    | future returned by `example` is not `Send`
    |
note: future is not `Send` as this value is used across an await
    |
11  |     let non_send = Rc::new(1);
    |         -------- has type `Rc<i32>` which is not `Send`
12  |     // A `.await` point
13  |     yield_now().await;
    |                 ^^^^^ 
    |   await occurs here, with `non_send` maybe used later
note: required by a bound in `tokio::spawn`
    |
164 |     pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
    |            ----- required by a bound in this function
165 |     where
166 |         F: Future + Send + 'static,
    |                     ^^^^ required by this bound in `spawn`
```

Bunun neden böyle olduğunu anlamak için Rust'ın asenkron modeline dair
anlayışımızı biraz geliştirmemiz gerekiyor.

## `Future` Trait'i {#the-future-trait-1}

Daha önce `async` fonksiyonların **future**'lar, yani `Future` trait'ini implement eden
türler döndürdüğünü söylemiştik. Bir future'ı bir **durum makinesi** olarak düşünebilirsin.
Future iki durumdan birindedir:

- **pending** (beklemede): hesaplama henüz bitmedi.
- **ready** (hazır): hesaplama bitti, işte çıktısı.

Bu, trait tanımında şöyle kodlanmıştır:

```rust
trait Future {
    type Output;
    
    // Şimdilik `Pin` ve `Context`'i görmezden gel
    fn poll(
      self: Pin<&mut Self>, 
      cx: &mut Context<'_>
    ) -> Poll<Self::Output>;
}
```

### `poll` {#poll}

`poll` metodu `Future` trait'inin kalbidir.\
Bir future kendi başına hiçbir şey yapmaz. İlerleme kaydetmesi için **poll** edilmesi gerekir.\
`poll`'u çağırdığında future'dan biraz iş yapmasını istiyorsun.
`poll` ilerleme kaydetmeye çalışır ve ardından şunlardan birini döndürür:

- `Poll::Pending`: future henüz hazır değil. Daha sonra `poll`'u tekrar çağırman gerekir.
- `Poll::Ready(value)`: future bitti. `value`, `Self::Output` türündeki
  hesaplamanın sonucudur.

`Future::poll`, `Poll::Ready` döndürdükten sonra future tekrar poll edilmemelidir: future
tamamlanmıştır, yapılacak bir şey kalmamıştır.

### Runtime'ın Rolü {#the-role-of-the-runtime}

Doğrudan poll çağırman nadiren gerekir, belki de hiç gerekmez.\
Bu, async runtime'ının işidir: future'larının ilerleyebildikleri her an ilerlemesini sağlamak için
gereken tüm bilgiye (`poll`'un imzasındaki `Context`) sahiptir.

## `async fn` Ve Future'lar {#async-fn-and-futures}

Üst düzey arayüz olan asenkron fonksiyonlarla çalıştık.\
Şimdi de alt düzey primitive'e, `Future trait`'e baktık.

Bunlar birbiriyle nasıl ilişkili?

Bir fonksiyonu asenkron olarak her işaretlediğinde, o fonksiyon bir future döndürür.
Derleyici, asenkron fonksiyonunun gövdesini bir **durum makinesine** dönüştürür:
her `.await` noktası için bir durum.

`Rc` örneğimize geri dönelim:

```rust
use std::rc::Rc;
use tokio::task::yield_now;

async fn example() {
    let non_send = Rc::new(1);
    yield_now().await;
    println!("{}", non_send);
}
```

Derleyici bunu aşağı yukarı şuna benzeyen bir enum'a dönüştürür:

```rust
pub enum ExampleFuture {
    NotStarted,
    YieldNow(Rc<i32>),
    Terminated,
}
```

`example` çağrıldığında `ExampleFuture::NotStarted` döndürür. Future henüz hiç
poll edilmemiştir, dolayısıyla hiçbir şey olmamıştır.\
Runtime onu ilk kez poll ettiğinde `ExampleFuture` bir sonraki `.await` noktasına kadar
ilerler: durum makinesinin `ExampleFuture::YieldNow(Rc<i32>)` aşamasında durur
ve `Poll::Pending` döndürür.\
Tekrar poll edildiğinde kalan kodu (`println!`) çalıştırır ve
`Poll::Ready(())` döndürür.

Durum makinesi gösterimine, yani `ExampleFuture`'a baktığında,
`example`'ın neden `Send` olmadığı artık açıktır: bir `Rc` tutar, bu nedenle
`Send` olamaz.

## Yield Point'ler {#yield-points}

`example` ile az önce gördüğün gibi, her `.await` noktası bir future'ın yaşam döngüsünde
yeni bir ara durum oluşturur.\
Bu yüzden `.await` noktaları **yield point**'ler olarak da bilinir: future'ın, onu poll eden
runtime'a _kontrolü geri yield eder_; bu da runtime'ın onu duraklatmasına ve (gerekirse)
çalıştırılmak üzere başka bir task'i (görevi) zamanlamasına olanak tanır; böylece birden çok
cephede eşzamanlı olarak ilerleme sağlanır.

Yield etmenin önemine ilerleyen bir kısımda geri döneceğiz.
