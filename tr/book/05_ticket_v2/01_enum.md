# Enumeration'lar {#enumerations}

[Önceki bir bölümde](../03_ticket_v1/02_validation.md) yazdığın doğrulama mantığına göre,
bir ticket için yalnızca birkaç geçerli durum vardır: `To-Do`, `InProgress` ve `Done`.\
`Ticket` struct'ındaki `status` alanına ya da `new` metodundaki `status`
parametresinin türüne bakarsak bu pek belli olmuyor:

```rust
#[derive(Debug, PartialEq)]
pub struct Ticket {
    title: String,
    description: String,
    status: String,
}

impl Ticket {
    pub fn new(
        title: String, 
        description: String, 
        status: String
    ) -> Self {
        // [...]
    }
}
```

Her iki durumda da `status` alanını temsil etmek için `String` kullanıyoruz.
`String` çok genel bir türdür—`status` alanının sınırlı sayıda olası değeri olduğu bilgisini
hemen iletmez. Daha da kötüsü, `Ticket::new`'yu çağıran, verdiği durumun geçerli olup olmadığını
ancak **çalışma zamanında** öğrenir.

**Enumeration**'lar (numaralandırmalar) sayesinde bundan daha iyisini yapabiliriz.

## `enum` {#enum}

Enumeration, alabileceği değerler sabit bir kümeyle sınırlı olan bir türdür; bu değerlere **variant** denir.\
Rust'ta bir enumeration'ı `enum` anahtar kelimesiyle tanımlarsın:

```rust
enum Status {
    ToDo,
    InProgress,
    Done,
}
```

`enum`, tıpkı `struct` gibi, **yeni bir Rust türü** tanımlar.
