# Sıralama {#ordering}

Bir `Vec`'ten bir `HashMap`'e geçerek ticket yönetim sistemimizin performansını iyileştirdik
ve bu süreçte kodumuzu da sadeleştirdik.\
Yine de her şey güllük gülistanlık değil. `Vec` tabanlı bir store üzerinde gezinirken, ticket'ların
eklendikleri sırayla döndürüleceğinden emin olabiliyorduk.\
`HashMap` için durum böyle değil: ticket'lar üzerinde gezinebilirsin, ama sıra rastgeledir.

`HashMap`'ten `BTreeMap`'e geçerek tutarlı bir sıralamayı yeniden elde edebiliriz.

## `BTreeMap` {#btreemap}

Bir `BTreeMap`, girdilerin anahtarlarına göre sıralı olmasını garanti eder.\
Bu, girdiler üzerinde belirli bir sırayla gezinmen gerektiğinde ya da aralık sorguları
(ör. "id'si 10 ile 20 arasında olan tüm ticket'ları ver") yapman gerektiğinde kullanışlıdır.

Tıpkı `HashMap`'te olduğu gibi, `BTreeMap`'in tanımında da trait bound bulamazsın.
Ama metotlarında trait bound'lar bulursun. `insert`'e bakalım:

```rust
// `K` ve `V` sırasıyla anahtar ve değer türlerini temsil eder,
// tıpkı `HashMap`'te olduğu gibi.
impl<K, V> BTreeMap<K, V> {
    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord,
    {
        // implementation
    }
}
```

`Hash` artık gerekli değil. Bunun yerine anahtar türü `Ord` trait'ini implement etmelidir.

## `Ord` {#ord}

`Ord` trait'i değerleri karşılaştırmak için kullanılır.\
`PartialEq` eşitlik açısından karşılaştırmak için kullanılırken, `Ord` sıralama açısından karşılaştırmak için kullanılır.

`std::cmp` içinde tanımlanmıştır:

```rust
pub trait Ord: Eq + PartialOrd {
    fn cmp(&self, other: &Self) -> Ordering;
}
```

`cmp` metodu, `Less`, `Equal` ya da `Greater` değerlerinden biri olabilen
bir `Ordering` enum'u döndürür.\
`Ord`, iki başka trait'in de implement edilmiş olmasını gerektirir: `Eq` ve `PartialOrd`.

## `PartialOrd` {#partialord}

Tıpkı `PartialEq` trait'inin `Eq` trait'inin daha zayıf bir versiyonu olması gibi, `PartialOrd` da `Ord`'un daha zayıf bir versiyonudur.
Nedenini tanımına bakarak görebilirsin:

```rust
pub trait PartialOrd: PartialEq {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>;
}
```

`PartialOrd::partial_cmp` bir `Option` döndürür—iki değerin karşılaştırılabileceği
garanti edilmez.\
Örneğin `f32`, `NaN` değerleri karşılaştırılabilir olmadığı için `Ord` trait'ini implement etmez;
`f32`'nin `Eq` trait'ini implement etmemesinin nedeni de aynıdır.

## `Ord` ve `PartialOrd`'u Implement Etmek {#implementing-ord-and-partialord}

`Ord` ve `PartialOrd`, türlerin için derive edilebilir:

```rust
// `Ord` onları gerektirdiği için
// `Eq` ve `PartialEq` trait'lerini de eklemen gerekir.
#[derive(Eq, PartialEq, Ord, PartialOrd)]
struct TicketId(u64);
```

Bunları elle implement etmeyi seçersen (ya da buna mecbur kalırsan) dikkatli ol:

- `Ord` ve `PartialOrd`, `Eq` ve `PartialEq` ile tutarlı olmalıdır.
- `Ord` ve `PartialOrd` birbirleriyle tutarlı olmalıdır.
