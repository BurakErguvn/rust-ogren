# Ticket ID'leri {#ticket-ids}

Ticket yönetim sistemimizi bir kez daha düşünelim.\
Ticket modelimiz şu anda şöyle görünüyor:

```rust
pub struct Ticket {
    pub title: TicketTitle,
    pub description: TicketDescription,
    pub status: Status
}
```

Burada bir şey eksik: bir ticket'ı benzersiz biçimde tanımlamak için bir **tanımlayıcı**.\
Bu tanımlayıcı her ticket için benzersiz olmalıdır. Bu, yeni bir ticket oluşturulduğunda tanımlayıcıyı otomatik olarak
üreterek garanti edilebilir.

## Modeli İyileştirmek {#refining-the-model}

Id nerede saklanmalı?\
`Ticket` struct'ına yeni bir alan ekleyebiliriz:

```rust
pub struct Ticket {
    pub id: TicketId,
    pub title: TicketTitle,
    pub description: TicketDescription,
    pub status: Status
}
```

Ama ticket'ı oluşturmadan önce id'yi bilmiyoruz. Dolayısıyla id en başından orada olamaz.\
İsteğe bağlı olması gerekirdi:

```rust
pub struct Ticket {
    pub id: Option<TicketId>,
    pub title: TicketTitle,
    pub description: TicketDescription,
    pub status: Status
}
```

Bu da ideal değil—ticket oluşturulduktan sonra id'nin her zaman orada olması gerektiğini bilsek bile, store'dan
her ticket aldığımızda `None` durumunu ele almamız gerekirdi.

En iyi çözüm, iki ayrı türle temsil edilen iki farklı ticket **durumuna** sahip olmaktır:
bir `TicketDraft` ve bir `Ticket`:

```rust
pub struct TicketDraft {
    pub title: TicketTitle,
    pub description: TicketDescription
}

pub struct Ticket {
    pub id: TicketId,
    pub title: TicketTitle,
    pub description: TicketDescription,
    pub status: Status
}
```

Bir `TicketDraft`, henüz oluşturulmamış bir ticket'tır. Id'si yoktur ve durumu da yoktur.\
Bir `Ticket`, oluşturulmuş bir ticket'tır. Bir id'si ve bir durumu vardır.\
`TicketDraft` ve `Ticket`'taki her alan kendi kısıtlarını içinde barındırdığından, mantığı iki tür arasında
tekrarlamamız gerekmez.
