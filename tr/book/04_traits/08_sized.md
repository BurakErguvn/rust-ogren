# `Sized` {#sized}

Deref coercion'ı inceledikten sonra bile `&str`, göründüğünden
fazlasını barındırıyor.\
[Bellek düzenleri üzerine önceki tartışmamızdan](../03_ticket_v1/10_references_in_memory.md) yola çıkarak,
`&str`'nin stack'te tek bir `usize` olarak, yani bir pointer (işaretçi) olarak temsil edilmesini
beklemek makul olurdu. Ama durum öyle değil. `&str`, pointer'ın yanında bazı **meta veriler**
saklar: gösterdiği slice'ın (dilimin) uzunluğu.
[Önceki bir kısımdaki](06_str_slice.md) örneğe geri dönersek:

```rust
let mut s = String::with_capacity(5);
s.push_str("Hello");
// `String`'den bir string slice referansı oluştur,
// ilk byte'ı atlayarak.
let slice: &str = &s[1..];
```

Bellekte şunu elde ederiz:

```text
                    s                              slice
      +---------+--------+----------+      +---------+--------+
Stack | pointer | length | capacity |      | pointer | length |
      |    |    |   5    |    5     |      |    |    |   4    |
      +----|----+--------+----------+      +----|----+--------+
           |        s                           |  
           |                                    |
           v                                    | 
         +---+---+---+---+---+                  |
Heap:    | H | e | l | l | o |                  |
         +---+---+---+---+---+                  |
               ^                                |
               |                                |
               +--------------------------------+
```

Neler oluyor?

## Dynamically Sized Type'lar (Dinamik Boyutlu Türler) {#dynamically-sized-types}

`str` bir **dynamically sized type**'tır (DST).\
DST, boyutu derleme zamanında bilinmeyen bir türdür. Bir DST'ye referansın olduğunda
(örneğin `&str`), bu referans gösterdiği veri hakkında ek bilgi
içermek zorundadır. Bu bir **fat pointer**'dır.\
`&str` söz konusu olduğunda, gösterdiği slice'ın uzunluğunu saklar.
Kursun geri kalanında DST'lerin başka örneklerini de göreceğiz.

## `Sized` Trait'i {#the-sized-trait}

Rust'ın `std` kütüphanesi `Sized` adlı bir trait tanımlar.

```rust
pub trait Sized {
    // Bu boş bir trait; implement edilecek metot yok.
}
```

Bir türün boyutu derleme zamanında biliniyorsa o tür `Sized`'dır. Başka bir deyişle, bir DST değildir.

### Marker Trait'ler {#marker-traits}

`Sized`, karşılaştığın ilk **marker trait** örneğidir.\
Marker trait, implement edilmesi gereken hiçbir metot şart koşmayan bir trait'tir. Herhangi bir davranış tanımlamaz.
Yalnızca bir türü belirli özelliklere sahip olarak **işaretlemeye** yarar.
Bu işaret daha sonra derleyici tarafından belirli davranışları ya da optimizasyonları etkinleştirmek için kullanılır.

### Auto Trait'ler {#auto-traits}

Özellikle, `Sized` aynı zamanda bir **auto trait**'tir.\
Onu açıkça implement etmen gerekmez; derleyici, türün tanımına göre onu senin için
otomatik olarak implement eder.

### Örnekler {#examples}

Şimdiye kadar gördüğümüz tüm türler `Sized`'dır: `u32`, `String`, `bool` vb.

Az önce gördüğümüz gibi `str`, `Sized` değildir.\
Ama `&str` `Sized`'dır! Boyutunu derleme zamanında biliyoruz: biri pointer, biri de uzunluk için
olmak üzere iki `usize`.
