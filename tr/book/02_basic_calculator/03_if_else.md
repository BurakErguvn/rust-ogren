# Kontrol Akışı, 1. Kısım {#control-flow-part-1}

Şimdiye kadarki tüm programlarımız oldukça basitti.\
Komutlar sırayla, yukarıdan aşağıya çalıştırılıyor, o kadar.

Biraz **dallanma** eklemenin zamanı geldi.

## `if` Blokları {#if-clauses}

`if` anahtar kelimesi, bir kod bloğunu yalnızca bir koşul doğruysa çalıştırmak için kullanılır.

İşte basit bir örnek:

```rust
let number = 3;
if number < 5 {
    println!("`number` is smaller than 5");
}
```

Bu program `number is smaller than 5` yazdırır, çünkü `number < 5` koşulu doğrudur.

### `else` Blokları {#else-clauses}

Çoğu programlama dili gibi Rust da, bir `if` ifadesindeki koşul yanlış olduğunda bir kod bloğunu çalıştırmak için
isteğe bağlı bir `else` dalını destekler.\
Örneğin:

```rust
let number = 3;

if number < 5 {
    println!("`number` is smaller than 5");
} else {
    println!("`number` is greater than or equal to 5");
}
```

### `else if` Blokları {#else-if-clauses}

Birden fazla `if` ifadesini iç içe yerleştirdiğinde kodun giderek daha da sağa kayar.

```rust
let number = 3;

if number < 5 {
    println!("`number` is smaller than 5");
} else {
    if number < 8 {
        println!("`number` is greater than or equal to 5, but smaller than 8");
    } else {
        println!("`number` is greater than or equal to 8");
    }
}
```

Birden fazla `if` ifadesini tek bir ifadede birleştirmek için `else if` anahtar kelimesini kullanabilirsin:

```rust
let number = 3;

if number < 5 {
    println!("`number` is smaller than 5");
} else if number < 8 {
    println!("`number` is greater than or equal to 5, but smaller than 8");
} else {
    println!("`number` is greater than or equal to 8");
}
```

## Boolean'lar {#booleans}

Bir `if` ifadesindeki koşul `bool` türünde, yani bir **boolean** olmalıdır.\
Boolean'lar da tam sayılar gibi Rust'ta ilkel bir türdür.

Bir boolean iki değerden birini alabilir: `true` ya da `false`.

### Truthy Ya Da Falsy Değerler Yok {#no-truthy-or-falsy-values}

Bir `if` ifadesindeki koşul boolean değilse derleme hatası alırsın.

Örneğin, aşağıdaki kod derlenmez:

```rust
let number = 3;
if number {
    println!("`number` is not zero");
}
```

Şu derleme hatasını alırsın:

```text
error[E0308]: mismatched types
 --> src/main.rs:3:8
  |
3 |     if number {
  |        ^^^^^^ expected `bool`, found integer
```

Bu, Rust'ın type coercion (örtük tür dönüşümü) konusundaki felsefesinin bir sonucudur: boolean olmayan türlerden boolean'lara otomatik bir dönüşüm yoktur.
Rust'ta JavaScript ya da Python'daki gibi **truthy** ya da **falsy** değer kavramı yoktur.\
Kontrol etmek istediğin koşul konusunda açık olman gerekir.

### Karşılaştırma Operatörleri {#comparison-operators}

`if` ifadeleri için koşul oluşturmak amacıyla karşılaştırma operatörlerini kullanmak oldukça yaygındır.\
Tam sayılarla çalışırken Rust'ta kullanabileceğin karşılaştırma operatörleri şunlardır:

- `==`: eşittir
- `!=`: eşit değildir
- `<`: küçüktür
- `>`: büyüktür
- `<=`: küçük ya da eşittir
- `>=`: büyük ya da eşittir

## `if/else` Bir İfadedir {#ifelse-is-an-expression}

Rust'ta `if` ifadeleri deyim değil, **ifadedir**: bir değer döndürürler.\
Bu değer bir değişkene atanabilir ya da başka ifadelerde kullanılabilir. Örneğin:

```rust
let number = 3;
let message = if number < 5 {
    "smaller than 5"
} else {
    "greater than or equal to 5"
};
```

Yukarıdaki örnekte `if`'in her dalı sonuç olarak bir string literal'i verir,
bu değer de ardından `message` değişkenine atanır.\
Tek gereklilik, iki `if` dalının da aynı türü döndürmesidir.
