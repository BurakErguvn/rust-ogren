# Mutable Slice'lar {#mutable-slices}

Slice (dilim) türlerinden (`str` ve `[T]` gibi) her söz ettiğimizde, onların immutable (değiştirilemez) ödünç alma biçimlerini (`&str` ve `&[T]`) kullandık.\
Ama slice'lar mutable (değiştirilebilir) da olabilir!

Mutable bir slice'ı şöyle oluşturursun:

```rust
let mut numbers = vec![1, 2, 3];
let slice: &mut [i32] = &mut numbers;
```

Ardından slice'taki elemanları değiştirebilirsin:

```rust
slice[0] = 42;
```

Bu, `Vec`'in ilk elemanını `42` olarak değiştirir.

## Sınırlamalar {#limitations}

Immutable ödünç almalarla çalışırken öneri açıktı: owned (sahipli) türe ait referanslar yerine slice referanslarını tercih et
(ör. `&Vec<T>` yerine `&[T]`).\
Mutable ödünç almalarda durum böyle **değildir**.

Şu senaryoyu düşün:

```rust
let mut numbers = Vec::with_capacity(2);
let mut slice: &mut [i32] = &mut numbers;
slice.push(1);
```

Derlenmez!\
`push`, slice'lar üzerinde değil, `Vec` üzerinde tanımlı bir metottur. Bu, daha genel bir ilkenin yansımasıdır: Rust,
bir slice'a eleman eklemene ya da bir slice'tan eleman çıkarmana izin vermez. Yalnızca zaten orada olan elemanları
değiştirebilir ya da yerlerine başkalarını koyabilirsin.

Bu açıdan bir `&mut Vec` ya da bir `&mut String`, bir `&mut [T]` ya da bir `&mut str`'den kesinlikle daha güçlüdür.\
Yapman gereken işlemlere göre en uygun türü seç.
