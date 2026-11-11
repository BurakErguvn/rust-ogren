# Struct'lar {#structs}

Her ticket için üç bilgiyi takip etmemiz gerekiyor:

- Bir başlık
- Bir açıklama
- Bir durum

Bunları temsil etmek için bir [`String`](https://doc.rust-lang.org/std/string/struct.String.html)
kullanarak başlayabiliriz. `String`, [UTF-8 ile kodlanmış](https://en.wikipedia.org/wiki/UTF-8) metni temsil etmek için
Rust'ın standart kütüphanesinde tanımlanmış türdür.

Peki bu üç bilgiyi tek bir varlıkta nasıl **birleştiririz**?

## Bir `struct` Tanımlamak {#defining-a-struct}

Bir `struct`, **yeni bir Rust türü** tanımlar.

```rust
struct Ticket {
    title: String,
    description: String,
    status: String
}
```

Struct, diğer programlama dillerinde sınıf ya da nesne diyeceğin şeye oldukça benzer.

## Alanları Tanımlamak {#defining-fields}

Yeni tür, diğer türlerin **alanlar** olarak bir araya getirilmesiyle oluşturulur.\
Her alanın, iki nokta üst üste (`:`) ile ayrılmış bir adı ve bir türü olmalıdır. Birden fazla alan varsa, bunlar virgülle (`,`) ayrılır.

Aşağıdaki `Configuration` struct'ında görebileceğin gibi, alanların aynı türde olması gerekmez:

```rust
struct Configuration {
   version: u32,
   active: bool
}
```

## Instance (Örnek) Oluşturma {#instantiation}

Her alanın değerini belirterek bir struct'ın instance'ını oluşturabilirsin:

```rust
// Söz dizimi: <StructName> { <field_name>: <value>, ... }
let ticket = Ticket {
    title: "Build a ticket system".into(),
    description: "A Kanban board".into(),
    status: "Open".into()
};
```

## Alanlara Erişmek {#accessing-fields}

Bir struct'ın alanlarına `.` operatörünü kullanarak erişebilirsin:

```rust
// Alan erişimi
let x = ticket.description;
```

## Metotlar {#methods}

Struct'larımıza **metotlar** tanımlayarak davranış ekleyebiliriz.\
Örnek olarak `Ticket` struct'ını kullanalım:

```rust
impl Ticket {
    fn is_open(self) -> bool {
        self.status == "Open"
    }
}

// Söz dizimi:
// impl <StructName> {
//    fn <method_name>(<parameters>) -> <return_type> {
//        // Metot gövdesi
//    }
// }
```

Metotlar fonksiyonlara oldukça benzer, ancak iki temel farkları vardır:

1. metotlar bir **`impl` bloğu** içinde tanımlanmalıdır
2. metotlar ilk parametre olarak `self` kullanabilir.
   `self` bir anahtar kelimedir ve metodun üzerinde çağrıldığı struct instance'ını temsil eder.

### `self` {#self}

Bir metot ilk parametre olarak `self` alıyorsa, **metot çağrı söz dizimi** kullanılarak çağrılabilir:

```rust
// Metot çağrı söz dizimi: <instance>.<method_name>(<parameters>)
let is_open = ticket.is_open();
```

Bu, [önceki bölümde](../02_basic_calculator/09_saturating.md) `u32` değerleri üzerinde saturating arithmetic (doygun aritmetik)
işlemleri yapmak için kullandığın çağırma söz diziminin aynısıdır.

### Statik Metotlar {#static-methods}

Bir metot ilk parametre olarak `self` almıyorsa, bu bir **statik metottur**.

```rust
struct Configuration {
    version: u32,
    active: bool
}

impl Configuration {
    // `default`, `Configuration` üzerinde tanımlı bir statik metottur
    fn default() -> Configuration {
        Configuration { version: 0, active: false }
    }
}
```

Statik bir metodu çağırmanın tek yolu **fonksiyon çağrı söz dizimini** kullanmaktır:

```rust
// Fonksiyon çağrı söz dizimi: <StructName>::<method_name>(<parameters>)
let default_config = Configuration::default();
```

### Eşdeğerlik {#equivalence}

Fonksiyon çağrı söz dizimini, ilk parametre olarak `self` alan metotlar için bile kullanabilirsin:

```rust
// Fonksiyon çağrı söz dizimi:
//   <StructName>::<method_name>(<instance>, <parameters>)
let is_open = Ticket::is_open(ticket);
```

Fonksiyon çağrı söz dizimi, `ticket`'ın metodun ilk parametresi olan `self` olarak kullanıldığını oldukça net gösterir,
ama kesinlikle daha ayrıntılı bir yazım gerektirir. Mümkün olduğunda metot çağrı söz dizimini tercih et.
