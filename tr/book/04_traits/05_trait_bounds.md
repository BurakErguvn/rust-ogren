# Trait Bound'lar {#trait-bounds}

Şimdiye kadar trait'ler için iki kullanım alanı gördük:

- "Yerleşik" davranışın kilidini açmak (örneğin operator overloading, yani operatör aşırı yükleme)
- Var olan türlere yeni davranış eklemek (yani extension trait'ler)

Üçüncü bir kullanım alanı daha var: **generic programlama**.

## Sorun {#the-problem}

Şimdiye kadar tüm fonksiyonlarımız ve metotlarımız **somut türlerle** çalıştı.\
Somut türler üzerinde çalışan kodu yazmak ve anlamak genellikle kolaydır. Ama bu kodun
yeniden kullanılabilirliği de sınırlıdır.\
Örneğin, bir tam sayı çift ise `true` döndüren bir fonksiyon yazmak istediğimizi düşünelim.
Somut türlerle çalışırsak, desteklemek istediğimiz her tam sayı türü için ayrı bir fonksiyon
yazmamız gerekir:

```rust
fn is_even_i32(n: i32) -> bool {
    n % 2 == 0
}

fn is_even_i64(n: i64) -> bool {
    n % 2 == 0
}

// Vb.
```

Alternatif olarak, tek bir extension trait yazıp ardından her tam sayı türü için farklı implementation'lar yazabiliriz:

```rust
trait IsEven {
    fn is_even(&self) -> bool;
}

impl IsEven for i32 {
    fn is_even(&self) -> bool {
        self % 2 == 0
    }
}

impl IsEven for i64 {
    fn is_even(&self) -> bool {
        self % 2 == 0
    }
}

// Vb.
```

Kod tekrarı hâlâ ortada.

## Generic Programlama {#generic-programming}

**Generic**'ler kullanarak daha iyisini yapabiliriz.\
Generic'ler, somut bir tür yerine bir **tür parametresiyle** çalışan kod yazmamızı sağlar:

```rust
fn print_if_even<T>(n: T)
where
    T: IsEven + Debug
{
    if n.is_even() {
        println!("{n:?} is even");
    }
}
```

`print_if_even` bir **generic fonksiyondur**.\
Belirli bir girdi türüne bağlı değildir. Bunun yerine, şu koşulları sağlayan herhangi bir `T` türüyle çalışır:

- `IsEven` trait'ini implement eder.
- `Debug` trait'ini implement eder.

Bu sözleşme bir **trait bound** ile ifade edilir: `T: IsEven + Debug`.\
`+` sembolü, `T`'nin birden fazla trait'i implement etmesini şart koşmak için kullanılır. `T: IsEven + Debug`,
"`T`'nin `IsEven` **ve** `Debug` implement ettiği durumda" ifadesine eşdeğerdir.

## Trait Bound'lar {#trait-bounds-1}

Trait bound'lar `print_if_even` içinde ne işe yarar?\
Bunu öğrenmek için onları kaldırmayı deneyelim:

```rust
fn print_if_even<T>(n: T) {
    if n.is_even() {
        println!("{n:?} is even");
    }
}
```

Bu kod derlenmez:

```text
error[E0599]: no method named `is_even` found for type parameter `T` 
              in the current scope
 --> src/lib.rs:2:10
  |
1 | fn print_if_even<T>(n: T) {
  |                  - method `is_even` not found 
  |                    for this type parameter
2 |     if n.is_even() {
  |          ^^^^^^^ method not found in `T`

error[E0277]: `T` doesn't implement `Debug`
 --> src/lib.rs:3:19
  |
3 |         println!("{n:?} is even");
  |                   ^^^^^ 
  |   `T` cannot be formatted using `{:?}` because 
  |         it doesn't implement `Debug`
  |
help: consider restricting type parameter `T`
  |
1 | fn print_if_even<T: std::fmt::Debug>(n: T) {
  |                   +++++++++++++++++
```

Trait bound'lar olmadan derleyici `T`'nin **neler yapabileceğini** bilmez.\
`T`'nin bir `is_even` metodu olduğunu bilmez ve `T`'yi yazdırmak için nasıl biçimlendireceğini de bilmez.
Derleyicinin bakış açısından, çıplak bir `T`'nin hiçbir davranışı yoktur.\
Trait bound'lar, fonksiyon gövdesinin gerektirdiği davranışın mevcut olmasını garanti ederek
kullanılabilecek türlerin kümesini kısıtlar.

## Söz Dizimi: Trait Bound'ları Satır İçine Yazmak {#syntax-inlining-trait-bounds}

Yukarıdaki tüm örneklerde trait bound'ları belirtmek için bir **`where` bloğu** kullanıldı:

```rust
fn print_if_even<T>(n: T)
where
    T: IsEven + Debug
//  ^^^^^^^^^^^^^^^^^
//  Bu bir `where` bloğudur
{
    // [...]
}
```

Trait bound'lar basitse, onları doğrudan tür parametresinin yanına **satır içi** yazabilirsin:

```rust
fn print_if_even<T: IsEven + Debug>(n: T) {
    //           ^^^^^^^^^^^^^^^^^
    //           Bu, satır içi bir trait bound'dur
    // [...]
}
```

## Söz Dizimi: Anlamlı İsimler {#syntax-meaningful-names}

Yukarıdaki örneklerde tür parametresinin adı olarak `T` kullandık. Bir fonksiyonun yalnızca tek bir
tür parametresi olduğunda bu yaygın bir gelenektir.\
Yine de daha anlamlı bir isim kullanmanı hiçbir şey engellemez:

```rust
fn print_if_even<Number: IsEven + Debug>(n: Number) {
    // [...]
}
```

Birden fazla tür parametresi söz konusu olduğunda ya da `T` ismi türün fonksiyondaki rolü hakkında
yeterli bilgi vermediğinde anlamlı isimler kullanmak aslında **tercih edilir**.
Tür parametrelerini adlandırırken, tıpkı değişkenlerde ya da fonksiyon parametrelerinde yapacağın gibi açıklığı ve okunabilirliği en üst düzeye çıkar.
Yine de Rust'ın geleneklerine uy: [tür parametresi adları için upper camel case kullan](https://rust-lang.github.io/api-guidelines/naming.html#casing-conforms-to-rfc-430-c-case).

## Fonksiyon İmzası Kraldır {#the-function-signature-is-king}

Trait bound'lara neden ihtiyaç duyduğumuzu merak edebilirsin. Derleyici gerekli trait'leri fonksiyonun gövdesinden çıkarsayamaz mı?\
Çıkarsayabilirdi, ama çıkarsamaz.\
Gerekçe, [fonksiyon parametrelerindeki açık tür belirtimleri](../02_basic_calculator/02_variables.md#function-arguments-are-variables) ile aynıdır:
her fonksiyon imzası, çağıran ile çağrılan arasında bir sözleşmedir ve koşulların açıkça belirtilmesi gerekir.
Bu; daha iyi hata mesajları, daha iyi dokümantasyon, sürümler arasında daha az istenmeyen kırılma
ve daha kısa derleme süreleri sağlar.
