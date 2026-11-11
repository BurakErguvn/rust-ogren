# Güncelleme İşlemleri {#update-operations}

Şimdiye kadar yalnızca ekleme ve getirme işlemlerini implement ettik.\
Sistemi bir güncelleme işlemi sunacak şekilde nasıl genişletebileceğimize bakalım.

## Eski Usul Güncellemeler {#legacy-updates}

Sistemin thread'siz (iş parçacığı kullanmayan) sürümünde güncellemeler oldukça basitti: `TicketStore`, çağıranın bir ticket'a
mutable (değiştirilebilir) bir referans almasını ve ardından onu değiştirmesini sağlayan bir `get_mut` metodu sunuyordu.

## Çok Thread'li Güncellemeler {#multithreaded-updates}

Aynı strateji mevcut çok thread'li sürümde işe yaramaz. Borrow checker bizi
durdurur: `&mut Ticket`, `'static` lifetime'ını (yaşam süresi) karşılamadığı için `SyncSender<&mut Ticket>` `'static` değildir; dolayısıyla
`std::thread::spawn`'a verilen closure tarafından yakalanamazlar.

Bu kısıtlamayı aşmanın birkaç yolu var. İlerleyen egzersizlerde bunlardan birkaçını inceleyeceğiz.

### Patch'leme {#patching}

Bir kanal üzerinden `&mut Ticket` gönderemeyiz, dolayısıyla istemci tarafında değişiklik yapamayız.\
Sunucu tarafında değişiklik yapabilir miyiz?

Sunucuya neyin değişmesi gerektiğini söylersek yapabiliriz. Başka bir deyişle, sunucuya bir **patch** (yama) gönderirsek:

```rust
struct TicketPatch {
    id: TicketId,
    title: Option<TicketTitle>,
    description: Option<TicketDescription>,
    status: Option<TicketStatus>,
}
```

`id` alanı zorunludur, çünkü güncellenmesi gereken ticket'ı tanımlamak için gereklidir.\
Diğer tüm alanlar isteğe bağlıdır:

- Bir alan `None` ise, o alanın değiştirilmemesi gerektiği anlamına gelir.
- Bir alan `Some(value)` ise, o alanın `value` olarak değiştirilmesi gerektiği anlamına gelir.
