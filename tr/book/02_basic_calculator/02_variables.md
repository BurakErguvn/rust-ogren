# Değişkenler {#variables}

Rust'ta **değişken** bildirmek için `let` anahtar kelimesini kullanabilirsin.\
Örneğin:

```rust
let x = 42;
```

Yukarıda `x` adında bir değişken tanımladık ve ona `42` değerini atadık.

## Tür {#type}

Rust'taki her değişkenin bir türü olmalıdır. Bu tür ya derleyici tarafından çıkarsanır ya da geliştirici tarafından
açıkça belirtilir.

### Açık Tür Belirtimi {#explicit-type-annotation}

Değişken adından sonra iki nokta `:` ve ardından türü ekleyerek değişkenin türünü belirtebilirsin. Örneğin:

```rust
// let <değişken_adı>: <tür> = <ifade>;
let x: u32 = 42;
```

Yukarıdaki örnekte `x`'in türünü açıkça `u32` olarak kısıtladık.

### Tür Çıkarımı {#type-inference}

Bir değişkenin türünü belirtmezsek, derleyici onu değişkenin kullanıldığı bağlama bakarak çıkarsamaya
çalışır.

```rust
let x = 42;
let y: u32 = x;
```

Yukarıdaki örnekte `x`'in türünü belirtmedik.\
`x` daha sonra, türü açıkça `u32` olarak belirtilmiş olan `y`'ye atanıyor. Rust otomatik type coercion (örtük tür dönüşümü) yapmadığı için
derleyici `x`'in türünü `u32` olarak çıkarsar—bu, `y` ile aynı tür ve programın hatasız derlenmesini sağlayacak
tek türdür.

### Çıkarımın Sınırları {#inference-limitations}

Derleyicinin, kullanımına bakarak doğru değişken türünü çıkarsayabilmesi için bazen biraz yardıma ihtiyacı olur.\
Bu durumlarda bir derleme hatası alırsın ve derleyici, durumu netleştirmek için açık bir tür ipucu vermeni
ister.

## Fonksiyon Argümanları Birer Değişkendir {#function-arguments-are-variables}

Her kahraman pelerin giymez, her değişken de `let` ile bildirilmez.\
Fonksiyon argümanları da birer değişkendir!

```rust
fn add_one(x: u32) -> u32 {
    x + 1
}
```

Yukarıdaki örnekte `x`, `u32` türünde bir değişkendir.\
`x` ile, `let` kullanılarak bildirilmiş bir değişken arasındaki tek fark, fonksiyon argümanlarının türünün **mutlaka**
açıkça bildirilmesi gerektiğidir. Derleyici bunu senin için çıkarsamaz.\
Bu kısıt, Rust derleyicisinin (ve biz insanların!) fonksiyonun imzasını, implementation'ına bakmak zorunda kalmadan
anlamasını sağlar. Bu, derleme hızı için büyük bir kazanç[^speed]!

## İlk Değer Atama {#initialization}

Bir değişkeni bildirirken ona ilk değer atamak zorunda değilsin.\
Örneğin

```rust
let x: u32;
```

geçerli bir değişken bildirimidir.\
Ancak değişkeni kullanmadan önce ona ilk değer atamalısın. Atamazsan derleyici hata verir:

```rust
let x: u32;
let y = x + 1;
```

bir derleme hatası verir:

```text
error[E0381]: used binding `x` isn't initialized
 --> src/main.rs:3:9
  |
2 | let x: u32;
  |     - binding declared here but left uninitialized
3 | let y = x + 1;
  |         ^ `x` used here but it isn't initialized
  |
help: consider assigning a value
  |
2 | let x: u32 = 0;
  |            +++
```

[^speed]: Konu derleme hızı olunca Rust derleyicisi alabileceği her yardıma muhtaçtır.
