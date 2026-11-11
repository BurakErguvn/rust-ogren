# İndeksleme {#indexing}

`TicketStore::get`, verilen bir `TicketId` için bir `Option<&Ticket>` döndürür.\
Dizilerin ve vektörlerin elemanlarına Rust'ın indeksleme söz dizimiyle nasıl erişileceğini
daha önce görmüştük:

```rust
let v = vec![0, 1, 2];
assert_eq!(v[0], 0);
```

Aynı deneyimi `TicketStore` için nasıl sağlayabiliriz?\
Doğru tahmin ettin: bir trait implement etmemiz gerekiyor, `Index`!

## `Index` {#index}

`Index` trait'i Rust'ın standart kütüphanesinde tanımlanmıştır:

```rust
// Biraz basitleştirilmiş hali
pub trait Index<Idx>
{
    type Output;

    // Zorunlu metot
    fn index(&self, index: Idx) -> &Self::Output;
}
```

Şunlara sahiptir:

- İndeks türünü temsil eden bir generic parametre, `Idx`
- İndeks kullanılarak elde ettiğimiz türü temsil eden bir associated type (ilişkili tür), `Output`

`index` metodunun bir `Option` döndürmediğine dikkat et. Varsayım şudur: tıpkı dizi ve vec indekslemede
olduğu gibi, orada olmayan bir elemana erişmeye çalışırsan `index` panic'ler.
