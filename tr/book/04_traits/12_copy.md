# Değerleri Kopyalamak, 2. Kısım {#copying-values-pt-2}

Öncekiyle aynı örneği ele alalım, ama küçük bir farkla: tür olarak `String` yerine `u32` kullanalım.

```rust
fn consumer(s: u32) { /* */ }

fn example() {
     let s: u32 = 5;
     consumer(s);
     let t = s + 1;
}
```

Hatasız derlenecek! Burada neler oluyor? `String` ile `u32` arasında, ikincisinin `.clone()` olmadan
çalışmasını sağlayan fark nedir?

## `Copy` {#copy}

`Copy`, Rust'ın standart kütüphanesinde tanımlanmış başka bir trait'tir:

```rust
pub trait Copy: Clone { }
```

Tıpkı `Sized` gibi bir marker trait'tir.

Bir tür `Copy` implement ediyorsa, türün yeni bir instance'ını (örneğini) oluşturmak için `.clone()` çağırmaya gerek yoktur:
Rust bunu senin için **örtük olarak** yapar.\
`u32`, `Copy` implement eden bir tür örneğidir; yukarıdaki örneğin hatasız derlenmesinin nedeni de budur:
`consumer(s)` çağrıldığında Rust, `s`'in **bit düzeyinde bir kopyasını** oluşturarak yeni bir `u32` instance'ı yaratır
ve ardından bu yeni instance'ı `consumer`'a iletir. Bunların hepsi perde arkasında, senin hiçbir şey yapmana gerek kalmadan olur.

## Neler `Copy` Olabilir? {#what-can-be-copy}

`Copy`, "otomatik klonlama" ile eşdeğer değildir, ama onu da kapsar.\
Türlerin `Copy` implement etmesine izin verilmesi için birkaç gereksinimi karşılaması gerekir.

Her şeyden önce, tür `Clone` implement etmelidir, çünkü `Copy`, `Clone`'un bir subtrait'idir.
Bu mantıklıdır: Rust bir türün yeni bir instance'ını _örtük olarak_ oluşturabiliyorsa,
`.clone()` çağırarak _açıkça_ da yeni bir instance oluşturabilmelidir.

Ama hepsi bu değil. Birkaç koşulun daha sağlanması gerekir:

1. Tür, bellekte kapladığı `std::mem::size_of` byte'ın ötesinde herhangi bir _ek_ kaynak (örneğin heap belleği, dosya handle'ları vb.)
   yönetmez.
2. Tür bir mutable (değiştirilebilir) referans (`&mut T`) değildir.

Her iki koşul da sağlanıyorsa Rust, orijinal instance'ın **bit düzeyinde bir kopyasını** oluşturarak türün yeni bir instance'ını
güvenle oluşturabilir—buna genellikle, bit düzeyinde kopyalamayı yapan C standart kütüphanesi fonksiyonunun adından
esinlenerek `memcpy` işlemi denir.

### Vaka Çalışması 1: `String` {#case-study-1-string}

`String`, `Copy` implement etmeyen bir türdür.\
Neden? Çünkü ek bir kaynak yönetir: string'in verisini saklayan, heap'te ayrılmış bellek buffer'ı.

Rust'ın `String`'in `Copy` implement etmesine izin verdiğini hayal edelim.\
O zaman, orijinal instance'ın bit düzeyinde bir kopyası oluşturularak yeni bir `String` instance'ı yaratıldığında, hem orijinal
hem de yeni instance aynı bellek buffer'ını gösterirdi:

```text
              s                                 copied_s
+---------+--------+----------+      +---------+--------+----------+
| pointer | length | capacity |      | pointer | length | capacity |
|  |      |   5    |    5     |      |  |      |   5    |    5     |
+--|------+--------+----------+      +--|------+--------+----------+
   |                                    |
   |                                    |
   v                                    |
 +---+---+---+---+---+                  |
 | H | e | l | l | o |                  |
 +---+---+---+---+---+                  |
   ^                                    |
   |                                    |
   +------------------------------------+
```

Bu kötü!
Her iki `String` instance'ı da kapsam dışına çıktıklarında bellek buffer'ını serbest bırakmaya çalışırdı
ve bu bir double-free hatasına yol açardı.
Ayrıca aynı bellek buffer'ını gösteren iki ayrı `&mut String` referansı oluşturabilir
ve Rust'ın borrowing (ödünç alma) kurallarını ihlal edebilirdin.

### Vaka Çalışması 2: `u32` {#case-study-2-u32}

`u32`, `Copy` implement eder. Aslında tüm tam sayı türleri implement eder.\
Bir tam sayı, sayıyı bellekte temsil eden byte'lardan "ibarettir". Fazlası yok!
O byte'ları kopyalarsan, tamamen geçerli başka bir tam sayı instance'ı elde edersin.
Kötü bir şey olamaz, bu yüzden Rust buna izin verir.

### Vaka Çalışması 3: `&mut u32` {#case-study-3-mut-u32}

Ownership'i (sahipliği) ve mutable ödünç almayı tanıttığımızda bir kuralı oldukça net bir şekilde belirtmiştik: herhangi
bir anda bir değerin yalnızca _tek_ bir mutable ödünç alması olabilir.\
`u32` `Copy` implement ettiği halde `&mut u32`'nin implement etmemesinin nedeni budur.

`&mut u32` `Copy` implement etseydi, aynı değere birden fazla mutable referans oluşturup
onu aynı anda birden fazla yerde değiştirebilirdin.
Bu, Rust'ın borrowing kurallarının ihlali olurdu!
Buradan, `T` ne olursa olsun `&mut T`'nin asla `Copy` implement etmediği sonucu çıkar.

## `Copy`'yi Implement Etmek {#implementing-copy}

Çoğu durumda `Copy`'yi elle implement etmen gerekmez.
Şöyle derive etmen yeterlidir:

```rust
#[derive(Copy, Clone)]
struct MyStruct {
    field: u32,
}
```
