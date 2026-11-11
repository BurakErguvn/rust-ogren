# Operator Overloading (Operatör Aşırı Yükleme) {#operator-overloading}

Trait'lerin ne olduğunu temel düzeyde anladığımıza göre, **operator overloading**'e geri dönelim.
Operator overloading, `+`, `-`, `*`, `/`, `==`, `!=` vb. operatörler için özel davranış tanımlayabilme yeteneğidir.

## Operatörler Birer Trait'tir {#operators-are-traits}

Rust'ta operatörler birer trait'tir.\
Her operatör için, o operatörün davranışını tanımlayan karşılık gelen bir trait vardır.
O trait'i kendi türün için implement ederek, karşılık gelen operatörlerin kullanımının **kilidini açarsın**.

Örneğin [`PartialEq` trait'i](https://doc.rust-lang.org/std/cmp/trait.PartialEq.html),
`==` ve `!=` operatörlerinin davranışını tanımlar:

```rust
// Rust'ın standart kütüphanesindeki `PartialEq` trait tanımı
// (Şimdilik *biraz* basitleştirilmiştir)
pub trait PartialEq {
    // Gerekli metot
    //
    // `Self`, "trait'i implement eden tür" anlamına gelen
    // bir Rust anahtar kelimesidir
    fn eq(&self, other: &Self) -> bool;

    // Sağlanan metot
    fn ne(&self, other: &Self) -> bool { ... }
}
```

`x == y` yazdığında derleyici, `x` ve `y`'nin türleri için `PartialEq` trait'inin bir implementation'ını arar
ve `x == y`'yi `x.eq(y)` ile değiştirir. Bu bir syntactic sugar'dır (sözdizimsel şeker)!

Ana operatörler için karşılıklar şöyledir:

| Operatör                 | Trait                                                                   |
| ------------------------ | ----------------------------------------------------------------------- |
| `+`                      | [`Add`](https://doc.rust-lang.org/std/ops/trait.Add.html)               |
| `-`                      | [`Sub`](https://doc.rust-lang.org/std/ops/trait.Sub.html)               |
| `*`                      | [`Mul`](https://doc.rust-lang.org/std/ops/trait.Mul.html)               |
| `/`                      | [`Div`](https://doc.rust-lang.org/std/ops/trait.Div.html)               |
| `%`                      | [`Rem`](https://doc.rust-lang.org/std/ops/trait.Rem.html)               |
| `==` ve `!=`             | [`PartialEq`](https://doc.rust-lang.org/std/cmp/trait.PartialEq.html)   |
| `<`, `>`, `<=` ve `>=`   | [`PartialOrd`](https://doc.rust-lang.org/std/cmp/trait.PartialOrd.html) |

Aritmetik operatörler [`std::ops`](https://doc.rust-lang.org/std/ops/index.html) modülünde,
karşılaştırma operatörleri ise [`std::cmp`](https://doc.rust-lang.org/std/cmp/index.html) modülünde bulunur.

## Varsayılan Implementation'lar {#default-implementations}

`PartialEq::ne` üzerindeki yorum, "`ne` sağlanan bir metottur" diyor.\
Bu, `PartialEq` trait'inin, trait tanımında `ne` için bir **varsayılan implementation** sağladığı anlamına gelir—tanımı
gösteren kod parçasındaki atlanmış `{ ... }` bloğu.\
Atlanmış bloğu açarsak şöyle görünür:

```rust
pub trait PartialEq {
    fn eq(&self, other: &Self) -> bool;

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
```

Beklediğin gibi: `ne`, `eq` metodunun olumsuzlamasıdır.\
Varsayılan bir implementation sağlandığı için, kendi türün için `PartialEq` implement ederken `ne`'yi implement etmeyi atlayabilirsin.
Yalnızca `eq` metodunu implement etmen yeterlidir:

```rust
struct WrappingU8 {
    inner: u8,
}

impl PartialEq for WrappingU8 {
    fn eq(&self, other: &WrappingU8) -> bool {
        self.inner == other.inner
    }
    
    // Burada `ne` implementation'ı yok
}
```

Yine de varsayılan implementation'ı kullanmak zorunda değilsin.
Trait'i implement ederken onu override etmeyi seçebilirsin:

```rust
struct MyType;

impl PartialEq for MyType {
    fn eq(&self, other: &MyType) -> bool {
        // Özel implementation
    }

    fn ne(&self, other: &MyType) -> bool {
        // Özel implementation
    }
}
```
