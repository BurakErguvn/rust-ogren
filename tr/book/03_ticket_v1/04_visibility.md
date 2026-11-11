# Görünürlük {#visibility}

Kodunu birden fazla modüle bölmeye başladığında **görünürlük** hakkında düşünmeye başlaman gerekir.
Görünürlük, kodunun (ya da başkalarının kodunun) hangi bölgelerinin belirli bir varlığa erişebileceğini belirler;
bu varlık bir struct, bir fonksiyon, bir alan vb. olabilir.

## Varsayılan Olarak Private {#private-by-default}

Rust'ta varsayılan olarak her şey **private**'tır.\
Private bir varlığa yalnızca şu şekillerde erişilebilir:

1. tanımlandığı modülün içinden, ya da
2. alt modüllerinden biri tarafından

Bunu önceki egzersizlerde yoğun bir şekilde kullandık:

- `create_todo_ticket` çalıştı (bir `use` deyimi ekledikten sonra), çünkü `helpers`, `Ticket`'ın tanımlandığı
  crate kökünün bir alt modülüdür. Bu nedenle `create_todo_ticket`, `Ticket` private olmasına rağmen
  `Ticket`'a sorunsuzca erişebilir.
- Tüm birim testlerimiz, test ettikleri kodun bir alt modülünde tanımlanmıştır; bu yüzden her şeye kısıtlama olmadan
  erişebilirler.

## Görünürlük Belirleyicileri {#visibility-modifiers}

Bir varlığın varsayılan görünürlüğünü bir **görünürlük belirleyicisi** kullanarak değiştirebilirsin.\
Yaygın görünürlük belirleyicilerinden bazıları şunlardır:

- `pub`: varlığı **public** yapar, yani tanımlandığı modülün dışından, hatta muhtemelen başka
  crate'lerden erişilebilir kılar.
- `pub(crate)`: varlığı aynı **crate** içinde public yapar, ama crate'in dışında değil.
- `pub(super)`: varlığı üst modül içinde public yapar.
- `pub(in path::to::module)`: varlığı belirtilen modül içinde public yapar.

Bu belirleyicileri modüllerde, struct'larda, fonksiyonlarda, alanlarda vb. kullanabilirsin.
Örneğin:

```rust
pub struct Configuration {
    pub(crate) version: u32,
    active: bool,
}
```

`Configuration` public'tir, ama `version` alanına yalnızca aynı crate içinden erişebilirsin.
`active` alanı ise private'tır ve yalnızca aynı modülün ya da alt modüllerinden birinin içinden erişilebilir.
