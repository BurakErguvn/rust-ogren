# Referanslar {#references}

Peki `&String` ya da `&mut String` gibi referanslar? Bellekte nasıl temsil edilirler?

Rust'taki çoğu referans[^fat] bellekte, bir bellek konumuna işaret eden bir pointer (işaretçi) olarak temsil edilir.\
Buradan, boyutlarının bir pointer'ın boyutuyla, yani bir `usize` ile aynı olduğu sonucu çıkar.

Bunu `std::mem::size_of` kullanarak doğrulayabilirsin:

```rust
assert_eq!(std::mem::size_of::<&String>(), 8);
assert_eq!(std::mem::size_of::<&mut String>(), 8);
```

Özellikle bir `&String`, `String`'in meta verilerinin depolandığı bellek konumuna işaret eden bir pointer'dır.\
Şu kod parçasını çalıştırırsan:

```rust
let s = String::from("Hey");
let r = &s;
```

bellekte şuna benzer bir şey elde edersin:

```
           --------------------------------------
           |                                    |
      +----v----+--------+----------+      +----|----+
Stack | pointer | length | capacity |      | pointer |
      |  |      |   3    |    5     |      |         |
      +--|  ----+--------+----------+      +---------+
         |          s                           r
         |
         v
       +---+---+---+---+---+
Heap   | H | e | y | ? | ? |
       +---+---+---+---+---+
```

İstersen buna, heap'te ayrılmış veriye işaret eden bir pointer'a işaret eden bir pointer diyebilirsin.
Aynısı `&mut String` için de geçerlidir.

## Tüm Pointer'lar Heap'i Göstermez {#not-all-pointers-point-to-the-heap}

Yukarıdaki örnek bir şeyi netleştirmiş olmalı: tüm pointer'lar heap'i göstermez.\
Yalnızca bir bellek konumunu gösterirler; bu konum heap'te _olabilir_, ama olmak zorunda değildir.

[^fat]: [Kursun ilerleyen kısımlarında](../04_traits/06_str_slice.md) **fat pointer**'lardan, yani ek meta veri
taşıyan pointer'lardan bahsedeceğiz. Adından da anlaşılacağı gibi, bunlar bu bölümde
ele aldığımız ve **thin pointer** olarak da bilinen pointer'lardan daha büyüktür.
