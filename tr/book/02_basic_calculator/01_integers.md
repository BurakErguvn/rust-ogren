# Türler, 1. Kısım {#types-part-1}

["Syntax" kısmında](../01_intro/01_syntax.md) `compute`'un girdi parametreleri `u32` türündeydi.\
Bunun ne _anlama geldiğini_ açalım.

## İlkel Türler {#primitive-types}

`u32`, Rust'ın **ilkel türlerinden** biridir. İlkel türler bir dilin en temel yapı taşlarıdır.
Dilin kendisine yerleşiktirler—yani başka türler cinsinden tanımlanmazlar.

Bu ilkel türleri birleştirerek daha karmaşık türler oluşturabilirsin. Bunun nasıl yapıldığını yakında göreceğiz.

## Tam Sayılar {#integers}

Daha açık söylemek gerekirse `u32`, **işaretsiz 32-bit bir tam sayıdır**.

Tam sayı, kesirli bir kısım olmadan yazılabilen bir sayıdır. Ör. `1` bir tam sayıdır, `1.2` ise değildir.

### İşaretli Ve İşaretsiz {#signed-vs-unsigned}

Bir tam sayı **işaretli** ya da **işaretsiz** olabilir.\
İşaretsiz bir tam sayı yalnızca negatif olmayan sayıları (yani `0` ya da daha büyük) temsil edebilir.
İşaretli bir tam sayı ise hem pozitif hem negatif sayıları (ör. `-1`, `12` vb.) temsil edebilir.

`u32`'deki `u`, **unsigned** (işaretsiz) anlamına gelir.\
İşaretli tam sayı için eşdeğer tür `i32`'dir; buradaki `i`, integer (tam sayı) anlamına gelir (yani pozitif ya da
negatif, herhangi bir tam sayı).

### Bit Genişliği {#bit-width}

`u32`'deki `32`, sayıyı bellekte temsil etmek için kullanılan **bit sayısını[^bit]** ifade eder.\
Bit sayısı ne kadar fazlaysa, temsil edilebilecek sayıların aralığı da o kadar geniş olur.

Rust, tam sayılar için birden fazla bit genişliğini destekler: `8`, `16`, `32`, `64`, `128`.

32 bit ile `u32`, `0`'dan `2^32 - 1`'e (diğer adıyla [`u32::MAX`](https://doc.rust-lang.org/std/primitive.u32.html#associatedconstant.MAX)) kadar olan sayıları temsil edebilir.\
Aynı sayıda bit ile işaretli bir tam sayı (`i32`), `-2^31`'den `2^31 - 1`'e kadar olan sayıları temsil edebilir
(yani [`i32::MIN`](https://doc.rust-lang.org/std/primitive.i32.html#associatedconstant.MIN)'den
[`i32::MAX`](https://doc.rust-lang.org/std/primitive.i32.html#associatedconstant.MAX)'a kadar).\
`i32` için maksimum değer, `u32` için maksimum değerden küçüktür, çünkü bir bit sayının işaretini temsil etmek için
kullanılır. İşaretli tam sayıların bellekte nasıl temsil edildiğine dair daha fazla ayrıntı için [ikiye tümleyen](https://en.wikipedia.org/wiki/Two%27s_complement)
gösterimine göz at.

### Özet {#summary}

İki boyutu (işaretli/işaretsiz ve bit genişliği) birleştirince şu tam sayı türlerini elde ederiz:

| Bit genişliği | İşaretli | İşaretsiz |
| ------------- | -------- | --------- |
| 8-bit         | `i8`     | `u8`      |
| 16-bit        | `i16`    | `u16`     |
| 32-bit        | `i32`    | `u32`     |
| 64-bit        | `i64`    | `u64`     |
| 128-bit       | `i128`   | `u128`    |

## Literal'ler {#literals}

**Literal**, kaynak kodda sabit bir değeri temsil etmeye yarayan bir gösterimdir.\
Örneğin `42`, kırk iki sayısı için bir Rust literal'idir.

### Literal'ler İçin Tür Belirtimleri {#type-annotations-for-literals}

Ama Rust'taki tüm değerlerin bir türü vardır, peki... `42`'nin türü ne?

Rust derleyicisi bir literal'in türünü, nasıl kullanıldığına bakarak çıkarsamaya çalışır.\
Hiçbir bağlam sağlamazsan, derleyici tam sayı literal'leri için varsayılan olarak `i32`'yi kullanır.\
Farklı bir tür kullanmak istersen, istediğin tam sayı türünü sonek olarak ekleyebilirsin—ör. `2u64`, türü açıkça
`u64` olarak belirtilmiş bir 2'dir.

### Literal'lerde Alt Çizgiler {#underscores-in-literals}

Büyük sayıların okunabilirliğini artırmak için alt çizgi `_` kullanabilirsin.\
Örneğin `1_000_000`, `1000000` ile aynıdır.

## Aritmetik Operatörler {#arithmetic-operators}

Rust, tam sayılar için şu aritmetik operatörleri[^traits] destekler:

- `+` toplama için
- `-` çıkarma için
- `*` çarpma için
- `/` bölme için
- `%` kalan için

Bu operatörlerin öncelik ve birleşme kuralları matematikteki ile aynıdır.\
Varsayılan önceliği override etmek için parantez kullanabilirsin. Ör. `2 * (3 + 4)`.

> ⚠️ **Uyarı**
>
> Bölme operatörü `/`, tam sayı türleriyle kullanıldığında tam sayı bölmesi yapar.
> Yani sonuç sıfıra doğru kırpılır. Örneğin `5 / 2`, `2.5` değil `2`'dir.

## Otomatik Type Coercion (Örtük Tür Dönüşümü) Yok {#no-automatic-type-coercion}

Bir önceki egzersizde konuştuğumuz gibi, Rust statik türlü bir dildir.\
Özellikle type coercion konusunda Rust oldukça katıdır. Dönüşüm kayıpsız olsa bile, bir değeri bir türden başka bir türe
otomatik olarak dönüştürmez[^coercion].
Bunu açıkça yapman gerekir.

Örneğin, tüm `u8` değerleri geçerli `u32` değerleri olsa bile, `u32` türündeki bir değişkene bir `u8` değeri
atayamazsın:

```rust
let b: u8 = 100;
let a: u32 = b;
```

Bir derleme hatası verir:

```text
error[E0308]: mismatched types
  |
3 |     let a: u32 = b;
  |            ---   ^ expected `u32`, found `u8`
  |            |
  |            expected due to this
  |
```

Türler arasında nasıl dönüşüm yapılacağını [bu kursun ilerleyen kısımlarında](../04_traits/09_from.md) göreceğiz.

## İleri Okuma {#further-reading}

- Resmi Rust kitabındaki [tam sayı türleri kısmı](https://doc.rust-lang.org/book/ch03-02-data-types.html#integer-types)

[^bit]: Bit, bir bilgisayardaki en küçük veri birimidir. Yalnızca iki değer alabilir: `0` ya da `1`.

[^traits]: Rust özel operatörler tanımlamana izin vermez, ama yerleşik operatörlerin nasıl davranacağının kontrolünü
sana bırakır.
Trait'leri ele aldıktan sonra, [kursun ilerleyen kısımlarında](../04_traits/03_operator_overloading.md) operator overloading (operatör aşırı yükleme) konusunu konuşacağız.

[^coercion]: Bu kuralın, çoğunlukla referanslar, smart pointer'lar (akıllı pointer'lar) ve ergonomiyle ilgili bazı istisnaları vardır. Bunları
[daha sonra](../04_traits/07_deref.md) ele alacağız.
O zamana kadar "tüm dönüşümler açıktır" zihinsel modeli işini fazlasıyla görecektir.
