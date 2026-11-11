# Trait'ler {#traits}

Önceki bölümde Rust'ın tür ve ownership (sahiplik) sisteminin temellerini ele aldık.\
Şimdi daha derine inme zamanı: Rust'ın arayüzlere yaklaşımı olan **trait**'leri keşfedeceğiz.

Trait'leri öğrendiğinde, izlerini her yerde görmeye başlayacaksın.\
Aslında trait'leri önceki bölüm boyunca iş başında zaten gördün; örneğin `.into()` çağrıları ve
`==` ile `+` gibi operatörler.

Trait'lerin bir kavram olarak kendisinin yanı sıra, Rust'ın standart kütüphanesinde tanımlanmış temel trait'lerden bazılarını da ele alacağız:

- Operatör trait'leri (örneğin `Add`, `Sub`, `PartialEq` vb.)
- Başarısız olamayan dönüşümler için `From` ve `Into`
- Değerleri kopyalamak için `Clone` ve `Copy`
- `Deref` ve deref coercion
- Boyutu bilinen türleri işaretlemek için `Sized`
- Özel temizlik mantığı için `Drop`

Dönüşümlerden bahsedeceğimiz için, bu fırsattan yararlanıp önceki bölümden kalan bazı "bilgi boşluklarını"
da dolduracağız—örneğin `"A title"` tam olarak nedir? Slice'lar (dilimler) hakkında da daha fazlasını öğrenme zamanı!
