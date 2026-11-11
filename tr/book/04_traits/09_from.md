# `From` Ve `Into` {#from-and-into}

String yolculuğumuzun başladığı yere geri dönelim:

```rust
let ticket = Ticket::new(
    "A title".into(), 
    "A description".into(), 
    "To-Do".into()
);
```

`.into()`'nun burada ne yaptığını çözümlemeye başlamak için artık yeterince şey biliyoruz.

## Sorun {#the-problem}

`new` metodunun imzası şöyle:

```rust
impl Ticket {
    pub fn new(
        title: String, 
        description: String, 
        status: String
    ) -> Self {
        // [...]
    }
}
```

String literal'lerin (`"A title"` gibi) `&str` türünde olduğunu da gördük.\
Burada bir tür uyuşmazlığı var: bir `String` bekleniyor, ama elimizde bir `&str` var.
Bu sefer bizi kurtaracak sihirli bir coercion (örtük tür dönüşümü) yok; **bir dönüşüm yapmamız** gerekiyor.

## `From` Ve `Into` {#from-and-into-1}

Rust standart kütüphanesi, `std::convert` modülünde **başarısız olamayan dönüşümler** için iki trait tanımlar:
`From` ve `Into`.

```rust
pub trait From<T>: Sized {
    fn from(value: T) -> Self;
}

pub trait Into<T>: Sized {
    fn into(self) -> T;
}
```

Bu trait tanımları, daha önce görmediğimiz birkaç kavramı sergiliyor: **supertrait**'ler ve **örtük trait bound**'lar.
Önce bunları açalım.

### Supertrait / Subtrait {#supertrait--subtrait}

`From: Sized` söz dizimi, `From`'un `Sized`'ın bir **subtrait**'i olduğunu ifade eder: `From`'u
implement eden her tür `Sized`'ı da implement etmelidir.
Bunu başka bir şekilde, `Sized`'ın `From`'un bir **supertrait**'i olduğunu söyleyerek de ifade edebilirsin.

### Örtük Trait Bound'lar {#implicit-trait-bounds}

Ne zaman generic bir tür parametren olsa, derleyici örtük olarak onun `Sized` olduğunu varsayar.

Örneğin:

```rust
pub struct Foo<T> {
    inner: T,
}
```

aslında şuna eşdeğerdir:

```rust
pub struct Foo<T: Sized> 
{
    inner: T,
}
```

`From<T>` söz konusu olduğunda, trait tanımı şuna eşdeğerdir:

```rust
pub trait From<T: Sized>: Sized {
    fn from(value: T) -> Self;
}
```

Başka bir deyişle, ilk bound örtük olsa da _hem_ `T` hem de `From<T>`'yi implement eden tür
`Sized` olmalıdır.

### Negative Trait Bound'lar {#negative-trait-bounds}

Örtük `Sized` bound'undan bir **negative trait bound** ile vazgeçebilirsin:

```rust
pub struct Foo<T: ?Sized> {
    //            ^^^^^^^
    //            Bu bir negative trait bound'dur
    inner: T,
}
```

Bu söz dizimi "`T`, `Sized` olabilir de olmayabilir de" diye okunur ve `T`'yi
bir DST'ye (dinamik boyutlu tür) bağlamana (örneğin `Foo<str>`) olanak tanır. Yine de bu özel bir durumdur: negative trait bound'lar yalnızca `Sized`'a özgüdür,
onları başka trait'lerle kullanamazsın.

## `&str`'den `String`'e {#str-to-string}

[`std`'nin dokümantasyonunda](https://doc.rust-lang.org/std/convert/trait.From.html#implementors)
hangi `std` türlerinin `From` trait'ini implement ettiğini görebilirsin.\
`String`'in `From<&str> for String`'i implement ettiğini göreceksin. Böylece şunu yazabiliriz:

```rust
let title = String::from("A title");
```

Yine de çoğunlukla `.into()` kullanıyorduk.\
[`Into`'yu implement eden türlere](https://doc.rust-lang.org/std/convert/trait.Into.html#implementors)
bakarsan `Into<String> for &str` bulamazsın. Neler oluyor?

`From` ve `Into` **ikiz trait**'lerdir.\
Daha açık söylemek gerekirse, `Into`, `From`'u implement eden her tür için bir **blanket implementation** ile implement edilmiştir:

```rust
impl<T, U> Into<U> for T
where
    U: From<T>,
{
    fn into(self) -> U {
        U::from(self)
    }
}
```

Bir `U` türü `From<T>`'yi implement ediyorsa, `Into<U> for T` otomatik olarak implement edilir.
`let title = "A title".into();` yazabilmemizin nedeni budur.

## `.into()` {#into}

Ne zaman `.into()` görsen, türler arasında bir dönüşüme tanıklık ediyorsun.\
Peki hedef tür nedir?

Çoğu durumda hedef tür şunlardan biridir:

- Bir fonksiyonun/metodun imzasıyla belirtilir (örneğin yukarıdaki örneğimizdeki `Ticket::new`)
- Değişken bildiriminde bir tür belirtimiyle belirtilir (örneğin `let title: String = "A title".into();`)

Derleyici hedef türü bağlamdan belirsizlik olmadan çıkarsayabildiği sürece `.into()` doğrudan çalışır.
