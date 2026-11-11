# Hata Enum'ları {#error-enums}

Önceki egzersizdeki çözümün garip gelmiş olabilir: string'ler üzerinde match yapmak ideal değil!\
Bir iş arkadaşın `Ticket::new`'nun döndürdüğü hata mesajlarını (ör. okunabilirliği artırmak için) yeniden düzenleyebilir ve
bir anda çağıran kodun bozulabilir.

Bunu düzeltmek için gereken mekanizmayı zaten biliyorsun: enum'lar!

## Hatalara Tepki Vermek {#reacting-to-errors}

Çağıranın, oluşan belirli hataya göre farklı davranabilmesini istediğinde, farklı hata durumlarını
temsil etmek için bir enum kullanabilirsin:

```rust
// Bir string'den `u32` parse ederken oluşabilecek
// farklı hata durumlarını temsil eden bir hata enum'u.
enum U32ParseError {
    NotANumber,
    TooLarge,
    Negative,
}
```

Bir hata enum'u kullanarak farklı hata durumlarını tür sisteminde kodlamış olursun—bu durumlar, başarısız olabilen
fonksiyonun imzasının bir parçası haline gelir.\
Bu, çağıran için hata yönetimini basitleştirir, çünkü çağıran farklı hata durumlarına tepki vermek için bir `match` ifadesi
kullanabilir:

```rust
match s.parse_u32() {
    Ok(n) => n,
    Err(U32ParseError::Negative) => 0,
    Err(U32ParseError::TooLarge) => u32::MAX,
    Err(U32ParseError::NotANumber) => {
        panic!("Not a number: {}", s);
    }
}
```
