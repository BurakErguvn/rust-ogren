# Trait'leri Implement Etmek {#implementing-traits}

Bir tür başka bir crate'te tanımlandığında (örneğin Rust'ın standart kütüphanesinden `u32`),
onun için doğrudan yeni metotlar tanımlayamazsın. Denersen:

```rust
impl u32 {
    fn is_even(&self) -> bool {
        self % 2 == 0
    }
}
```

derleyici şikâyet eder:

```text
error[E0390]: cannot define inherent `impl` for primitive types
  |
1 | impl u32 {
  | ^^^^^^^^
  |
  = help: consider using an extension trait instead
```

## Extension Trait {#extension-trait}

**Extension trait**, temel amacı `u32` gibi yabancı türlere yeni metotlar
iliştirmek olan bir trait'tir.
Önceki egzersizde `IsEven` trait'ini tanımlayıp ardından onu `i32` ve `u32` için implement ederek
uyguladığın kalıp tam olarak buydu. Bundan sonra, `IsEven` kapsamda olduğu sürece
bu türler üzerinde `is_even`'ı serbestçe çağırabilirsin.

```rust
// Trait'i kapsama al
use my_library::IsEven;

fn main() {
    // Metodunu, trait'i implement eden bir tür üzerinde çağır
    if 4.is_even() {
        // [...]
    }
}
```

## Tek Bir Implementation {#one-implementation}

Yazabileceğin trait implementation'larının bazı sınırlamaları vardır.\
En basit ve en anlaşılır olanı şudur: bir crate içinde aynı trait'i aynı tür için
iki kez implement edemezsin.

Örneğin:

```rust
trait IsEven {
    fn is_even(&self) -> bool;
}

impl IsEven for u32 {
    fn is_even(&self) -> bool {
        true
    }
}

impl IsEven for u32 {
    fn is_even(&self) -> bool {
        false
    }
}
```

Derleyici bunu reddeder:

```text
error[E0119]: conflicting implementations of trait `IsEven` for type `u32`
   |
5  | impl IsEven for u32 {
   | ------------------- first implementation here
...
11 | impl IsEven for u32 {
   | ^^^^^^^^^^^^^^^^^^^ conflicting implementation for `u32`
```

`IsEven::is_even` bir `u32` değeri üzerinde çağrıldığında hangi trait implementation'ının kullanılacağı
konusunda hiçbir belirsizlik olamaz; bu yüzden yalnızca bir tane olabilir.

## Orphan Rule (Yetim Kuralı) {#orphan-rule}

Birden fazla crate işin içine girdiğinde işler daha incelikli hale gelir.
Özellikle, aşağıdakilerden en az birinin doğru olması gerekir:

- Trait, mevcut crate'te tanımlanmıştır
- Trait'i implement eden tür, mevcut crate'te tanımlanmıştır

Buna Rust'ın **orphan rule**'u denir. Amacı, metot çözümleme sürecini
belirsizlikten arındırmaktır.

Şu durumu hayal et:

- `A` crate'i `IsEven` trait'ini tanımlar
- `B` crate'i `IsEven`'ı `u32` için implement eder
- `C` crate'i `u32` için `IsEven` trait'inin (farklı) bir implementation'ını sağlar
- `D` crate'i hem `B`'ye hem de `C`'ye bağımlıdır ve `1.is_even()` çağırır

Hangi implementation kullanılmalı? `B`'de tanımlanan mı? Yoksa `C`'de tanımlanan mı?\
İyi bir cevabı yok; bu yüzden bu senaryoyu önlemek için orphan rule tanımlandı.
Orphan rule sayesinde ne `B` crate'i ne de `C` crate'i derlenir.

## İleri Okuma {#further-reading}

- Yukarıda belirtildiği haliyle orphan rule'un bazı incelikleri ve istisnaları vardır.
  Bu inceliklere aşina olmak istersen [referansa](https://doc.rust-lang.org/reference/items/implementations.html#trait-implementation-coherence)
  göz at.
