# `Deref` Trait'i {#deref-trait}

Önceki egzersizde pek bir şey yapman gerekmedi, değil mi?

Şunu

```rust
impl Ticket {
    pub fn title(&self) -> &String {
        &self.title
    }
}
```

şuna

```rust
impl Ticket {
    pub fn title(&self) -> &str {
        &self.title
    }
}
```

dönüştürmek, kodun derlenmesi ve testlerin geçmesi için yapman gereken tek şeydi.
Yine de kafanda bazı alarm zilleri çalıyor olmalı.

## Çalışmaması Gerekiyor, Ama Çalışıyor {#it-shouldnt-work-but-it-does}

Gerçekleri gözden geçirelim:

- `self.title` bir `String`'dir
- Dolayısıyla `&self.title` bir `&String`'dir
- (Değiştirilmiş) `title` metodunun çıktısı ise `&str`

Bir derleyici hatası beklerdin, değil mi? `Expected &String, found &str` ya da benzeri bir şey.
Bunun yerine, kod düpedüz çalışıyor. **Neden**?

## İmdada `Deref` Yetişiyor {#deref-to-the-rescue}

`Deref` trait'i, [**deref coercion**](https://doc.rust-lang.org/std/ops/trait.Deref.html#deref-coercion) olarak bilinen dil özelliğinin arkasındaki mekanizmadır.\
Trait, standart kütüphanede, `std::ops` modülünde tanımlanmıştır:

```rust
// Tanımı şimdilik biraz basitleştirdim.
// Tam tanımı daha sonra göreceğiz.
pub trait Deref {
    type Target;
    
    fn deref(&self) -> &Self::Target;
}
```

`type Target` bir **associated type**'tır (ilişkili tür).\
Trait implement edilirken belirtilmesi gereken somut bir tür için bir yer tutucudur.

## Deref Coercion {#deref-coercion}

Bir `T` türü için `Deref<Target = U>` implement ederek derleyiciye `&T` ve `&U`'nun
bir ölçüde birbirinin yerine kullanılabileceğini söylersin.\
Daha açık söylemek gerekirse, şu davranışı elde edersin:

- `T`'ye referanslar örtük olarak `U`'ya referanslara dönüştürülür (yani `&T`, `&U` olur)
- `U` üzerinde tanımlanmış ve girdi olarak `&self` alan tüm metotları `&T` üzerinde çağırabilirsin.

Dereference operatörü `*` ile ilgili bir şey daha var, ama ona henüz ihtiyacımız yok (merak ediyorsan `std`'nin
dokümantasyonuna bak).

## `String`, `Deref`'i Implement Eder {#string-implements-deref}

`String`, `Deref`'i `Target = str` ile implement eder:

```rust
impl Deref for String {
    type Target = str;
    
    fn deref(&self) -> &str {
        // [...]
    }
}
```

Bu implementation ve deref coercion sayesinde, bir `&String` gerektiğinde otomatik olarak bir `&str`'ye dönüştürülür.

## Deref Coercion'ı Kötüye Kullanma {#dont-abuse-deref-coercion}

Deref coercion güçlü bir özelliktir, ama kafa karışıklığına yol açabilir.\
Türleri otomatik olarak dönüştürmek, kodu okumayı ve anlamayı zorlaştırabilir. Aynı isimli bir metot
hem `T` hem de `U` üzerinde tanımlıysa hangisi çağrılır?

Deref coercion'ın "en güvenli" kullanım alanlarını, yani smart pointer'ları (akıllı pointer'ları), kursun ilerleyen kısımlarında inceleyeceğiz.
