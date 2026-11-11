# Kısa Ve Öz Dallanma {#concise-branching}

Önceki egzersizdeki çözümün muhtemelen şuna benziyor:

```rust
impl Ticket {
    pub fn assigned_to(&self) -> &str {
        match &self.status {
            Status::InProgress { assigned_to } => assigned_to,
            Status::Done | Status::ToDo => {
                panic!(
                    "Only `In-Progress` tickets can be \
                    assigned to someone"
                )
            }
        }
    }
}
```

Yalnızca `Status::InProgress` variant'ı ile ilgileniyorsun.
Diğer tüm variant'lar üzerinde match yapmana gerçekten gerek var mı?

Yeni yapılar imdada yetişiyor!

## `if let` {#if-let}

`if let` yapısı, diğer tüm variant'ları ele almak zorunda kalmadan
bir enum'un tek bir variant'ı üzerinde match yapmanı sağlar.

`assigned_to` metodunu basitleştirmek için `if let`'i şöyle kullanabilirsin:

```rust
impl Ticket {
    pub fn assigned_to(&self) -> &str {
        if let Status::InProgress { assigned_to } = &self.status {
            assigned_to
        } else {
            panic!(
                "Only `In-Progress` tickets can be assigned to someone"
            );
        }
    }
}
```

## `let/else` {#letelse}

`else` dalının erken dönmesi amaçlanıyorsa (bir panic de erken dönmek sayılır!),
`let/else` yapısını kullanabilirsin:

```rust
impl Ticket {
    pub fn assigned_to(&self) -> &str {
        let Status::InProgress { assigned_to } = &self.status else {
            panic!(
                "Only `In-Progress` tickets can be assigned to someone"
            );
        };
        assigned_to
    }
}
```

Bu yapı, destructure edilmiş (parçalarına ayrılmış) değişkeni herhangi bir "sağa kayma"ya yol açmadan
atamanı sağlar; yani değişken, kendisinden önce gelen kodla aynı girinti düzeyinde atanır.

## Stil {#style}

Hem `if let` hem de `let/else` idiomatik Rust yapılarıdır.\
Kodunun okunabilirliğini artırmak için onları uygun gördüğün şekilde kullan,
ama abartma: ihtiyaç duyduğunda `match` her zaman oradadır.
