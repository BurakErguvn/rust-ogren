# Thread'ler (İş Parçacıkları) {#threads}

Çok thread'li kod yazmaya başlamadan önce bir adım geri çekilelim ve thread'lerin ne olduğunu
ve neden onları kullanmak isteyebileceğimizi konuşalım.

## Thread Nedir? {#what-is-a-thread}

**Thread**, altta yatan işletim sisteminin yönettiği bir yürütme bağlamıdır.\
Her thread'in kendi stack'i ve instruction pointer'ı (komut işaretçisi) vardır.

Tek bir **süreç** (process) birden fazla thread'i yönetebilir.
Bu thread'ler aynı bellek alanını paylaşır, yani aynı verilere erişebilirler.

Thread'ler **mantıksal** bir yapıdır. Sonuçta, **fiziksel** yürütme birimi olan bir CPU çekirdeğinde
aynı anda yalnızca bir komut kümesi çalıştırabilirsin.\
CPU çekirdeğinden çok daha fazla thread olabileceği için, herhangi bir anda hangi thread'in çalışacağına
işletim sisteminin **scheduler**'ı (zamanlayıcı) karar verir;
işlem hacmini ve yanıt verebilirliği en üst düzeye çıkarmak için CPU zamanını thread'ler arasında paylaştırır.

## `main` {#main}

Bir Rust programı başladığında tek bir thread üzerinde, **ana thread** üzerinde çalışır.\
Bu thread işletim sistemi tarafından oluşturulur ve `main` fonksiyonunu çalıştırmaktan
sorumludur.

```rust
use std::thread;
use std::time::Duration;

fn main() {
    loop {
        thread::sleep(Duration::from_secs(2));
        println!("Hello from the main thread!");
    }
}
```

## `std::thread` {#stdthread}

Rust'ın standart kütüphanesi, thread oluşturmanı ve yönetmeni sağlayan
bir modül sunar: `std::thread`.

### `spawn` {#spawn}

Yeni thread'ler oluşturmak ve üzerlerinde kod çalıştırmak için `std::thread::spawn` kullanabilirsin.

Örneğin:

```rust
use std::thread;
use std::time::Duration;

fn main() {
    let handle = thread::spawn(|| {
        loop {
            thread::sleep(Duration::from_secs(1));
            println!("Hello from a thread!");
        }
    });
    
    loop {
        thread::sleep(Duration::from_secs(2));
        println!("Hello from the main thread!");
    }
}
```

Bu programı [Rust playground'unda](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=afedf7062298ca8f5a248bc551062eaa) çalıştırırsan
ana thread'in ve spawn edilen thread'in eşzamanlı olarak çalıştığını görürsün.\
Her thread diğerinden bağımsız olarak ilerler.

### Sürecin Sonlanması {#process-termination}

Ana thread bittiğinde sürecin tamamı sonlanır.\
Spawn edilen bir thread, kendisi bitene ya da ana thread bitene kadar çalışmaya devam eder.

```rust
use std::thread;
use std::time::Duration;

fn main() {
    let handle = thread::spawn(|| {
        loop {
            thread::sleep(Duration::from_secs(1));
            println!("Hello from a thread!");
        }
    });

    thread::sleep(Duration::from_secs(5));
}
```

Yukarıdaki örnekte "Hello from a thread!" mesajının yaklaşık beş kez yazdırıldığını görmeyi bekleyebilirsin.\
Ardından ana thread biter (`sleep` çağrısı döndüğünde) ve sürecin tamamı sonlandığı için spawn edilen thread
de sonlandırılır.

### `join` {#join}

`spawn`'ın döndürdüğü `JoinHandle` üzerinde `join` metodunu çağırarak spawn edilen bir thread'in bitmesini de bekleyebilirsin.

```rust
use std::thread;
fn main() {
    let handle = thread::spawn(|| {
        println!("Hello from a thread!");
    });

    handle.join().unwrap();
}
```

Bu örnekte ana thread, sonlanmadan önce spawn edilen thread'in bitmesini bekler.\
Bu, iki thread arasında bir tür **senkronizasyon** getirir: program sonlanmadan önce
"Hello from a thread!" mesajının yazdırıldığını görmen garantidir, çünkü ana thread, spawn edilen thread
bitene kadar sonlanmaz.
