# Error Trait'i {#error-trait}

## Hata Raporlama {#error-reporting}

Önceki egzersizde hata mesajını çıkarmak ve `panic!` makrosuna aktarmak için `TitleError` variant'ını
destructure etmen (parçalarına ayırman) gerekti.\
Bu, **hata raporlamanın** (ilkel) bir örneğidir: bir hata türünü bir kullanıcıya, bir servis operatörüne ya da bir geliştiriciye
gösterilebilecek bir temsile dönüştürmek.

Her Rust geliştiricisinin kendi hata raporlama stratejisini bulması pratik değildir: zaman kaybı olurdu
ve projeler arasında birbiriyle iyi uyum sağlamazdı.
Bu yüzden Rust, `std::error::Error` trait'ini sunar.

## `Error` Trait'i {#the-error-trait}

Bir `Result` içindeki `Err` variant'ının türü üzerinde hiçbir kısıtlama yoktur, ama `Error` trait'ini implement eden
bir tür kullanmak iyi bir pratiktir.
`Error`, Rust'ın hata yönetimi hikâyesinin temel taşıdır:

```rust
// `Error` trait'inin biraz basitleştirilmiş tanımı
pub trait Error: Debug + Display {}
```

`:` söz dizimini [`From` trait'inden](../04_traits/09_from.md#supertrait--subtrait) hatırlıyor olabilirsin—**supertrait**'leri belirtmek için kullanılır.
`Error` için iki supertrait vardır: `Debug` ve `Display`. Bir tür `Error`'ı implement etmek istiyorsa
`Debug` ve `Display`'yi de implement etmelidir.

## `Display` Ve `Debug` {#display-and-debug}

`Debug` trait'iyle [önceki bir egzersizde](../04_traits/04_derive.md) zaten karşılaşmıştık—`assert_eq!`'in, assertion başarısız
olduğunda karşılaştırdığı değişkenlerin değerlerini göstermek için kullandığı trait'tir.

"Mekanik" bir bakış açısından `Display` ve `Debug` aynıdır—bir türün string benzeri bir temsile nasıl
dönüştürülmesi gerektiğini kodlarlar:

```rust
// `Debug`
pub trait Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;
}

// `Display`
pub trait Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;
}
```

Fark _amaçlarındadır_: `Display`, "son kullanıcılar" için tasarlanmış bir temsil döndürürken
`Debug`, geliştiricilere ve servis operatörlerine daha uygun, düşük seviyeli bir temsil sunar.\
Bu yüzden `Debug`, `#[derive(Debug)]` attribute'u (öznitelik) kullanılarak otomatik olarak implement edilebilirken `Display`
elle yazılmış bir implementation **gerektirir**.
