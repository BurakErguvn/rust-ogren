# Duruma Göre Davranış {#case-by-case-behavior}

`overflow-checks` kaba bir araçtır: tüm programı etkileyen global bir ayardır.\
Çoğu zaman tam sayı overflow'larını (taşmalarını) bağlama göre farklı şekilde ele almak istersin: bazen
wrap around (başa sarma) doğru seçimdir, bazen de panic'lemek tercih edilir.

## `wrapping_` Metotları {#wrapping_-methods}

`wrapping_` metotlarını[^method] kullanarak wrapping aritmetiğini işlem bazında tercih edebilirsin.\
Örneğin iki tam sayıyı wrapping ile toplamak için `wrapping_add` kullanabilirsin:

```rust
let x = 255u8;
let y = 1u8;
let sum = x.wrapping_add(y);
assert_eq!(sum, 0);
```

## `saturating_` Metotları {#saturating_-methods}

Alternatif olarak, `saturating_` metotlarını kullanarak **saturating arithmetic**'i (doygun aritmetik) tercih edebilirsin.\
Saturating arithmetic, wrap around yapmak yerine tam sayı türünün maksimum ya da minimum değerini döndürür.
Örneğin:

```rust
let x = 255u8;
let y = 1u8;
let sum = x.saturating_add(y);
assert_eq!(sum, 255);
```

`255 + 1` `256` eder ve bu `u8::MAX`'tan büyüktür; bu yüzden sonuç `u8::MAX` (255) olur.\
Underflow'larda (alttan taşmalarda) bunun tersi olur: `0 - 1` `-1` eder ve bu `u8::MIN`'den küçüktür; bu yüzden sonuç
`u8::MIN` (0) olur.

Saturating arithmetic'i `overflow-checks` profil ayarıyla elde edemezsin—aritmetik işlemi yaparken onu açıkça tercih
etmen gerekir.

[^method]: Metotları, belirli bir türe "iliştirilmiş" fonksiyonlar olarak düşünebilirsin.
Metotları (ve nasıl tanımlanacaklarını) bir sonraki bölümde ele alacağız.
