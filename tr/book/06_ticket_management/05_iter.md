# `.iter()` {#iter}

`IntoIterator`, bir iterator oluşturmak için `self`'i **tüketir**.

Bunun faydaları vardır: iterator'dan **owned** (sahipli) değerler alırsın.
Örneğin: bir `Vec<Ticket>` üzerinde `.into_iter()` çağırırsan, `Ticket` değerleri döndüren bir iterator elde edersin.

Bu aynı zamanda dezavantajıdır: üzerinde `.into_iter()` çağırdıktan sonra orijinal koleksiyonu artık kullanamazsın.
Çoğu zaman bir koleksiyon üzerinde onu tüketmeden gezinmek, bunun yerine değerlere ait **referanslara** bakmak istersin.
`Vec<Ticket>` durumunda, `&Ticket` değerleri üzerinde gezinmek istersin.

Çoğu koleksiyon, koleksiyonun elemanlarına ait referanslar üzerinde bir iterator döndüren `.iter()` adlı bir metot sunar.
Örneğin:

```rust
let numbers: Vec<u32> = vec![1, 2];
// `n` burada `&u32` türündedir
for n in numbers.iter() {
    // [...]
}
```

Bu kalıp, **koleksiyona ait bir referans** için `IntoIterator` implement edilerek basitleştirilebilir.
Yukarıdaki örneğimizde bu `&Vec<Ticket>` olurdu.\
Standart kütüphane bunu yapar; aşağıdaki kodun çalışmasının nedeni de budur:

```rust
let numbers: Vec<u32> = vec![1, 2];
// `n` burada `&u32` türündedir
// `.iter()`'ı açıkça çağırmamız gerekmedi
// `for` döngüsünde `&numbers` kullanmak yeterliydi
for n in &numbers {
    // [...]
}
```

İki seçeneği birden sunmak idiomatiktir:

- Koleksiyona ait bir referans için bir `IntoIterator` implementation'ı.
- Koleksiyonun elemanlarına ait referanslar üzerinde bir iterator döndüren bir `.iter()` metodu.

İlki `for` döngülerinde kullanışlıdır, ikincisi ise daha açıktır ve başka bağlamlarda da kullanılabilir.
