# `'static` {#static}

Önceki egzersizde vektörden bir slice (dilim) ödünç almayı denediysen,
muhtemelen şuna benzer bir derleyici hatası almışsındır:

```text
error[E0597]: `v` does not live long enough
   |
11 | pub fn sum(v: Vec<i32>) -> i32 {
   |            - binding `v` declared here
...
15 |     let right = &v[split_point..];
   |                  ^ borrowed value does not live long enough
16 |     let left_handle = spawn(move || left.iter().sum::<i32>());
   |                             -------------------------------- 
                     argument requires that `v` is borrowed for `'static`
19 | }
   |  - `v` dropped here while still borrowed
```

`argument requires that v is borrowed for 'static`, bu ne anlama geliyor?

`'static` lifetime'ı (yaşam süresi), Rust'ta özel bir lifetime'dır.\
Değerin programın tüm çalışma süresi boyunca geçerli olacağı anlamına gelir.

## Ayrık Thread'ler {#detached-threads}

`thread::spawn` ile başlatılan bir thread (iş parçacığı), onu spawn eden thread'den **daha uzun yaşayabilir**.\
Örneğin:

```rust
use std::thread;

fn f() {
    thread::spawn(|| {
        thread::spawn(|| {
            loop {
                thread::sleep(std::time::Duration::from_secs(1));
                println!("Hello from the detached thread!");
            }
        });
    });
}
```

Bu örnekte, spawn edilen ilk thread de kendi sırasında
her saniye bir mesaj yazdıran bir alt thread spawn eder.\
İlk thread ardından biter ve sonlanır. Bu olduğunda
alt thread'i, sürecin tamamı çalıştığı sürece
**çalışmaya devam eder**.\
Rust jargonunda, alt thread'in üst thread'inden **daha uzun yaşadığını**
söyleriz.

## `'static` Lifetime'ı {#static-lifetime}

Spawn edilen bir thread:

- onu spawn eden thread'den (üst thread'inden) daha uzun yaşayabildiği
- program sonlanana kadar çalışabildiği

için, program sonlanmadan önce drop edilebilecek hiçbir değeri ödünç almamalıdır;
bu kısıtlamayı ihlal etmek bizi bir use-after-free bug'ına maruz bırakırdı.\
Bu yüzden `std::thread::spawn`'ın imzası, kendisine verilen closure'ın
`'static` lifetime'ına sahip olmasını şart koşar:

```rust
pub fn spawn<F, T>(f: F) -> JoinHandle<T> 
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static
{
    // [..]
}
```

## `'static` (Sadece) Referanslarla İlgili Değildir {#static-is-not-just-about-references}

Rust'ta yalnızca referansların değil, tüm değerlerin bir lifetime'ı vardır.

Özellikle, verisine sahip olan bir tür (`Vec` ya da `String` gibi)
`'static` kısıtlamasını karşılar: ona sahipsen, onu oluşturan fonksiyon
döndükten sonra bile onunla istediğin kadar
çalışmaya devam edebilirsin.

Dolayısıyla `'static`'i şunu söylemenin bir yolu olarak yorumlayabilirsin:

- Bana owned (sahipli) bir değer ver
- Bana programın tüm çalışma süresi boyunca geçerli olan bir referans ver

Önceki egzersizdeki sorunu ilk yaklaşımla çözdün:
orijinal vektörün sol ve sağ kısımlarını tutacak yeni vektörler için bellek ayırarak;
bu vektörler daha sonra spawn edilen thread'lere move edildi (taşındı).

## `'static` Referanslar {#static-references}

İkinci duruma, yani programın tüm çalışma süresi boyunca geçerli olan referanslara
bakalım.

### Statik Veri {#static-data}

En yaygın durum, string literal'ler gibi **statik veriye** yönelik bir referanstır:

```rust
let s: &'static str = "Hello world!";
```

String literal'ler derleme zamanında bilindiği için Rust onları çalıştırılabilir dosyanın _içinde_,
**salt okunur veri segmenti** olarak bilinen bir bölgede saklar.
Dolayısıyla o bölgeyi gösteren tüm referanslar, program çalıştığı sürece
geçerli olur; `'static` sözleşmesini karşılarlar.

## İleri Okuma {#further-reading}

- [Veri segmenti](https://en.wikipedia.org/wiki/Data_segment)
