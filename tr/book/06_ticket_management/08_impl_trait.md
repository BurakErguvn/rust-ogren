# `impl Trait` {#impl-trait}

`TicketStore::to_dos` bir `Vec<&Ticket>` döndürür.\
Bu imza, `to_dos` her çağrıldığında yeni bir heap allocation'ı (bellek ayırma) ortaya çıkarır; çağıranın sonuçla ne
yapması gerektiğine bağlı olarak bu gereksiz olabilir.
`to_dos` bir `Vec` yerine bir iterator döndürse daha iyi olurdu; böylece sonuçları bir `Vec`'te toplayıp toplamamaya
ya da yalnızca üzerlerinde gezinmeye çağıran karar verebilirdi.

Ama bu biraz zor!
Aşağıda implement edildiği haliyle `to_dos`'un dönüş türü nedir?

```rust
impl TicketStore {
    pub fn to_dos(&self) -> ??? {
        self.tickets.iter().filter(|t| t.status == Status::ToDo)
    }
}
```

## Adlandırılamayan Türler {#unnameable-types}

`filter` metodu, aşağıdaki tanıma sahip olan `std::iter::Filter`'ın bir instance'ını (örneğini) döndürür:

```rust
pub struct Filter<I, P> { /* alanlar gösterilmiyor */ }
```

burada `I` üzerinde filtreleme yapılan iterator'ın türü, `P` ise elemanları filtrelemek için kullanılan predicate'tir (yüklem).\
Bu durumda `I`'ın `std::slice::Iter<'_, Ticket>` olduğunu biliyoruz, peki ya `P`?\
`P` bir closure, yani bir **anonim fonksiyondur**. Adından da anlaşılacağı gibi closure'ların bir adı yoktur,
dolayısıyla onları kodumuzda yazamayız.

Rust'ın bunun için bir çözümü var: **impl Trait**.

## `impl Trait` {#impl-trait-1}

`impl Trait`, bir türü adını belirtmeden döndürmene olanak tanıyan bir özelliktir.
Yalnızca türün hangi trait'i ya da trait'leri implement ettiğini bildirirsin, gerisini Rust halleder.

Bu durumda, `Ticket`'lara ait referanslardan oluşan bir iterator döndürmek istiyoruz:

```rust
impl TicketStore {
    pub fn to_dos(&self) -> impl Iterator<Item = &Ticket> {
        self.tickets.iter().filter(|t| t.status == Status::ToDo)
    }
}
```

Hepsi bu!

## Generic Mi? {#generic}

Dönüş konumundaki `impl Trait`, bir generic parametre **değildir**.

Generic'ler, fonksiyonu çağıran tarafından doldurulan türler için yer tutuculardır.
Generic parametreli bir fonksiyon **polimorfiktir**: farklı türlerle çağrılabilir ve derleyici her tür için
farklı bir implementation üretir.

`impl Trait` için durum böyle değildir.
`impl Trait` kullanan bir fonksiyonun dönüş türü derleme zamanında **sabittir** ve derleyici onun için
tek bir implementation üretir.
Bu yüzden `impl Trait`'e **opak dönüş türü** de denir: çağıran, dönüş değerinin tam türünü bilmez,
yalnızca belirtilen trait'i ya da trait'leri implement ettiğini bilir. Ama derleyici tam türü bilir; işin içinde polimorfizm yoktur.

## RPIT {#rpit}

Rust hakkında RFC'ler ya da derinlemesine yazılar okursan **RPIT** kısaltmasıyla karşılaşabilirsin.\
Bu kısaltma **"Return Position Impl Trait"** ifadesinden gelir ve `impl Trait`'in dönüş konumunda kullanılmasını ifade eder.
