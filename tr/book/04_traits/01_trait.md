# Trait'ler {#traits}

`Ticket` türümüze tekrar bakalım:

```rust
pub struct Ticket {
    title: String,
    description: String,
    status: String,
}
```

Şimdiye kadar tüm testlerimiz `Ticket`'ın alanlarını kullanarak assertion yaptı.

```rust
assert_eq!(ticket.title(), "A new title");
```

Peki iki `Ticket` instance'ını (örneğini) doğrudan karşılaştırmak isteseydik?

```rust
let ticket1 = Ticket::new(/* ... */);
let ticket2 = Ticket::new(/* ... */);
ticket1 == ticket2
```

Derleyici bizi durdurur:

```text
error[E0369]: binary operation `==` cannot be applied to type `Ticket`
  --> src/main.rs:18:13
   |
18 |     ticket1 == ticket2
   |     ------- ^^ ------- Ticket
   |     |
   |     Ticket
   |
note: an implementation of `PartialEq` might be missing for `Ticket`
```

`Ticket` yeni bir türdür. Kutudan çıktığı haliyle ona **iliştirilmiş hiçbir davranış yoktur**.\
Rust, iki `Ticket` instance'ını nasıl karşılaştıracağını, sırf `String`'ler içeriyorlar diye sihirli bir şekilde çıkarsamaz.

Yine de Rust derleyicisi bizi doğru yöne itiyor: `PartialEq` için bir implementation'ın eksik olabileceğini
söylüyor. `PartialEq` bir **trait**'tir!

## Trait'ler Nedir? {#what-are-traits}

Trait'ler, Rust'ın **arayüz** tanımlama yoludur.\
Bir trait, bir türün o trait'in sözleşmesini karşılamak için implement etmesi gereken bir metotlar kümesi tanımlar.

### Bir Trait Tanımlamak {#defining-a-trait}

Bir trait tanımının söz dizimi şöyledir:

```rust
trait <TraitName> {
    fn <method_name>(<parameters>) -> <return_type>;
}
```

Örneğin, kendisini implement eden türlerin bir `is_zero` metodu tanımlamasını şart koşan `MaybeZero` adlı bir trait tanımlayabiliriz:

```rust
trait MaybeZero {
    fn is_zero(self) -> bool;
}
```

### Bir Trait'i Implement Etmek {#implementing-a-trait}

Bir tür için bir trait'i implement etmek amacıyla, tıpkı sıradan[^inherent] metotlarda yaptığımız gibi `impl` anahtar kelimesini kullanırız,
ama söz dizimi biraz farklıdır:

```rust
impl <TraitName> for <TypeName> {
    fn <method_name>(<parameters>) -> <return_type> {
        // Metot gövdesi
    }
}
```

Örneğin, `MaybeZero` trait'ini özel bir sayı türü olan `WrappingU32` için implement etmek istersek:

```rust
pub struct WrappingU32 {
    inner: u32,
}

impl MaybeZero for WrappingU32 {
    fn is_zero(self) -> bool {
        self.inner == 0
    }
}
```

### Bir Trait Metodunu Çağırmak {#invoking-a-trait-method}

Bir trait metodunu çağırmak için, tıpkı sıradan metotlarda yaptığımız gibi `.` operatörünü kullanırız:

```rust
let x = WrappingU32 { inner: 5 };
assert!(!x.is_zero());
```

Bir trait metodunu çağırabilmek için iki koşulun sağlanması gerekir:

- Tür, trait'i implement etmelidir.
- Trait kapsamda olmalıdır.

İkincisini sağlamak için trait için bir `use` deyimi eklemen gerekebilir:

```rust
use crate::MaybeZero;
```

Şu durumlarda buna gerek yoktur:

- Trait, çağrının yapıldığı modülün içinde tanımlanmışsa.
- Trait, standart kütüphanenin **prelude**'unda tanımlanmışsa.
  Prelude, her Rust programına otomatik olarak import edilen bir trait ve tür kümesidir.
  Sanki her Rust modülünün başına `use std::prelude::*;` eklenmiş gibidir.

Prelude'daki trait ve türlerin listesini
[Rust dokümantasyonunda](https://doc.rust-lang.org/std/prelude/index.html) bulabilirsin.

[^inherent]: Bir trait kullanmadan doğrudan bir tür üzerinde tanımlanan metoda **inherent metot** (türe doğrudan tanımlanmış metot) da denir.
