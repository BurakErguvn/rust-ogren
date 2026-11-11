# `match` {#match}

Merak ediyor olabilirsin—bir enum ile aslında ne **yapabilirsin**?\
En yaygın işlem, onun üzerinde **match** yapmaktır.

```rust
enum Status {
    ToDo,
    InProgress,
    Done
}

impl Status {
    fn is_done(&self) -> bool {
        match self {
            Status::Done => true,
            // `|` operatörü birden çok pattern'i eşleştirmeni sağlar.
            // "ya `Status::ToDo` ya da `Status::InProgress`" diye okunur.
            Status::InProgress | Status::ToDo => false
        }
    }
}
```

`match` deyimi, bir Rust değerini art arda gelen **pattern**'lerle (desenler) karşılaştırmanı sağlar.\
Onu tür düzeyinde bir `if` olarak düşünebilirsin. `status` bir `Done` variant'ıysa ilk bloğu çalıştır;
bir `InProgress` ya da `ToDo` variant'ıysa ikinci bloğu çalıştır.

## Eksiksizlik {#exhaustiveness}

Burada önemli bir ayrıntı var: `match` **eksiksizdir**. Tüm enum variant'larını ele almalısın.\
Bir variant'ı ele almayı unutursan Rust seni **derleme zamanında** bir hatayla durdurur.

Örneğin `ToDo` variant'ını ele almayı unutursak:

```rust
match self {
    Status::Done => true,
    Status::InProgress => false,
}
```

derleyici şikâyet eder:

```text
error[E0004]: non-exhaustive patterns: `ToDo` not covered
 --> src/main.rs:5:9
  |
5 |     match status {
  |     ^^^^^^^^^^^^ pattern `ToDo` not covered
```

Bu çok önemli!\
Kod tabanları zamanla evrilir—ileride yeni bir durum ekleyebilirsin, örneğin `Blocked`. Rust derleyicisi,
yeni variant için mantığı eksik olan her bir `match` deyimi için bir hata verir.
Rust geliştiricilerinin "derleyici güdümlü refactoring"i sıklıkla övmesinin nedeni budur—derleyici sana
sırada ne yapman gerektiğini söyler, senin yapman gereken tek şey onun bildirdiklerini düzeltmektir.

## Catch-all {#catch-all}

Bir ya da daha fazla variant ile ilgilenmiyorsan, catch-all olarak `_` pattern'ini kullanabilirsin:

```rust
match status {
    Status::Done => true,
    _ => false
}
```

`_` pattern'i, önceki pattern'lerin eşleşmediği her şeyle eşleşir.

<div class="warning">
  `_` kullanmanın kolaylığının bir bedeli var: enum değiştiğinde derleyici artık sana yardım edemez. Yukarıdaki örnekte, `Status`'a bir `Blocked` variant'ı eklendiğinde bu `match` derlenmeye devam eder, çünkü yeni `Blocked` variant'ı, istediğin davranış bu olsun ya da olmasın, `_` arm'ı (kolu) tarafından sessizce ele alınır.

Doğruluk önemli olduğunda, birkaç arm aynı mantığı paylaşsa bile her variant'ı tek tek yaz
(ör. `Status::ToDo | Status::InProgress => false`). Böylece her yeni variant, bilinçli bir karar vermeye zorlar.

</div>
