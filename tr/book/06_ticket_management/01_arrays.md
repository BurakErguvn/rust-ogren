# Diziler {#arrays}

"Ticket yönetimi"nden söz etmeye başlar başlamaz, _birden fazla_ ticket'ı saklamanın bir yolunu düşünmemiz gerekir.
Bu da koleksiyonları düşünmemiz gerektiği anlamına gelir. Özellikle de homojen koleksiyonları:
aynı türden birden fazla instance'ı (örneği) saklamak istiyoruz.

Rust bu konuda neler sunuyor?

## Diziler {#arrays-1}

İlk girişim bir **dizi** kullanmak olabilir.\
Rust'ta diziler, aynı türden elemanların sabit boyutlu koleksiyonlarıdır.

Bir diziyi şöyle tanımlayabilirsin:

```rust
// Dizi türü söz dizimi: [ <tür> ; <eleman sayısı> ]
let numbers: [u32; 3] = [1, 2, 3];
```

Bu, `1`, `2` ve `3` değerleriyle ilk değeri atanmış 3 tam sayılık bir dizi oluşturur.\
Dizinin türü `[u32; 3]`'tür; bu, "uzunluğu 3 olan, `u32`'lerden oluşan bir dizi" diye okunur.

Her elemanın aynı değerle başlaması gerektiğinde, bu değeri tekrar tekrar yazmana gerek yoktur. Rust'ın bunun için özel bir kısa söz dizimi vardır:

```rust
// Tekrar söz dizimi: [ <değer> ; <eleman sayısı> ]
let zeros: [u32; 3] = [0; 3];
```

`[0; 3]`, `[0, 0, 0]` ile eşdeğerdir.

### Elemanlara Erişmek {#accessing-elements}

Bir dizinin elemanlarına köşeli parantezler kullanarak erişebilirsin:

```rust
let first = numbers[0];
let second = numbers[1];
let third = numbers[2];
```

İndeks `usize` türünde olmalıdır.\
Diziler, Rust'taki her şey gibi **sıfırdan indekslenir**. Bunu daha önce string slice'larda ve
tuple'larda/tuple benzeri variant'larda alan indekslemede görmüştün.

### Sınırların Dışında Erişim {#out-of-bounds-access}

Sınırların dışında kalan bir elemana erişmeye çalışırsan Rust panic'ler:

```rust
let numbers: [u32; 3] = [1, 2, 3];
let fourth = numbers[3]; // Bu panic'leyecek
```

Bu, çalışma zamanında **sınır denetimi** ile zorunlu kılınır. Küçük bir performans maliyeti vardır, ama Rust
buffer overflow'ları bu şekilde önler.\
Bazı senaryolarda Rust derleyicisi sınır denetimlerini optimize ederek kaldırabilir, özellikle de iterator'lar söz
konusuysa—bundan ileride daha ayrıntılı bahsedeceğiz.

Panic'lemek istemiyorsan, `Option<&T>` döndüren `get` metodunu kullanabilirsin:

```rust
let numbers: [u32; 3] = [1, 2, 3];
assert_eq!(numbers.get(0), Some(&1));
// Sınırların dışındaki bir indekse erişmeye çalışırsan
// panic yerine bir `None` alırsın.
assert_eq!(numbers.get(3), None);
```

### Performans {#performance}

Bir dizinin boyutu derleme zamanında bilindiği için, derleyici dizi için stack'te bellek ayırabilir.
Aşağıdaki kodu çalıştırırsan:

```rust
let numbers: [u32; 3] = [1, 2, 3];
```

şu bellek düzenini elde edersin:

```text
        +---+---+---+
Stack:  | 1 | 2 | 3 |
        +---+---+---+
```

Başka bir deyişle, bir dizinin boyutu `std::mem::size_of::<T>() * N`'dir; burada `T` elemanların türü, `N` ise
eleman sayısıdır.\
Her elemana `O(1)` sürede erişebilir ve onu değiştirebilirsin.
