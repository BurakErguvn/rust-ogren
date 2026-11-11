# Generic'ler Ve Associated Type'lar (İlişkili Türler) {#generics-and-associated-types}

Şimdiye kadar incelediğimiz trait'lerden ikisinin, `From` ve `Deref`'in, tanımına yeniden bakalım:

```rust
pub trait From<T> {
    fn from(value: T) -> Self;
}

pub trait Deref {
    type Target;
    
    fn deref(&self) -> &Self::Target;
}
```

İkisi de tür parametreleri içerir.\
`From` söz konusu olduğunda bu, generic bir parametredir: `T`.\
`Deref` söz konusu olduğunda ise bir associated type'tır: `Target`.

Fark nedir? Neden birini diğerine tercih ederiz?

## En Fazla Bir Implementation {#at-most-one-implementation}

Deref coercion'ın çalışma şekli nedeniyle, belirli bir tür için yalnızca bir "hedef" tür olabilir. Örneğin `String`
yalnızca `str`'ye deref edilebilir.
Mesele belirsizliği önlemektir: bir tür için `Deref`'i birden çok kez implement edebilseydin,
bir `&self` metodu çağırdığında derleyici hangi `Target` türünü seçmeliydi?

`Deref`'in bir associated type olan `Target`'ı kullanmasının nedeni budur.\
Bir associated type, **trait implementation'ı tarafından** benzersiz olarak belirlenir.
`Deref`'i birden fazla kez implement edemeyeceğin için, belirli bir tür için yalnızca bir `Target` belirtebilirsin
ve hiçbir belirsizlik olmaz.

## Generic Trait'ler {#generic-traits}

Öte yandan, **girdi türü `T` farklı olduğu sürece** bir tür için `From`'u birden çok kez implement edebilirsin.
Örneğin, `WrappingU32` için hem `u32` hem de `u16` girdi türlerini kullanarak `From` implement edebilirsin:

```rust
impl From<u32> for WrappingU32 {
    fn from(value: u32) -> Self {
        WrappingU32 { inner: value }
    }
}

impl From<u16> for WrappingU32 {
    fn from(value: u16) -> Self {
        WrappingU32 { inner: value.into() }
    }
}
```

Bu çalışır, çünkü `From<u16>` ve `From<u32>` **farklı trait'ler** olarak kabul edilir.\
Hiçbir belirsizlik yoktur: derleyici, dönüştürülen değerin türüne göre hangi implementation'ın kullanılacağını belirleyebilir.

## Vaka Çalışması: `Add` {#case-study-add}

Kapanış örneği olarak standart kütüphanedeki `Add` trait'ini ele alalım:

```rust
pub trait Add<RHS = Self> {
    type Output;
    
    fn add(self, rhs: RHS) -> Self::Output;
}
```

İki mekanizmayı da kullanır:

- generic bir parametresi vardır: varsayılan olarak `Self` olan `RHS` (right-hand side, sağ taraf)
- bir associated type'ı vardır: toplamanın sonucunun türü olan `Output`

### `RHS` {#rhs}

`RHS`, farklı türlerin birbiriyle toplanabilmesine olanak tanıyan generic bir parametredir.\
Örneğin standart kütüphanede şu iki implementation'ı bulursun:

```rust
impl Add<u32> for u32 {
    type Output = u32;
    
    fn add(self, rhs: u32) -> u32 {
      //                      ^^^
      // Bunun yerine `Self::Output` olarak da yazılabilirdi.
      // Burada belirttiğin tür, hemen yukarıda `Output`'a
      // atadığın türle eşleştiği sürece derleyici
      // umursamaz.
      // [...]
    }
}

impl Add<&u32> for u32 {
    type Output = u32;
    
    fn add(self, rhs: &u32) -> u32 {
        // [...]
    }
}
```

Bu, aşağıdaki kodun derlenmesini sağlar:

```rust
let x = 5u32 + &5u32 + 6u32;
```

çünkü `u32` hem `Add<&u32>`'yi _hem de_ `Add<u32>`'yi implement eder.

### `Output` {#output}

`Output`, toplamanın sonucunun türünü temsil eder.

En başta neden `Output`'a ihtiyacımız var? Çıktı olarak, `Add`'i implement eden tür olan `Self`'i kullanamaz mıyız?
Kullanabilirdik, ama bu trait'in esnekliğini sınırlardı. Örneğin standart kütüphanede
şu implementation'ı bulursun:

```rust
impl Add<&u32> for &u32 {
    type Output = u32;

    fn add(self, rhs: &u32) -> u32 {
        // [...]
    }
}
```

Trait'in implement edildiği tür `&u32`, ama toplamanın sonucu `u32`.\
`add` `Self`'i, yani bu durumda `&u32`'yi döndürmek zorunda olsaydı, bu implementation'ı sağlamak imkânsız[^flexible] olurdu.
`Output`, `std`'nin implement eden türü dönüş türünden ayırmasına olanak tanır ve böylece bu durumu destekler.

Öte yandan, `Output` generic bir parametre olamaz. İşlenenlerin türleri bilindiğinde işlemin çıktı türünün
benzersiz olarak belirlenmesi **şarttır**. Associated type olmasının nedeni budur: implement eden tür ve
generic parametrelerin belirli bir kombinasyonu için yalnızca bir `Output` türü vardır.

## Sonuç {#conclusion}

Özetlersek:

- Tür, belirli bir trait implementation'ı için benzersiz olarak belirlenmek zorundaysa bir **associated type** kullan.
- Aynı tür için, farklı girdi türleriyle trait'in birden fazla implementation'ına izin vermek istiyorsan
  bir **generic parametre** kullan.

[^flexible]: Esneklik nadiren bedavadır: `Output` yüzünden trait tanımı daha karmaşıktır ve implement eden türler ne döndürmek istediklerini
düşünmek zorundadır. Bu trade-off (ödünleşim) yalnızca o esnekliğe gerçekten ihtiyaç varsa haklı çıkar. Kendi trait'lerini
tasarlarken bunu aklında tut.
