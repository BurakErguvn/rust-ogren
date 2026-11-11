# Doğrulama {#validation}

Ticket tanımımıza geri dönelim:

```rust
struct Ticket {
    title: String,
    description: String,
    status: String,
}
```

`Ticket` struct'ımızın alanları için "ham" türler kullanıyoruz.
Bu, kullanıcıların boş başlıklı, çoooooooook uzun açıklamalı ya da anlamsız bir durumu (ör. "Funny") olan
bir ticket oluşturabileceği anlamına gelir.\
Bundan daha iyisini yapabiliriz!

## İleri Okuma {#further-reading}

- Sağladığı metotlara kapsamlı bir genel bakış için [`String`'in dokümantasyonuna](https://doc.rust-lang.org/std/string/struct.String.html)
  göz at. Egzersiz için ona ihtiyacın olacak!
