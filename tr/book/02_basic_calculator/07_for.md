# Döngüler, 2. Kısım: `for` {#loops-part-2-for}

Bir sayaç değişkenini elle artırmak zorunda olmak biraz zahmetlidir. Üstelik bu kalıp son derece yaygındır!\
Bunu kolaylaştırmak için Rust, bir değer aralığı üzerinde gezinmenin daha kısa bir yolunu sunar: `for` döngüsü.

## `for` Döngüsü {#the-for-loop}

`for` döngüsü, bir iterator'daki[^iterator] her eleman için bir kod bloğunu çalıştırmanın bir yoludur.

Genel söz dizimi şöyledir:

```rust
for <element> in <iterator> {
    // çalıştırılacak kod
}
```

## Aralıklar {#ranges}

Rust'ın standart kütüphanesi, bir sayı serisi üzerinde gezinmek için kullanılabilecek **aralık** türünü
sunar[^weird-ranges].

Örneğin 1'den 5'e kadar olan sayıları toplamak istersek:

```rust
let mut sum = 0;
for i in 1..=5 {
    sum += i;
}
```

Döngü her çalıştığında, kod bloğu çalıştırılmadan önce `i`'ye aralıktaki bir sonraki değer atanır.

Rust'ta beş çeşit aralık vardır:

- `1..5`: (Yarı açık) bir aralık. 1'den 4'e kadar tüm sayıları içerir. Son değeri, yani 5'i içermez.
- `1..=5`: Kapsayıcı bir aralık. 1'den 5'e kadar tüm sayıları içerir. Son değeri, yani 5'i içerir.
- `1..`: Ucu açık bir aralık. 1'den sonsuza kadar tüm sayıları içerir (aslında, tam sayı türünün maksimum değerine kadar).
- `..5`: Tam sayı türünün minimum değerinden başlayıp 4'te biten bir aralık. Son değeri, yani 5'i içermez.
- `..=5`: Tam sayı türünün minimum değerinden başlayıp 5'te biten bir aralık. Son değeri, yani 5'i içerir.

`for` döngüsünü, başlangıç noktasının açıkça belirtildiği ilk üç aralık türüyle kullanabilirsin. Son iki aralık
türü ise ileride ele alacağımız başka bağlamlarda kullanılır.

Bir aralığın uç değerlerinin tam sayı literal'i olması gerekmez—değişken ya da ifade de olabilirler!

Örneğin:

```rust
let end = 5;
let mut sum = 0;

for i in 1..(end + 1) {
    sum += i;
}
```

## İleri Okuma {#further-reading}

- [`for` döngüsü dokümantasyonu](https://doc.rust-lang.org/std/keyword.for.html)

[^iterator]: Kursun ilerleyen kısımlarında neyin "iterator" sayıldığına dair kesin bir tanım vereceğiz.
Şimdilik onu, üzerinde döngü kurabileceğin bir değerler serisi olarak düşün.
[^weird-ranges]: Aralıkları başka türlerle de (ör. karakterler ve IP adresleri) kullanabilirsin,
ama günlük Rust programlamada en yaygın durum kesinlikle tam sayılardır.
