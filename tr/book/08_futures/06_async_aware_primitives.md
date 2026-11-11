# Async-Aware (Async'e Duyarlı) Primitive'ler {#async-aware-primitives}

`tokio`'nun dokümantasyonuna göz atarsan, standart kütüphanedekileri "yansıtan" ama
asenkron bir dokunuşa sahip pek çok tür sağladığını fark edeceksin:
kilitler, kanallar, zamanlayıcılar ve daha fazlası.

Asenkron bir bağlamda çalışırken bu asenkron alternatifleri
senkron karşılıklarına tercih etmelisin.

Nedenini anlamak için önceki bölümde incelediğimiz karşılıklı dışlamalı kilit olan
`Mutex`'e bir göz atalım.

## Vaka Çalışması: `Mutex` {#case-study-mutex}

Basit bir örneğe bakalım:

```rust
use std::sync::{Arc, Mutex};

async fn run(m: Arc<Mutex<Vec<u64>>>) {
    let guard = m.lock().unwrap();
    http_call(&guard).await;
    println!("Sent {:?} to the server", &guard);
    // `guard` burada drop edilir
}

/// `v`'yi bir HTTP çağrısının gövdesi olarak kullan.
async fn http_call(v: &[u64]) {
  // [...]
}
```

### `std::sync::MutexGuard` Ve Yield Point'ler {#stdsyncmutexguard-and-yield-points}

Bu kod derlenir, ama tehlikelidir.

Asenkron bir bağlamda, `std`'deki bir `Mutex` üzerinde kilidi almaya çalışıyoruz.
Ardından ortaya çıkan `MutexGuard`'ı bir yield point boyunca (`http_call` üzerindeki
`.await`) tutuyoruz.

Tek thread'li bir runtime üzerinde `run`'ı eşzamanlı olarak çalıştıran iki task (görev)
olduğunu düşünelim. Şu zamanlama olayları serisini gözlemliyoruz:

```text
     Task A          Task B
        | 
  Acquire lock
Yields to runtime
        | 
        +--------------+
                       |
             Tries to acquire lock
```

Bir deadlock (kilitlenme) oluştu. Task B kilidi asla alamayacak, çünkü kilit şu anda
task A'nın elinde; task A ise kilidi bırakmadan önce runtime'a yield etti ve runtime
task B'yi preempt edemediği (kesintiye uğratamadığı) için bir daha zamanlanmayacak.

### `tokio::sync::Mutex` {#tokiosyncmutex}

Sorunu `tokio::sync::Mutex`'e geçerek çözebilirsin:

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

async fn run(m: Arc<Mutex<Vec<u64>>>) {
    let guard = m.lock().await;
    http_call(&guard).await;
    println!("Sent {:?} to the server", &guard);
    // `guard` burada drop edilir
}
```

Kilidi almak artık asenkron bir işlemdir; ilerleme kaydedemezse kontrolü runtime'a
geri yield eder.\
Önceki senaryoya dönersek, şunlar olur:

```text
       Task A          Task B
          | 
  Acquires the lock
  Starts `http_call`
  Yields to runtime
          | 
          +--------------+
                         |
             Tries to acquire the lock
              Cannot acquire the lock
                 Yields to runtime
                         |
          +--------------+
          |
`http_call` completes      
  Releases the lock
   Yield to runtime
          |
          +--------------+
                         |
                 Acquires the lock
                       [...]
```

Her şey yolunda!

### Çok Thread'li Olmak Seni Kurtarmaz {#multithreaded-wont-save-you}

Önceki örneğimizde çalıştırma bağlamı olarak tek thread'li bir runtime kullandık,
ama aynı risk çok thread'li bir runtime kullanırken de
devam eder.\
Tek fark, deadlock oluşturmak için gereken eşzamanlı task sayısındadır:
tek thread'li bir runtime'da 2 task yeterlidir; çok thread'li bir runtime'da ise
`N+1` task gerekir; burada `N`, runtime thread'lerinin (iş parçacıklarının) sayısıdır.

### Dezavantajlar {#downsides}

Async-aware bir `Mutex`'e sahip olmanın bir performans bedeli vardır.\
Kilidin ciddi bir çekişme altında olmadığından eminsen
_ve_ onu asla bir yield point boyunca tutmamaya dikkat ediyorsan, asenkron bir
bağlamda yine de `std::sync::Mutex` kullanabilirsin.

Ama performans kazancını, üstleneceğin canlılık (liveness) riskine karşı
iyice tart.

## Diğer Primitive'ler {#other-primitives}

Örnek olarak `Mutex`'i kullandık, ama aynı şey `RwLock`, semaforlar vb. için de geçerlidir.\
Asenkron bir bağlamda çalışırken sorun riskini en aza indirmek için async-aware
sürümleri tercih et.
