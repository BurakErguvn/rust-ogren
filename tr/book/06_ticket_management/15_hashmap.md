# `HashMap` {#hashmap}

`Index`/`IndexMut` implementation'ımız ideal değil: bir ticket'ı id'sine göre almak için tüm
`Vec` üzerinde gezinmemiz gerekiyor; algoritmik karmaşıklık `O(n)`'dir; burada
`n`, store'daki ticket sayısıdır.

Ticket'ları saklamak için farklı bir veri yapısı kullanarak daha iyisini yapabiliriz: bir `HashMap<K, V>`.

```rust
use std::collections::HashMap;

// Tür çıkarımı, açık bir tür imzasını yazmamamıza izin verir (bu
// örnekte imza `HashMap<String, String>` olurdu).
let mut book_reviews = HashMap::new();

book_reviews.insert(
    "Adventures of Huckleberry Finn".to_string(),
    "My favorite book.".to_string(),
);
```

`HashMap` anahtar-değer çiftleriyle çalışır. Her ikisi üzerinde de generic'tir: `K` anahtar türü için
generic parametre, `V` ise değer türü için olanıdır.

Ekleme, okuma ve silme işlemlerinin beklenen maliyeti **sabittir**, `O(1)`.
Bu bizim kullanım senaryomuz için mükemmel görünüyor, değil mi?

## Anahtar Gereksinimleri {#key-requirements}

`HashMap`'in struct tanımında hiç trait bound yoktur, ama metotlarında bazılarını
bulursun. Örneğin `insert`'e bakalım:

```rust
// Biraz basitleştirilmiş hali
impl<K, V> HashMap<K, V>
where
    K: Eq + Hash,
{
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        // [...]
    }
}
```

Anahtar türü `Eq` ve `Hash` trait'lerini implement etmelidir.\
Bu ikisine daha yakından bakalım.

## `Hash` {#hash}

Bir hash fonksiyonu (ya da hasher), potansiyel olarak sonsuz bir değerler kümesini (ör.
tüm olası string'ler) sınırlı bir aralığa (ör. bir `u64` değeri) eşler.\
Etrafta, her biri farklı özelliklere (hız, çakışma riski, tersine çevrilebilirlik vb.) sahip
pek çok farklı hash fonksiyonu vardır.

Adından da anlaşılacağı gibi bir `HashMap`, perde arkasında bir hash fonksiyonu kullanır.
Anahtarını hash'ler ve ardından ilişkili değeri saklamak/almak için o hash'i kullanır.
Bu strateji anahtar türünün hash'lenebilir olmasını gerektirir; `K` üzerindeki `Hash` trait bound'u da bu yüzden vardır.

`Hash` trait'ini `std::hash` modülünde bulabilirsin:

```rust
pub trait Hash {
    // Zorunlu metot
    fn hash<H>(&self, state: &mut H)
       where H: Hasher;
}
```

`Hash`'i elle implement etmen nadiren gerekir. Çoğu zaman onu derive edersin:

```rust
#[derive(Hash)]
struct Person {
    id: u32,
    name: String,
}
```

## `Eq` {#eq}

`HashMap`, anahtarları eşitlik açısından karşılaştırabilmelidir. Bu, özellikle hash çakışmalarıyla
uğraşırken—yani iki farklı anahtar aynı değere hash'lendiğinde—önemlidir.

Şunu merak edebilirsin: `PartialEq` trait'i tam olarak bunun için değil mi? Neredeyse!\
`PartialEq`, `HashMap` için yeterli değildir, çünkü yansımalılığı, yani `a == a`'nın her zaman `true` olmasını garanti etmez.\
Örneğin kayan noktalı sayılar (`f32` ve `f64`) `PartialEq` trait'ini implement eder,
ama yansımalılık özelliğini sağlamazlar: `f32::NAN == f32::NAN` ifadesi `false`'tur.\
Yansımalılık, `HashMap`'in doğru çalışması için kritiktir: o olmadan, bir değeri map'e eklerken kullandığın
anahtarın aynısıyla o değeri map'ten geri alamazdın.

`Eq` trait'i, `PartialEq` trait'ini yansımalılık özelliğiyle genişletir:

```rust
pub trait Eq: PartialEq {
    // Ek metot yok
}
```

Bir marker trait'tir: yeni bir metot eklemez; yalnızca derleyiciye, `PartialEq` içinde implement edilen
eşitlik mantığının yansımalı olduğunu söylemenin bir yoludur.

`PartialEq` trait'ini derive ettiğinde `Eq` trait'ini de otomatik olarak derive edebilirsin:

```rust
#[derive(PartialEq, Eq)]
struct Person {
    id: u32,
    name: String,
}
```

## `Eq` ve `Hash` Birbirine Bağlıdır {#eq-and-hash-are-linked}

`Eq` ile `Hash` arasında örtük bir sözleşme vardır: iki anahtar eşitse, hash'leri de eşit olmalıdır.
Bu, `HashMap`'in doğru çalışması için kritiktir. Bu sözleşmeyi bozarsan, `HashMap` kullanırken
anlamsız sonuçlar alırsın.
