# `Drop` Trait'i {#the-drop-trait}

[Destructor'ları (yıkıcıları)](../03_ticket_v1/11_destructor.md) tanıttığımızda
`drop` fonksiyonunun şunları yaptığından bahsetmiştik:

1. türün kapladığı belleği geri kazanır (yani `std::mem::size_of` byte)
2. değerin yönetiyor olabileceği tüm ek kaynakları temizler (örneğin bir `String`'in heap buffer'ı)

`Drop` trait'i tam olarak 2. adımda devreye girer.

```rust
pub trait Drop {
    fn drop(&mut self);
}
```

`Drop` trait'i, derleyicinin senin için otomatik olarak yaptıklarının ötesinde, türlerin için
_ek_ temizlik mantığı tanımlamanı sağlayan bir mekanizmadır.\
`drop` metoduna ne koyarsan, değer kapsam dışına çıktığında çalıştırılır.

## `Drop` Ve `Copy` {#drop-and-copy}

`Copy` trait'inden bahsederken, bir tür bellekte kapladığı `std::mem::size_of` byte'ın ötesinde
ek kaynaklar yönetiyorsa `Copy` implement edemeyeceğini söylemiştik.

Merak edebilirsin: derleyici bir türün ek kaynaklar yönetip yönetmediğini nereden bilir?
Doğru tahmin: `Drop` trait implementation'ları!\
Türünün açık bir `Drop` implementation'ı varsa derleyici, türüne iliştirilmiş ek kaynaklar
olduğunu varsayar ve `Copy` implement etmene izin vermez.

```rust
// Bu bir unit struct, yani alanı olmayan bir struct.
#[derive(Clone, Copy)]
struct MyType;

impl Drop for MyType {
    fn drop(&mut self) {
       // Burada hiçbir şey yapmamıza gerek yok,
       // "boş" bir Drop implementation'ının olması yeterli
    }
}
```

Derleyici şu hata mesajıyla şikâyet eder:

```text
error[E0184]: the trait `Copy` cannot be implemented for this type; 
              the type has a destructor
 --> src/lib.rs:2:17
  |
2 | #[derive(Clone, Copy)]
  |                 ^^^^ `Copy` not allowed on types with destructors
```
