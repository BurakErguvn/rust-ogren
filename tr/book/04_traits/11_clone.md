# Değerleri Kopyalamak, 1. Kısım {#copying-values-pt-1}

Önceki bölümde ownership (sahiplik) ve borrowing (ödünç alma) kavramlarını tanıttık.\
Özellikle şunları belirtmiştik:

- Rust'taki her değerin herhangi bir anda tek bir sahibi vardır.
- Bir fonksiyon bir değerin ownership'ini aldığında ("onu tükettiğinde"), çağıran artık o değeri kullanamaz.

Bu kısıtlamalar bir ölçüde sınırlayıcı olabilir.\
Bazen bir değerin ownership'ini alan bir fonksiyonu çağırmamız gerekebilir, ama o değeri sonrasında
yine de kullanmamız gerekir.

```rust
fn consumer(s: String) { /* */ }

fn example() {
     let mut s = String::from("hello");
     consumer(s);
     s.push_str(", world!"); // hata: value borrowed here after move
}
```

İşte `Clone` tam bu noktada devreye girer.

## `Clone` {#clone}

`Clone`, Rust'ın standart kütüphanesinde tanımlanmış bir trait'tir:

```rust
pub trait Clone {
    fn clone(&self) -> Self;
}
```

Metodu `clone`, `self`'e bir referans alır ve aynı türden yeni bir **owned** (sahipli) instance (örnek) döndürür.

## İş Başında {#in-action}

Yukarıdaki örneğe dönersek, `consumer`'ı çağırmadan önce yeni bir `String` instance'ı oluşturmak için `clone` kullanabiliriz:

```rust
fn consumer(s: String) { /* */ }

fn example() {
     let mut s = String::from("hello");
     let t = s.clone();
     consumer(t);
     s.push_str(", world!"); // hata yok
}
```

`s`'in ownership'ini `consumer`'a vermek yerine, (`s`'i klonlayarak) yeni bir `String` oluşturur ve
`consumer`'a onu veririz.\
`s`, `consumer` çağrısından sonra da geçerli ve kullanılabilir kalır.

## Bellekte {#in-memory}

Yukarıdaki örnekte bellekte neler olduğuna bakalım.
`let mut s = String::from("hello");` çalıştırıldığında bellek şöyle görünür:

```text
                    s
      +---------+--------+----------+
Stack | pointer | length | capacity | 
      |  |      |   5    |    5     |
      +--|------+--------+----------+
         |
         |
         v
       +---+---+---+---+---+
Heap:  | H | e | l | l | o |
       +---+---+---+---+---+
```

`let t = s.clone()` çalıştırıldığında, verinin bir kopyasını saklamak için heap'te bambaşka yeni bir bölge ayrılır:

```text
                    s                                    t
      +---------+--------+----------+      +---------+--------+----------+
Stack | pointer | length | capacity |      | pointer | length | capacity |
      |  |      |   5    |    5     |      |  |      |   5    |    5     |
      +--|------+--------+----------+      +--|------+--------+----------+
         |                                    |
         |                                    |
         v                                    v
       +---+---+---+---+---+                +---+---+---+---+---+
Heap:  | H | e | l | l | o |                | H | e | l | l | o |
       +---+---+---+---+---+                +---+---+---+---+---+
```

Java gibi bir dilden geliyorsan, `clone`'u bir nesnenin derin kopyasını oluşturmanın bir yolu olarak düşünebilirsin.

## `Clone`'u Implement Etmek {#implementing-clone}

Bir türü `Clone`'lanabilir hale getirmek için o tür için `Clone` trait'ini implement etmemiz gerekir.\
`Clone`'u neredeyse her zaman derive ederek implement edersin:

```rust
#[derive(Clone)]
struct MyType {
    // alanlar
}
```

Derleyici `MyType` için `Clone`'u beklediğin gibi implement eder: `MyType`'ın her alanını tek tek klonlar ve
ardından klonlanmış alanları kullanarak yeni bir `MyType` instance'ı oluşturur.\
`derive` makrolarının ürettiği kodu incelemek için `cargo expand`'i (ya da IDE'ni) kullanabileceğini unutma.
