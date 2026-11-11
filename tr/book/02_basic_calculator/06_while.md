# Döngüler, 1. Kısım: `while` {#loops-part-1-while}

`factorial` implementation'ın özyineleme kullanmak zorunda kaldı.\
Bu sana doğal gelebilir, özellikle fonksiyonel programlama geçmişinden geliyorsan.
Ya da C veya Python gibi daha imperatif dillere alışkınsan tuhaf gelebilir.

Aynı işlevselliği bunun yerine bir **döngü** kullanarak nasıl implement edebileceğine bakalım.

## `while` Döngüsü {#the-while-loop}

`while` döngüsü, bir **koşul** doğru olduğu sürece bir kod bloğunu çalıştırmanın bir yoludur.\
Genel söz dizimi şöyledir:

```rust
while <condition> {
    // çalıştırılacak kod
}
```

Örneğin 1'den 5'e kadar olan sayıları toplamak isteyebiliriz:

```rust
let sum = 0;
let i = 1;
// "i, 5'ten küçük ya da 5'e eşit olduğu sürece"
while i <= 5 {
    // `+=`, `sum = sum + i` için bir kısaltmadır
    sum += i;
    i += 1;
}
```

Bu, `i` artık 5'ten küçük ya da 5'e eşit olmayana kadar `i`'ye 1, `sum`'a da `i` eklemeye devam eder.

## `mut` Anahtar Kelimesi {#the-mut-keyword}

Yukarıdaki örnek bu haliyle derlenmez. Şuna benzer bir hata alırsın:

```text
error[E0384]: cannot assign twice to immutable variable `sum`
 --> src/main.rs:7:9
  |
2 |     let sum = 0;
  |         ---
  |         |
  |         first assignment to `sum`
  |         help: consider making this binding mutable: `mut sum`
...
7 |         sum += i;
  |         ^^^^^^^^ cannot assign twice to immutable variable

error[E0384]: cannot assign twice to immutable variable `i`
 --> src/main.rs:8:9
  |
3 |     let i = 1;
  |         -
  |         |
  |         first assignment to `i`
  |         help: consider making this binding mutable: `mut i`
...
8 |         i += 1;
  |         ^^^^^^ cannot assign twice to immutable variable
```

Bunun nedeni, Rust'ta değişkenlerin varsayılan olarak **immutable** (değiştirilemez) olmasıdır.\
Bir değer atandıktan sonra onu değiştiremezsin.

Değişikliklere izin vermek istiyorsan, değişkeni `mut` anahtar kelimesini kullanarak **mutable** (değiştirilebilir)
olarak bildirmen gerekir:

```rust
// `sum` ve `i` artık mutable!
let mut sum = 0;
let mut i = 1;

while i <= 5 {
    sum += i;
    i += 1;
}
```

Bu kod hatasız derlenir ve çalışır.

## İleri Okuma {#further-reading}

- [`while` döngüsü dokümantasyonu](https://doc.rust-lang.org/std/keyword.while.html)
