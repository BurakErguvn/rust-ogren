# Başarısız Olabilirlik {#fallibility}

Önceki egzersizdeki `Ticket::new` fonksiyonuna tekrar bakalım:

```rust
impl Ticket {
    pub fn new(
        title: String, 
        description: String, 
        status: Status
    ) -> Ticket {
        if title.is_empty() {
            panic!("Title cannot be empty");
        }
        if title.len() > 50 {
            panic!("Title cannot be longer than 50 bytes");
        }
        if description.is_empty() {
            panic!("Description cannot be empty");
        }
        if description.len() > 500 {
            panic!("Description cannot be longer than 500 bytes");
        }

        Ticket {
            title,
            description,
            status,
        }
    }
}
```

Kontrollerden biri başarısız olur olmaz fonksiyon panic'ler.
Bu ideal değil, çünkü çağırana **hatayı ele alma** şansı vermez.

Artık Rust'ın hata yönetimi için temel mekanizması olan `Result` türünü tanıtmanın zamanı geldi.

## `Result` Türü {#the-result-type}

`Result` türü, standart kütüphanede tanımlanmış bir enum'dur:

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

İki variant'ı vardır:

- `Ok(T)`: başarılı bir işlemi temsil eder. İşlemin çıktısı olan `T`'yi tutar.
- `Err(E)`: başarısız bir işlemi temsil eder. Oluşan hata olan `E`'yi tutar.

Hem `Ok` hem de `Err` generic'tir; bu sayede başarı ve hata durumları için kendi türlerini belirleyebilirsin.

## İstisna Yok {#no-exceptions}

Rust'ta kurtarılabilir hatalar **değer olarak temsil edilir**.\
Bunlar yalnızca bir türün instance'ıdır (örneğidir); diğer her değer gibi bir yerden bir yere aktarılır ve işlenirler.
Bu, hataları bildirmek için **istisnaların** kullanıldığı Python veya C# gibi diğer dillerden önemli bir farktır.

İstisnalar, üzerine akıl yürütmesi zor olabilen ayrı bir kontrol akışı yolu oluşturur.\
Yalnızca bir fonksiyonun imzasına bakarak onun bir istisna fırlatıp fırlatamayacağını bilemezsin.
Yalnızca bir fonksiyonun imzasına bakarak onun **hangi** istisna türlerini fırlatabileceğini bilemezsin.\
Bunu öğrenmek için ya fonksiyonun dokümantasyonunu okuman ya da implementation'ına bakman gerekir.

İstisna yönetimi mantığının yerelliği çok zayıftır: istisnayı fırlatan kod, onu yakalayan koddan çok uzaktadır
ve ikisi arasında doğrudan bir bağlantı yoktur.

## Başarısız Olabilirlik Tür Sisteminde Kodlanır {#fallibility-is-encoded-in-the-type-system}

Rust, `Result` ile seni **başarısız olabilirliği fonksiyonun imzasında kodlamaya** zorlar.\
Bir fonksiyon başarısız olabiliyorsa (ve çağıranın hatayı ele alma şansı olmasını istiyorsan), bir `Result` döndürmelidir.

```rust
// Yalnızca imzaya bakarak bu fonksiyonun başarısız olabileceğini
// bilirsin. Ne tür hatalar bekleyeceğini görmek için
// `ParseIntError`'ı da inceleyebilirsin.
fn parse_int(s: &str) -> Result<i32, ParseIntError> {
    // ...
}
```

`Result`'ın büyük avantajı budur: başarısız olabilirliği açık hale getirir.

Yine de panic'lerin var olduğunu unutma. Tıpkı diğer dillerdeki istisnalar gibi, tür sistemi tarafından takip edilmezler.
Ama **kurtarılamaz hatalar** için tasarlanmışlardır ve idareli kullanılmalıdırlar.
