# Overflow (Taşma) {#overflow}

Bir sayının faktöriyeli oldukça hızlı büyür.\
Örneğin 20'nin faktöriyeli 2,432,902,008,176,640,000 eder. Bu sayı, 32-bit bir tam sayının maksimum değeri olan
2,147,483,647'yi şimdiden aşıyor.

Bir aritmetik işlemin sonucu, belirli bir tam sayı türünün maksimum değerinden büyük olduğunda
**tam sayı overflow'undan** söz ederiz.

Tam sayı overflow'ları bir sorundur, çünkü aritmetik işlemlerin sözleşmesini ihlal ederler.\
Belirli bir türdeki iki tam sayı arasındaki aritmetik işlemin sonucu, aynı türde başka bir tam sayı olmalıdır.
Ama _matematiksel olarak doğru sonuç_ o tam sayı türüne sığmıyor!

> Sonuç, belirli bir tam sayı türünün minimum değerinden küçükse, bu olaya **tam sayı
> underflow'u** (alttan taşma) deriz.\
> Kısa tutmak için bu kısmın geri kalanında yalnızca tam sayı overflow'larından bahsedeceğiz, ama
> söylediğimiz her şeyin tam sayı underflow'ları için de geçerli olduğunu aklında tut.
>
> ["Variables" kısmında](02_variables.md) yazdığın `speed` fonksiyonu bazı girdi
> kombinasyonlarında underflow ediyordu.
> Ör. `end`, `start`'tan küçükse `end - start`, `u32` türünde underflow eder, çünkü sonucun negatif olması
> gerekir ama `u32` negatif sayıları temsil edemez.

## Otomatik Yükseltme Yok {#no-automatic-promotion}

Olası bir yaklaşım, sonucu otomatik olarak daha büyük bir tam sayı türüne yükseltmek olabilirdi.
Ör. iki `u8` tam sayısını topluyorsan ve sonuç 256 ise (`u8::MAX + 1`), Rust sonucu, 256'yı tutabilecek kadar büyük
olan bir sonraki tam sayı türü olan `u16` olarak yorumlamayı seçebilirdi.

Ama daha önce konuştuğumuz gibi, Rust tür dönüşümleri konusunda oldukça titizdir. Otomatik tam sayı yükseltme,
Rust'ın tam sayı overflow'u sorununa getirdiği çözüm değildir.

## Alternatifler {#alternatives}

Otomatik yükseltmeyi elediğimize göre, bir tam sayı overflow'u gerçekleştiğinde ne yapabiliriz?\
İş iki farklı yaklaşıma varıyor:

- İşlemi reddetmek
- Beklenen tam sayı türüne sığan "makul" bir sonuç bulmak

### İşlemi Reddetmek {#reject-the-operation}

Bu en temkinli yaklaşımdır: bir tam sayı overflow'u gerçekleştiğinde programı durdururuz.\
Bu, ["Panics" kısmında](04_panics.md) zaten gördüğümüz mekanizma olan panic aracılığıyla yapılır.

### "Makul" Bir Sonuç Bulmak {#come-up-with-a-sensible-result}

Bir aritmetik işlemin sonucu belirli bir tam sayı türünün maksimum değerinden büyük olduğunda,
**wrap around** (başa sarma) yapmayı seçebilirsin.\
Belirli bir tam sayı türünün tüm olası değerlerini bir çember olarak düşünürsen, wrap around, maksimum değere
ulaştığında minimum değerden yeniden başlamak demektir.

Örneğin 1 ile 255 (=`u8::MAX`) arasında bir **wrapping toplama** yaparsan sonuç 0 (=`u8::MIN`) olur.
İşaretli tam sayılarla çalışıyorsan da aynı ilke geçerlidir. Ör. 127'ye (=`i8::MAX`) wrapping ile 1 eklemek
sana -128 (=`i8::MIN`) verir.

## `overflow-checks` {#overflow-checks}

Rust, bir tam sayı overflow'u gerçekleştiğinde hangi yaklaşımın kullanılacağını sana, yani geliştiriciye bırakır.
Bu davranış `overflow-checks` profil ayarıyla kontrol edilir.

`overflow-checks` `true` olarak ayarlanırsa, bir tam sayı işlemi overflow ettiğinde Rust **çalışma zamanında panic'ler**.
`overflow-checks` `false` olarak ayarlanırsa, bir tam sayı işlemi overflow ettiğinde Rust **wrap around yapar**.

Merak ediyor olabilirsin—profil ayarı nedir? Hadi buna bakalım!

## Profiller {#profiles}

[**Profil**](https://doc.rust-lang.org/cargo/reference/profiles.html), Rust kodunun derlenme biçimini özelleştirmek
için kullanılabilecek bir yapılandırma seçenekleri kümesidir.

Cargo 4 yerleşik profil sunar: `dev`, `release`, `test` ve `bench`.\
`dev` profili, `cargo build`, `cargo run` ya da `cargo test` komutlarını her çalıştırdığında kullanılır. Yerel
geliştirmeye yöneliktir,
bu yüzden daha hızlı derleme süreleri ve daha iyi bir hata ayıklama deneyimi uğruna çalışma zamanı performansından ödün verir.\
`release` profili ise çalışma zamanı performansı için optimize edilmiştir, ama daha uzun derleme sürelerine yol açar. Bunu
`--release` bayrağıyla açıkça istemen gerekir—ör. `cargo build --release` ya da `cargo run --release`.
Kalan iki profil bunların üzerine kuruludur: `test` (`cargo test` tarafından kullanılır) `dev` ayarlarını devralırken,
`bench` (`cargo bench` tarafından kullanılır) `release` ayarlarını devralır. Pratikte zamanının çoğunu `dev`'de geçirecek, performans önemli olduğunda `release`'e geçeceksin.

> "Projeni release modunda derledin mi?" sorusu, Rust topluluğunda neredeyse bir internet geyiği haline gelmiştir.\
> Bu soru, Rust'a aşina olmayan ve projelerini release modunda derlemediklerini fark etmeden önce
> sosyal medyada (ör. Reddit, Twitter) Rust'ın performansından şikâyet eden geliştiricilere
> gönderme yapar.

Ayrıca özel profiller tanımlayabilir ya da yerleşik profilleri özelleştirebilirsin.

### `overflow-check` {#overflow-check}

Varsayılan olarak `overflow-checks` şu şekilde ayarlıdır:

- `dev` profili için `true`
- `release` profili için `false`

Bu, iki profilin hedefleriyle uyumludur.\
`dev` yerel geliştirmeye yöneliktir, bu yüzden olası sorunları olabildiğince erken ortaya çıkarmak için panic'ler.\
`release` ise çalışma zamanı performansı için ayarlanmıştır: overflow kontrolü programı yavaşlatacağından wrap around
yapmayı tercih eder.

Öte yandan, iki profil için farklı davranışlara sahip olmak sinsi bug'lara yol açabilir.\
Önerimiz, `overflow-checks`'i her iki profil için de etkinleştirmendir: sessizce yanlış sonuçlar üretmektense çökmek
daha iyidir. Çalışma zamanı performansına etkisi çoğu durumda göz ardı edilebilir düzeydedir; performansın kritik olduğu bir
uygulama üzerinde çalışıyorsan, bunu göze alıp alamayacağına karar vermek için benchmark'lar çalıştırabilirsin.

## İleri Okuma {#further-reading}

- Rust'ta tam sayı overflow'u üzerine derinlemesine bir tartışma için ["Myths and legends about integer overflow in Rust"](https://huonw.github.io/blog/2016/04/myths-and-legends-about-integer-overflow-in-rust/)
  yazısına göz at.
