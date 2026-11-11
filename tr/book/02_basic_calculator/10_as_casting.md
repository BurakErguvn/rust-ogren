# Dönüşümler, 1. Kısım {#conversions-pt-1}

Rust'ın tam sayılar için örtük tür dönüşümleri yapmayacağını
defalarca tekrarladık.\
Peki o zaman _açık_ dönüşümleri nasıl yaparsın?

## `as` {#as}

Tam sayı türleri arasında dönüşüm yapmak için `as` operatörünü kullanabilirsin.\
`as` dönüşümleri **başarısız olamaz**.

Örneğin:

```rust
let a: u32 = 10;

// `a`'yı `u64` türüne cast et
let b = a as u64;

// Derleyici tarafından doğru şekilde
// çıkarsanabiliyorsa, hedef tür olarak
// `_` kullanabilirsin. Örneğin:
let c: u64 = a as _;
```

Bu dönüşümün semantiği beklediğin gibidir: tüm `u32` değerleri geçerli `u64`
değerleridir.

### Kırpma {#truncation}

Ters yöne gidersek işler daha ilginç bir hal alır:

```rust
// `u8`'e sığamayacak
// kadar büyük bir sayı
let a: u16 = 255 + 1;
let b = a as u8;
```

Bu program sorunsuz çalışır, çünkü `as` dönüşümleri başarısız olamaz.
Peki `b`'nin değeri nedir?
Daha büyük bir tam sayı türünden daha küçüğüne geçerken Rust derleyicisi
bir **kırpma** yapar.

Ne olduğunu anlamak için, `256u16`'nın bellekte bir bit serisi olarak nasıl
temsil edildiğine bakarak başlayalım:

```text
 0 0 0 0 0 0 0 1 0 0 0 0 0 0 0 0
|               |               |
+---------------+---------------+
  First 8 bits    Last 8 bits
```

`u8`'e dönüştürürken Rust derleyicisi, `u16`'nın bellek gösteriminin son 8 bitini
korur:

```text
 0 0 0 0 0 0 0 0 
|               |
+---------------+
  Last 8 bits
```

Bu yüzden `256 as u8`, `0`'a eşittir. Bu da... çoğu senaryoda pek ideal değil.\
Hatta Rust derleyicisi, kırpmayla sonuçlanacak bir literal değeri cast etmeye (tür dönüştürmeye) çalıştığını
görürse seni aktif olarak durdurmaya çalışır:

```text
error: literal out of range for `i8`
  |
4 |     let a = 255 as i8;
  |             ^^^
  |
  = note: the literal `255` does not fit into the type `i8` 
          whose range is `-128..=127`
  = help: consider using the type `u8` instead
  = note: `#[deny(overflowing_literals)]` on by default
```

### Öneri {#recommendation}

Genel bir kural olarak, `as` ile cast etme konusunda oldukça dikkatli ol.\
Onu _yalnızca_ daha küçük bir türden daha büyük bir türe geçmek için kullan.
Daha büyük bir tam sayı türünden daha küçüğüne dönüştürmek için, kursun ilerleyen kısımlarında inceleyeceğimiz
[_başarısız olabilen_ dönüşüm mekanizmasına](../05_ticket_v2/13_try_from.md) güven.

### Sınırlamalar {#limitations}

Şaşırtıcı davranış, `as` ile cast etmenin tek dezavantajı değildir.
Ayrıca oldukça sınırlıdır: `as` ile cast etmeye yalnızca
ilkel türler ve birkaç başka özel durum için güvenebilirsin.\
Bileşik türlerle çalışırken, daha sonra inceleyeceğimiz
farklı dönüşüm mekanizmalarına ([başarısız olabilen](../05_ticket_v2/13_try_from.md)
ve [başarısız olamayan](../04_traits/09_from.md)) güvenmen gerekecek.

## İleri Okuma {#further-reading}

- Her kaynak/hedef kombinasyonu için `as` ile cast etmenin kesin davranışını ve izin verilen dönüşümlerin
  tam listesini öğrenmek için [Rust'ın resmi referansına](https://doc.rust-lang.org/reference/expressions/operator-expr.html#numeric-cast)
  göz at.
