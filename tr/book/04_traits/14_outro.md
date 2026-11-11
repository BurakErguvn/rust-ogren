# Toparlarken {#wrapping-up}

Bu bölümde epeyce farklı trait ele aldık—ve daha yüzeyi yeni kazıdık!
Hatırlaman gereken çok şey varmış gibi gelebilir, ama endişelenme: Rust kodu yazarken bu trait'lerle
o kadar sık karşılaşacaksın ki kısa sürede senin için doğal hale gelecekler.

## Kapanış Düşünceleri {#closing-thoughts}

Trait'ler güçlüdür, ama onları aşırı kullanma.\
Akılda tutman gereken birkaç ilke:

- Her zaman tek bir türle çağrılıyorsa bir fonksiyonu generic yapma. Bu, kod tabanına
  dolaylılık katar ve onu anlamayı ve bakımını yapmayı zorlaştırır.
- Yalnızca bir implementation'ın varsa trait oluşturma. Bu, trait'e ihtiyaç olmadığının bir işaretidir.
- Mantıklı olduğu her durumda türlerin için standart trait'leri (`Debug`, `PartialEq` vb.) implement et.
  Bu, türlerini daha idiomatik ve kullanımı daha kolay hale getirir; standart kütüphanenin ve ekosistem
  crate'lerinin sağladığı pek çok işlevselliğin kilidini açar.
- Üçüncü taraf crate'lerin trait'lerini, onların ekosisteminde açtıkları işlevselliğe ihtiyacın varsa implement et.
- Kodu yalnızca testlerinde mock kullanabilmek için generic yapmaktan kaçın. Bu yaklaşımın bakım maliyeti
  yüksek olabilir ve çoğu zaman farklı bir test stratejisi kullanmak daha iyidir. Yüksek doğruluklu testler hakkında
  ayrıntılar için [testing masterclass'ına](https://github.com/mainmatter/rust-advanced-testing-workshop)
  göz at.

## Bilgini Sına {#testing-your-knowledge}

Devam etmeden önce, öğrendiklerimizi pekiştirmek için son bir egzersizden geçelim.
Bu sefer çok az yönlendirme olacak—sana yol gösterecek olan yalnızca egzersiz açıklaması ve testler.
