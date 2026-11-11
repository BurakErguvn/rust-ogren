# Lifetime'lar (Yaşam Süreleri) {#lifetimes}

Önceki egzersizi, `for` döngülerinde azami kolaylık sağlamak için `&TicketStore` için bir `IntoIterator`
implementation'ı ekleyerek tamamlamaya çalışalım.

Implementation'ın en "bariz" kısımlarını doldurarak başlayalım:

```rust
impl IntoIterator for &TicketStore {
    type Item = &Ticket;
    type IntoIter = // Buraya ne gelecek?

    fn into_iter(self) -> Self::IntoIter {
        self.tickets.iter()
    }
}
```

`type IntoIter` neye ayarlanmalı?\
Sezgisel olarak, `self.tickets.iter()` tarafından döndürülen tür, yani `Vec::iter()` tarafından döndürülen tür olmalı.\
Standart kütüphane dokümantasyonuna bakarsan, `Vec::iter()`'in bir `std::slice::Iter` döndürdüğünü görürsün.
`Iter`'in tanımı şöyledir:

```rust
pub struct Iter<'a, T> { /* alanlar gösterilmiyor */ }
```

`'a` bir **lifetime parametresidir**.

## Lifetime Parametreleri {#lifetime-parameters}

Lifetime'lar, Rust derleyicisinin bir referansın—mutable (değiştirilebilir) ya da immutable (değiştirilemez)—ne kadar
süre geçerli olduğunu takip etmek için kullandığı **etiketlerdir**.\
Bir referansın lifetime'ı, referans verdiği değerin kapsamıyla sınırlıdır. Rust, dangling pointer'ları (sarkan pointer'ları)
ve use-after-free bug'larını önlemek için, referansların, işaret ettikleri değer drop edildikten sonra kullanılmadığından derleme zamanında her zaman emin olur.

Bu tanıdık gelmeli: ownership (sahiplik) ve borrowing (ödünç alma) konularını ele alırken bu kavramları iş başında zaten görmüştük.
Lifetime'lar, belirli bir referansın ne kadar süre geçerli olduğuna **ad vermenin** bir yoludur, o kadar.

Birden fazla referansın olduğu ve bunların **birbirleriyle nasıl ilişkili olduğunu** netleştirmen gereken durumlarda adlandırma önem kazanır.
`Vec::iter()`'in imzasına bakalım:

```rust
impl <T> Vec<T> {
    // Biraz basitleştirilmiş hali
    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        // [...]
    }
}
```

`Vec::iter()`, `'a` adlı bir lifetime parametresi üzerinde generic'tir.\
`'a`, `Vec`'in lifetime'ını ve `iter()` tarafından döndürülen `Iter`'in lifetime'ını **birbirine bağlamak** için kullanılır.
Basitçe söylemek gerekirse: `iter()` tarafından döndürülen `Iter`, oluşturulduğu `Vec` referansından (`&self`) daha uzun yaşayamaz.

Bu önemlidir, çünkü daha önce de konuştuğumuz gibi `Vec::iter`, `Vec`'in elemanlarına ait **referanslar** üzerinde bir iterator döndürür.
`Vec` drop edilirse, iterator'ın döndürdüğü referanslar geçersiz olur. Rust bunun olmamasını sağlamalıdır
ve lifetime'lar, bu kuralı zorla uygulatmak için kullandığı araçtır.

## Lifetime Elision {#lifetime-elision}

Rust'ın, birçok durumda açık lifetime belirtimlerini yazmamana izin veren ve **lifetime elision kuralları**
(lifetime'ın yazılmadan bırakılmasına dair kurallar) adı verilen birtakım kuralları vardır.
Örneğin, `std`'nin kaynak kodunda `Vec::iter`'in tanımı şöyle görünür:

```rust
impl <T> Vec<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        // [...]
    }
}
```

`Vec::iter()`'in imzasında hiçbir açık lifetime parametresi yoktur.
Elision kuralları, `iter()` tarafından döndürülen `Iter`'in lifetime'ının `&self` referansının lifetime'ına bağlı olduğunu ima eder.
`'_` işaretini, `&self` referansının lifetime'ı için bir **yer tutucu** olarak düşünebilirsin.

Lifetime elision hakkındaki resmi dokümantasyonun linki için [Referanslar](#references) kısmına bak.\
Çoğu durumda, açık lifetime belirtimleri eklemen gerektiğinde bunu sana derleyicinin söyleyeceğine güvenebilirsin.

## Referanslar {#references}

- [std::vec::Vec::iter](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.iter)
- [std::slice::Iter](https://doc.rust-lang.org/std/slice/struct.Iter.html)
- [Lifetime elision kuralları](https://doc.rust-lang.org/reference/lifetime-elision.html)
