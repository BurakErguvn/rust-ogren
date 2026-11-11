# `thiserror` {#thiserror}

Biraz dolambaçlı bir yol oldu, değil mi? Ama gerekliydi!\
Şimdi asıl konumuza dönelim: özel hata türleri ve `thiserror`.

## Özel Hata Türleri {#custom-error-types}

Özel bir hata türü için `Error` trait'ini "elle" nasıl implement edeceğimizi gördük.\
Bunu kod tabanındaki hata türlerinin çoğu için yapmak zorunda olduğunu düşün. Bu epey boilerplate demek, değil mi?

[`thiserror`](https://docs.rs/thiserror/latest/thiserror/) kullanarak boilerplate'in bir kısmını kaldırabiliriz;
bu, özel hata türlerinin oluşturulmasını basitleştirmek için bir **procedural macro** sağlayan bir Rust crate'idir.

```rust
#[derive(thiserror::Error, Debug)]
enum TicketNewError {
    #[error("{0}")]
    TitleError(String),
    #[error("{0}")]
    DescriptionError(String),
}
```

## Kendi Makrolarını Yazabilirsin {#you-can-write-your-own-macros}

Şimdiye kadar gördüğümüz tüm `derive` macro'lar Rust standart kütüphanesi tarafından sağlanıyordu.\
`thiserror::Error`, **üçüncü taraf** bir `derive` macro'nun ilk örneğidir.

`derive` macro'lar, derleme zamanında Rust kodu üretmenin bir yolu olan **procedural macro**'ların bir alt kümesidir.
Bu kursta bir procedural macro'nun nasıl yazılacağının ayrıntılarına girmeyeceğiz, ama kendi procedural macro'nu
yazabileceğini bilmek önemli!\
Bu, daha ileri düzey bir Rust kursunda ele alınacak bir konu.

## Özel Söz Dizimi {#custom-syntax}

Her procedural macro kendi söz dizimini tanımlayabilir; bu söz dizimi genellikle crate'in dokümantasyonunda açıklanır.
`thiserror` örneğinde şunlar var:

- `#[derive(thiserror::Error)]`: `thiserror`'ın yardımıyla özel bir hata türü için `Error` trait'ini derive etmenin söz dizimidir.
- `#[error("{0}")]`: özel hata türünün her variant'ı için bir `Display` implementation'ı tanımlamanın söz dizimidir.
  Hata gösterildiğinde `{0}`, variant'ın sıfırıncı alanıyla (bu durumda `String`) değiştirilir.
