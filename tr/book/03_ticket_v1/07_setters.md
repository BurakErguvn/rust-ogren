# Mutable (Değiştirilebilir) Referanslar {#mutable-references}

Erişim metotların artık şöyle görünmeli:

```rust
impl Ticket {
    pub fn title(&self) -> &String {
        &self.title
    }

    pub fn description(&self) -> &String {
        &self.description
    }

    pub fn status(&self) -> &String {
        &self.status
    }
}
```

Oraya buraya serpiştirilmiş birkaç `&` işi gördü!\
Artık bir `Ticket` instance'ının (örneğinin) alanlarına, onu bu süreçte tüketmeden erişmenin bir yoluna sahibiz.
Sırada `Ticket` struct'ımızı **setter metotlarla** nasıl geliştirebileceğimize bakalım.

## Setter'lar {#setters}

Setter metotlar, kullanıcıların `Ticket`'ın private alanlarının değerlerini değiştirmesine izin verirken
invariant'larının (değişmezlerinin) korunmasını da sağlar (yani bir `Ticket`'ın başlığını boş bir string'e ayarlayamazsın).

Rust'ta setter'ları implement etmenin iki yaygın yolu vardır:

- Girdi olarak `self` almak.
- Girdi olarak `&mut self` almak.

### Girdi Olarak `self` Almak {#taking-self-as-input}

İlk yaklaşım şöyle görünür:

```rust
impl Ticket {
    pub fn set_title(mut self, new_title: String) -> Self {
        // Yeni başlığı doğrula [...]
        self.title = new_title;
        self
    }
}
```

`self`'in ownership'ini (sahipliğini) alır, başlığı değiştirir ve değiştirilmiş `Ticket` instance'ını döndürür.\
Onu şöyle kullanırsın:

```rust
let ticket = Ticket::new(
    "Title".into(), 
    "Description".into(), 
    "To-Do".into()
);
let ticket = ticket.set_title("New title".into());
```

`set_title`, `self`'in ownership'ini aldığından (yani onu **tükettiğinden**) sonucu bir değişkene yeniden atamamız gerekir.
Yukarıdaki örnekte aynı değişken adını yeniden kullanmak için **değişken shadowing**'inden (gölgeleme) yararlanıyoruz:
mevcut bir değişkenle aynı ada sahip yeni bir değişken bildirdiğinde, yeni değişken eskisini **gölgeler**. Bu,
Rust kodunda yaygın bir kalıptır.

`self`-setter'lar, birden fazla alanı aynı anda değiştirmen gerektiğinde oldukça iyi çalışır: birden fazla çağrıyı birbirine zincirleyebilirsin!

```rust
let ticket = ticket
    .set_title("New title".into())
    .set_description("New description".into())
    .set_status("In Progress".into());
```

### Girdi Olarak `&mut self` Almak {#taking-mut-self-as-input}

Setter'lara ikinci yaklaşım, yani `&mut self` kullanan yaklaşım ise şöyle görünür:

```rust
impl Ticket {
    pub fn set_title(&mut self, new_title: String) {
        // Yeni başlığı doğrula [...]
        
        self.title = new_title;
    }
}
```

Bu sefer metot girdi olarak `self`'e mutable bir referans alır, başlığı değiştirir, o kadar.
Hiçbir şey döndürülmez.

Onu şöyle kullanırsın:

```rust
let mut ticket = Ticket::new(
    "Title".into(),
    "Description".into(),
    "To-Do".into()
);
ticket.set_title("New title".into());

// Değiştirilmiş ticket'ı kullan
```

Ownership çağıranda kalır, bu yüzden orijinal `ticket` değişkeni hâlâ geçerlidir. Sonucu yeniden atamamız gerekmez.
Yine de `ticket`'ı mutable olarak işaretlememiz gerekir, çünkü ona mutable bir referans alıyoruz.

`&mut`-setter'ların bir dezavantajı var: birden fazla çağrıyı birbirine zincirleyemezsin.
Değiştirilmiş `Ticket` instance'ını döndürmedikleri için, ilkinin sonucu üzerinde başka bir setter çağıramazsın.
Her setter'ı ayrı ayrı çağırman gerekir:

```rust
ticket.set_title("New title".into());
ticket.set_description("New description".into());
ticket.set_status("In Progress".into());
```
