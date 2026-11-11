# `TryFrom` Ve `TryInto` {#tryfrom-and-tryinto}

Önceki bölümde, Rust'ın **başarısız olamayan** tür dönüşümleri için idiomatik arayüzleri olan
[`From` ve `Into` trait'lerine](../04_traits/09_from.md) baktık.\
Peki ya dönüşümün başarılı olması garanti değilse?

Artık hatalar hakkında, `From` ve `Into`'nun **başarısız olabilen** karşılıklarını ele alacak kadar bilgimiz var:
`TryFrom` ve `TryInto`.

## `TryFrom` Ve `TryInto` {#tryfrom-and-tryinto-1}

Hem `TryFrom` hem de `TryInto`, tıpkı `From` ve `Into` gibi `std::convert` modülünde tanımlanmıştır.

```rust
pub trait TryFrom<T>: Sized {
    type Error;
    fn try_from(value: T) -> Result<Self, Self::Error>;
}

pub trait TryInto<T>: Sized {
    type Error;
    fn try_into(self) -> Result<T, Self::Error>;
}
```

`From`/`Into` ile `TryFrom`/`TryInto` arasındaki temel fark, ikincilerin bir `Result` türü döndürmesidir.\
Bu, dönüşümün başarısız olmasına, yani panic'lemek yerine bir hata döndürmesine olanak tanır.

## `Self::Error` {#selferror}

Hem `TryFrom`'un hem de `TryInto`'nun bir `Error` associated type'ı (ilişkili türü) vardır.
Bu, her implementation'ın kendi hata türünü, ideal olarak da denenen dönüşüm için en uygun olanını
belirlemesine olanak tanır.

`Self::Error`, trait'in kendisinde tanımlanan `Error` associated type'ına başvurmanın bir yoludur.

## İkizlik {#duality}

Tıpkı `From` ve `Into` gibi, `TryFrom` ve `TryInto` da ikiz trait'lerdir.\
Bir tür için `TryFrom`'u implement edersen, `TryInto`'yu bedavaya elde edersin.
