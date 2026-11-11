# Null Olabilirlik {#nullability}

`assigned` metodu için yazdığımız implementation oldukça kaba: yapılacak ve tamamlanmış ticket'lar için panic'lemek ideal olmaktan çok uzak.\
**Rust'ın `Option` türünü** kullanarak daha iyisini yapabiliriz.

## `Option` {#option}

`Option`, **null olabilen değerleri** temsil eden bir Rust türüdür.\
Rust'ın standart kütüphanesinde tanımlanmış bir enum'dur:

```rust
enum Option<T> {
    Some(T),
    None,
}
```

`Option`, bir değerin var (`Some(T)`) ya da yok (`None`) olabileceği fikrini kodlar.\
Ayrıca seni **her iki durumu da açıkça ele almaya** zorlar. Null olabilen bir değerle çalışırken
`None` durumunu ele almayı unutursan bir derleyici hatası alırsın.\
Bu, `null` kontrolünü unutabildiğin ve böylece bir çalışma zamanı hatasına yol açabildiğin
diğer dillerdeki "örtük" null olabilirliğe göre önemli bir gelişmedir.

## `Option`'ın Tanımı {#options-definition}

`Option`'ın tanımı, daha önce görmediğin bir Rust yapısını kullanır: **tuple benzeri variant'lar**.

### Tuple Benzeri Variant'lar {#tuple-like-variants}

`Option`'ın iki variant'ı vardır: `Some(T)` ve `None`.\
`Some` bir **tuple benzeri variant**'tır: **isimsiz alanlar** tutan bir variant'tır.

Tuple benzeri variant'lar, saklanacak tek bir alan olduğunda, özellikle de `Option` gibi bir
"wrapper" (sarmalayıcı) türe baktığımızda sıklıkla kullanılır.

### Tuple Benzeri Struct'lar {#tuple-like-structs}

Bunlar enum'lara özgü değildir—tuple benzeri struct'lar da tanımlayabilirsin:

```rust
struct Point(i32, i32);
```

Ardından bir `Point` instance'ının (örneğinin) iki alanına konum indeksleriyle erişebilirsin:

```rust
let point = Point(3, 4);
let x = point.0;
let y = point.1;
```

### Tuple'lar {#tuples}

Henüz tuple'ları görmemişken bir şeyin tuple benzeri olduğunu söylemek tuhaf!\
Tuple'lar, ilkel bir Rust türünün bir başka örneğidir.
Sabit sayıda değeri, (farklı olabilen) türleriyle birlikte bir araya toplarlar:

```rust
// İki değer, aynı tür
let first: (i32, i32) = (3, 4);
// Üç değer, farklı türler
let second: (i32, u32, u8) = (-42, 3, 8);
```

Söz dizimi basittir: değerlerin türlerini parantez içinde, virgülle ayırarak listelersin.
Bir tuple'ın alanlarına nokta gösterimi ve alan indeksiyle erişebilirsin:

```rust
assert_eq!(second.0, -42);
assert_eq!(second.1, 3);
assert_eq!(second.2, 8);
```

Tuple'lar, ayrı bir struct türü tanımlamaya üşendiğinde değerleri bir araya getirmenin pratik bir yoludur.
