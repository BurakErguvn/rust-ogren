# Bellek Düzeni {#memory-layout}

Ownership'e (sahipliğe) ve referanslara işlevsel bir bakış açısıyla baktık—onlarla neler yapıp neler yapamayacağına.
Şimdi kaputun altına bakmanın tam zamanı: hadi **bellek** hakkında konuşalım.

## Stack Ve Heap {#stack-and-heap}

Bellekten bahsederken insanların sıklıkla **stack** ve **heap**'ten bahsettiğini duyarsın.\
Bunlar, programların veri depolamak için kullandığı iki farklı bellek bölgesidir.

Stack ile başlayalım.

## Stack {#stack}

**Stack**, bir **LIFO** (Last In, First Out; son giren ilk çıkar) veri yapısıdır.\
Bir fonksiyon çağırdığında stack'in en üstüne yeni bir **stack frame** eklenir. Bu stack frame, fonksiyonun
argümanlarını, yerel değişkenlerini ve birkaç "kayıt tutma" değerini depolar.\
Fonksiyon döndüğünde stack frame stack'ten çıkarılır[^stack-overflow].

```text
+-----------------+
| frame for func1 |
+-----------------+
        |
        | func2 is 
        | called
        v
+-----------------+
| frame for func2 |
+-----------------+
| frame for func1 |
+-----------------+
        |
        | func2  
        | returns
        v
+-----------------+
| frame for func1 |
+-----------------+
```

İşlevsel açıdan bakıldığında, stack allocation'ı (bellek ayırma) ve serbest bırakma işlemleri **çok hızlıdır**.\
Veriyi her zaman stack'in en üstüne ekleyip en üstünden çıkarıyoruz, bu yüzden boş bellek aramamıza gerek yok.
Parçalanma konusunda da endişelenmemize gerek yok: stack tek bir bitişik bellek bloğudur.

### Rust {#rust}

Rust çoğu zaman veriler için stack'te bellek ayırır.\
Bir fonksiyonda `u32` türünde bir girdi argümanın mı var? O 32 bit stack'te olacak.\
`i64` türünde yerel bir değişken mi tanımlıyorsun? O 64 bit stack'te olacak.\
Hepsi oldukça sorunsuz çalışır, çünkü bu tam sayıların boyutu derleme zamanında bilinir; bu yüzden
derlenmiş program, stack'te onlar için ne kadar yer ayırması gerektiğini bilir.

### `std::mem::size_of` {#stdmemsize_of}

Bir türün stack'te ne kadar yer kaplayacağını
[`std::mem::size_of`](https://doc.rust-lang.org/std/mem/fn.size_of.html) fonksiyonunu kullanarak doğrulayabilirsin.

Örneğin bir `u8` için:

```rust
// Bu tuhaf görünümlü söz dizimini (`::<u8>`) daha sonra açıklayacağız.
// Şimdilik onu görmezden gel.
assert_eq!(std::mem::size_of::<u8>(), 1);
```

1 mantıklı, çünkü bir `u8` 8 bit, yani 1 byte uzunluğundadır.

[^stack-overflow]: İç içe fonksiyon çağrıların varsa, her fonksiyon çağrıldığında verisini stack'e ekler, ama
en içteki fonksiyon dönene kadar onu çıkarmaz.
Çok fazla iç içe fonksiyon çağrın varsa stack alanın tükenebilir—stack sonsuz değildir!
Buna [**stack overflow**](https://en.wikipedia.org/wiki/Stack_overflow) denir.
