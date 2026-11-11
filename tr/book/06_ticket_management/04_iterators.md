# Yineleme {#iteration}

Daha ilk egzersizlerde, Rust'ın `for` döngüleriyle koleksiyonlar üzerinde gezinmene izin verdiğini öğrendin.
O sırada aralıklara bakıyorduk (ör. `0..5`), ama aynısı diziler ve vektörler gibi koleksiyonlar için de geçerlidir.

```rust
// `Vec`'ler için çalışır
let v = vec![1, 2, 3];
for n in v {
    println!("{}", n);
}

// Diziler için de çalışır
let a: [u32; 3] = [1, 2, 3];
for n in a {
    println!("{}", n);
}
```

Bunun perde arkasında nasıl çalıştığını anlamanın zamanı geldi.

## `for`'un Desugar Edilmesi {#for-desugaring}

Rust'ta her `for` döngüsü yazdığında, derleyici onu aşağıdaki koda _desugar eder_ (sözdizimsel şekeri açar):

```rust
let mut iter = IntoIterator::into_iter(v);
loop {
    match iter.next() {
        Some(n) => {
            println!("{}", n);
        }
        None => break,
    }
}
```

`loop`, `for` ve `while`'a ek olarak bir başka döngü yapısıdır.\
Bir `loop` bloğu, ondan açıkça `break` ile çıkmadığın sürece sonsuza kadar çalışır.

## `Iterator` Trait'i {#iterator-trait}

Önceki kod parçasındaki `next` metodu `Iterator` trait'inden gelir.
`Iterator` trait'i Rust'ın standart kütüphanesinde tanımlanmıştır ve bir değerler serisi üretebilen
türler için ortak bir arayüz sağlar:

```rust
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
```

`Item` associated type'ı (ilişkili türü), iterator tarafından üretilen değerlerin türünü belirtir.

`next`, serideki bir sonraki değeri döndürür.\
Döndürülecek bir değer varsa `Some(value)`, yoksa `None` döndürür.

Dikkat: bir iterator'ın `None` döndürdüğünde tükendiğinin hiçbir garantisi yoktur. Bu yalnızca iterator
(daha kısıtlayıcı olan) [`FusedIterator`](https://doc.rust-lang.org/std/iter/trait.FusedIterator.html) trait'ini
implement ediyorsa garanti edilir.

## `IntoIterator` Trait'i {#intoiterator-trait}

Her tür `Iterator`'ı implement etmez, ama birçoğu onu implement eden bir türe dönüştürülebilir.\
İşte burada `IntoIterator` trait'i devreye girer:

```rust
trait IntoIterator {
    type Item;
    type IntoIter: Iterator<Item = Self::Item>;
    fn into_iter(self) -> Self::IntoIter;
}
```

`into_iter` metodu orijinal değeri tüketir ve onun elemanları üzerinde bir iterator döndürür.\
Bir türün `IntoIterator` için yalnızca bir implementation'ı olabilir: `for`'un neye desugar edileceği konusunda hiçbir belirsizlik olamaz.

Bir ayrıntı: `Iterator`'ı implement eden her tür otomatik olarak `IntoIterator`'ı da implement eder.
Bu türler `into_iter`'den basitçe kendilerini döndürürler!

## Sınır Denetimleri {#bounds-checks}

Iterator'lar üzerinde gezinmenin güzel bir yan etkisi vardır: tasarım gereği sınırların dışına çıkamazsın.\
Bu, Rust'ın üretilen makine kodundan sınır denetimlerini kaldırmasına olanak tanır ve yinelemeyi hızlandırır.

Başka bir deyişle,

```rust
let v = vec![1, 2, 3];
for n in v {
    println!("{}", n);
}
```

genellikle şundan daha hızlıdır:

```rust
let v = vec![1, 2, 3];
for i in 0..v.len() {
    println!("{}", v[i]);
}
```

Bu kuralın istisnaları vardır: derleyici bazen elle indekslemede bile sınırların dışına çıkmadığını kanıtlayabilir
ve böylece sınır denetimlerini yine de kaldırabilir. Ama genel olarak, mümkün olduğunda indeksleme yerine yinelemeyi tercih et.
