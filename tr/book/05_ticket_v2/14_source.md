# `Error::source` {#errorsource}

`Error` trait'ini ele alışımızı tamamlamak için konuşmamız gereken bir şey daha var: `source` metodu.

```rust
// Bu sefer tam tanım!
pub trait Error: Debug + Display {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
```

`source` metodu, varsa **hata nedenine** erişmenin bir yoludur.\
Hatalar sıklıkla zincirlenir, yani bir hata diğerinin nedenidir: daha düşük seviyeli bir hatadan (ör. veritabanının
host adı çözümlenemiyor) kaynaklanan yüksek seviyeli bir hatan olur (ör. veritabanına bağlanılamıyor).
`source` metodu, hata zincirinin tamamında "gezinmeni" sağlar; bu genellikle log'larda hata bağlamını kaydederken kullanılır.

## `source`'u Implement Etmek {#implementing-source}

`Error` trait'i, her zaman `None` döndüren (yani altta yatan bir neden olmadığını belirten) varsayılan bir implementation sağlar.
Bu yüzden önceki egzersizlerde `source` ile ilgilenmen gerekmedi.\
Hata türüne bir neden sağlamak için bu varsayılan implementation'ı override edebilirsin.

```rust
use std::error::Error;

#[derive(Debug)]
struct DatabaseError {
    source: std::io::Error
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Failed to connect to the database")
    }
}

impl std::error::Error for DatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}
```

Bu örnekte `DatabaseError`, kaynağı olarak bir `std::io::Error`'ı sarmalar.
Ardından `source` metodunu, çağrıldığında bu kaynağı döndürecek şekilde override ederiz.

## `&(dyn Error + 'static)` {#dyn-error--static}

Bu `&(dyn Error + 'static)` türü de ne?\
Hadi açalım:

- `dyn Error` bir **trait object**'tir (trait nesnesi). `Error` trait'ini implement eden herhangi bir türe başvurmanın bir yoludur.
- `'static` özel bir **lifetime** (yaşam süresi) belirleyicisidir.
  `'static`, referansın "ihtiyacımız olduğu sürece", yani programın tüm çalışması boyunca geçerli olduğunu ifade eder.

Birleştirirsek: `&(dyn Error + 'static)`, `Error` trait'ini implement eden ve programın tüm çalışması boyunca
geçerli olan bir trait object'e yapılmış bir referanstır.

Şimdilik bu kavramların ikisi için de fazla endişelenme. Onları ilerideki bölümlerde daha ayrıntılı ele alacağız.

## `source`'u `thiserror` İle Implement Etmek {#implementing-source-using-thiserror}

`thiserror`, hata türlerin için `source`'u otomatik olarak implement etmenin üç yolunu sunar:

- `source` adlı bir alan otomatik olarak hatanın kaynağı olarak kullanılır.
  ```rust
  use thiserror::Error;

  #[derive(Error, Debug)]
  pub enum MyError {
      #[error("Failed to connect to the database")]
      DatabaseError {
          source: std::io::Error
      }
  }
  ```
- `#[source]` attribute'u (öznitelik) ile işaretlenmiş bir alan otomatik olarak hatanın kaynağı olarak kullanılır.
  ```rust
  use thiserror::Error;

  #[derive(Error, Debug)]
  pub enum MyError {
      #[error("Failed to connect to the database")]
      DatabaseError {
          #[source]
          inner: std::io::Error
      }
  }
  ```
- `#[from]` attribute'u ile işaretlenmiş bir alan otomatik olarak hatanın kaynağı olarak kullanılır **ve**
  `thiserror`, işaretlenmiş türü senin hata türüne dönüştürmek için otomatik olarak bir `From` implementation'ı üretir.
  ```rust
  use thiserror::Error;

  #[derive(Error, Debug)]
  pub enum MyError {
      #[error("Failed to connect to the database")]
      DatabaseError {
          #[from]
          inner: std::io::Error
      }
  }
  ```

## `?` Operatörü {#the--operator}

`?` operatörü, hataları yukarı iletmenin kısa bir yoludur.\
`Result` döndüren bir fonksiyonda kullanıldığında, `Result` bir `Err` ise bir hatayla erken döner.

Örneğin:

```rust
use std::fs::File;

fn read_file() -> Result<String, std::io::Error> {
    let mut file = File::open("file.txt")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}
```

şuna eşdeğerdir:

```rust
use std::fs::File;

fn read_file() -> Result<String, std::io::Error> {
    let mut file = match File::open("file.txt") {
        Ok(file) => file,
        Err(e) => {
            return Err(e);
        }
    };
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => (),
        Err(e) => {
            return Err(e);
        }
    }
    Ok(contents)
}
```

`?` operatörünü hata yönetimi kodunu önemli ölçüde kısaltmak için kullanabilirsin.\
Özellikle `?` operatörü, bir dönüşüm mümkünse (yani uygun bir `From` implementation'ı varsa), başarısız olabilen
işlemin hata türünü fonksiyonun hata türüne otomatik olarak dönüştürür
