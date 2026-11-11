# Combinator'lar {#combinators}

Iterator'lar `for` döngülerinden çok daha fazlasını yapabilir!\
`Iterator` trait'inin dokümantasyonuna bakarsan, iterator'ları çeşitli şekillerde dönüştürmek, filtrelemek ve
birleştirmek için yararlanabileceğin **devasa** bir metot koleksiyonu bulursun.

En yaygın olanlardan bahsedelim:

- `map`, iterator'ın her elemanına bir fonksiyon uygular.
- `filter`, yalnızca bir predicate'i (yüklemi) sağlayan elemanları tutar.
- `filter_map`, `filter` ve `map`'i tek adımda birleştirir.
- `cloned`, referanslardan oluşan bir iterator'ı, her elemanı klonlayarak değerlerden oluşan bir iterator'a dönüştürür.
- `enumerate`, `(index, value)` çiftleri üreten yeni bir iterator döndürür.
- `skip`, iterator'ın ilk `n` elemanını atlar.
- `take`, iterator'ı `n` elemandan sonra durdurur.
- `chain`, iki iterator'ı tek bir iterator'da birleştirir.

Bu metotlara **combinator** denir.\
Genellikle karmaşık dönüşümleri kısa ve okunabilir bir şekilde oluşturmak için birbirlerine **zincirlenirler**:

```rust
let numbers = vec![1, 2, 3, 4, 5];
// Çift sayıların karelerinin toplamı
let outcome: u32 = numbers.iter()
    .filter(|&n| n % 2 == 0)
    .map(|&n| n * n)
    .sum();
```

## Closure'lar {#closures}

Yukarıdaki `filter` ve `map` metotlarında neler oluyor?\
Argüman olarak **closure** alıyorlar.

Closure'lar **anonim fonksiyonlardır**, yani alışık olduğumuz `fn` söz dizimi kullanılarak tanımlanmayan fonksiyonlardır.\
`|args| body` söz dizimiyle tanımlanırlar; burada `args` argümanlar, `body` ise fonksiyon gövdesidir.
`body` bir kod bloğu ya da tek bir ifade olabilir.
Örneğin:

```rust
// Argümanına 1 ekleyen anonim bir fonksiyon
let add_one = |x| x + 1;
// Bir blokla da yazılabilirdi:
let add_one = |x| { x + 1 };
```

Closure'lar birden fazla argüman alabilir:

```rust
let add = |x, y| x + y;
let sum = add(1, 2);
```

Ortamlarındaki değişkenleri de yakalayabilirler:

```rust
let x = 42;
let add_x = |y| x + y;
let sum = add_x(1);
```

Gerekirse argümanların türlerini ve/veya dönüş türünü belirtebilirsin:

```rust
// Yalnızca girdi türü
let add_one = |x: i32| x + 1;
// Ya da `fn` söz dizimini kullanarak hem girdi hem de çıktı türleri
let add_one: fn(i32) -> i32 = |x| x + 1;
```

## `collect` {#collect}

Combinator'ları kullanarak bir iterator'ı dönüştürmeyi bitirdiğinde ne olur?\
Ya dönüştürülmüş değerler üzerinde bir `for` döngüsüyle gezinirsin ya da onları bir koleksiyonda toplarsın.

İkincisi `collect` metoduyla yapılır.\
`collect`, iterator'ı tüketir ve elemanlarını seçtiğin bir koleksiyonda toplar.

Örneğin, çift sayıların karelerini bir `Vec`'te toplayabilirsin:

```rust
let numbers = vec![1, 2, 3, 4, 5];
let squares_of_evens: Vec<u32> = numbers.iter()
    .filter(|&n| n % 2 == 0)
    .map(|&n| n * n)
    .collect();
```

`collect`, **dönüş türü** üzerinde generic'tir.\
Bu yüzden derleyicinin doğru türü çıkarsamasına yardımcı olmak için genellikle bir tür ipucu vermen gerekir.
Yukarıdaki örnekte `squares_of_evens`'in türünü `Vec<u32>` olarak belirttik.
Alternatif olarak, türü belirtmek için **turbofish söz dizimini** kullanabilirsin:

```rust
let squares_of_evens = numbers.iter()
    .filter(|&n| n % 2 == 0)
    .map(|&n| n * n)
    // Turbofish söz dizimi: `<method_name>::<type>()`
    // `::<>` bir balığa benzediği için turbofish denir
    .collect::<Vec<u32>>();
```

## İleri Okuma {#further-reading}

- [`Iterator`'ın dokümantasyonu](https://doc.rust-lang.org/std/iter/trait.Iterator.html) sana `std`'deki
  iterator'lar için mevcut metotlara genel bir bakış sunar.
- [`itertools` crate'i](https://docs.rs/itertools/) iterator'lar için **daha da fazla** combinator tanımlar.
