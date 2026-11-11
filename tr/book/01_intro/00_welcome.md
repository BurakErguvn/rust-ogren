# Hoş Geldin {#welcome}

**"100 Exercises To Learn Rust"** kursuna hoş geldin!

Bu kurs sana Rust'ın temel kavramlarını her seferinde bir egzersizle öğretecek.\
Rust'ın söz dizimini, tür sistemini, standart kütüphanesini ve ekosistemini öğreneceksin.

Rust hakkında önceden bilgi sahibi olduğunu varsaymıyoruz, ama en az bir başka programlama
dili bildiğini varsayıyoruz.
Sistem programlama ya da bellek yönetimi konusunda önceden bilgi sahibi olduğunu da varsaymıyoruz. Bu
konular kursta ele alınacak.

Başka bir deyişle, sıfırdan başlayacağız!\
Rust bilgini küçük, yönetilebilir adımlarla geliştireceksin.
Kursun sonunda ~100 egzersiz çözmüş olacaksın; bu, küçük ve orta ölçekli Rust projelerinde
rahatça çalışabilmen için yeterli.

## Yöntem {#methodology}

Bu kurs "yaparak öğrenme" ilkesine dayanır.\
Etkileşimli ve uygulamalı olacak şekilde tasarlandı.

[Mainmatter](https://mainmatter.com/rust-consulting/) bu kursu
4 gün boyunca sınıf ortamında verilmek üzere geliştirdi: her katılımcı derslerde kendi hızında
ilerlerken deneyimli bir eğitmen yol gösterir, soruları yanıtlar ve gerektiğinde konuların
derinine iner.\
Eğitmen eşliğindeki bir sonraki oturuma [web sitemizden](https://ti.to/mainmatter/rust-from-scratch-jan-2025) kaydolabilirsin.
Şirketin için özel bir oturum düzenlemek istersen lütfen [bizimle iletişime geç](https://mainmatter.com/contact/).

Kursları kendi başına da takip edebilirsin, ama takılırsan yol boyunca sana yardım edecek bir arkadaş
ya da mentor bulmanı öneririz. Tüm egzersizlerin çözümlerini
[GitHub repo'sunun `solutions` branch'inde](https://github.com/mainmatter/100-exercises-to-learn-rust/tree/solutions)
bulabilirsin.

## Formatlar {#formats}

Kurs materyalini [tarayıcıda](https://rust-exercises.com/100-exercises/) okuyabilir ya da çevrimdışı okumak için [PDF dosyası olarak indirebilirsin](https://rust-exercises.com/100-exercises-to-learn-rust.pdf).\
Kurs materyalinin basılı halini tercih edersen [Amazon'dan ciltsiz baskısını satın al](https://www.amazon.com/dp/B0DJ14KQQG/).

## Yapı {#structure}

Ekranın sol tarafında kursun kısımlara ayrıldığını görebilirsin.
Her kısım, Rust dilinin yeni bir kavramını ya da özelliğini tanıtır.\
Anladığını doğrulayabilmen için her kısım, çözmen gereken bir egzersizle eşleştirilmiştir.

Egzersizleri
[kursa eşlik eden GitHub repo'sunda](https://github.com/mainmatter/100-exercises-to-learn-rust) bulabilirsin.\
Kursa başlamadan önce repo'yu kendi bilgisayarına klonladığından emin ol:

```bash
# If you have an SSH key set up with GitHub
git clone git@github.com:mainmatter/100-exercises-to-learn-rust.git
# Otherwise, use the HTTPS URL:
#   https://github.com/mainmatter/100-exercises-to-learn-rust.git
```

Ayrıca bir branch üzerinde çalışmanı öneririz; böylece ilerlemeni kolayca takip edebilir ve gerekirse
ana repo'daki güncellemeleri çekebilirsin:

```bash
cd 100-exercises-to-learn-rust
git checkout -b my-solutions
```

Tüm egzersizler `exercises` klasöründe bulunur.
Her egzersiz bir Rust paketi olarak yapılandırılmıştır.
Paket; egzersizin kendisini, ne yapman gerektiğine dair talimatları (`src/lib.rs` içinde) ve çözümünü
otomatik olarak doğrulayan bir test suite'i içerir.

### Araçlar {#tools}

Bu kursu takip edebilmek için şunlara ihtiyacın olacak:

- [**Rust**](https://www.rust-lang.org/tools/install).
  Sisteminde `rustup` zaten kuruluysa, en son kararlı sürümü kullandığından emin olmak için `rustup update` komutunu (ya da Rust'ı sistemine nasıl kurduğuna bağlı olarak uygun başka bir komutu) çalıştır.
- _(İsteğe bağlı ama önerilir)_ Rust otomatik tamamlama desteği olan bir IDE.
  Aşağıdakilerden birini öneririz:
  - [RustRover](https://www.jetbrains.com/rust/);
  - [Visual Studio Code](https://code.visualstudio.com) ve [`rust-analyzer`](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer) eklentisi.

### Workshop Runner, `wr` {#workshop-runner-wr}

Çözümlerini doğrulaman için kurs boyunca sana rehberlik edecek bir araç da hazırladık: "workshop runner" ifadesinin kısaltması olan `wr` CLI'ı.
`wr` aracını [web sitesindeki](https://mainmatter.github.io/rust-workshop-runner/) talimatları izleyerek kur.

`wr` kurulduktan sonra yeni bir terminal aç ve repo'nun kök klasörüne git.
Kursa başlamak için `wr` komutunu çalıştır:

```bash
wr
```

`wr`, mevcut egzersizin çözümünü doğrulayacak.\
Mevcut kısmın egzersizini çözmeden bir sonraki kısma geçme.

> Kursta ilerledikçe çözümlerini Git'e commit'lemeni öneririz;
> böylece ilerlemeni kolayca takip edebilir ve gerekirse bilinen bir noktadan "yeniden başlayabilirsin".

Kursun tadını çıkar!

## Yazar {#author}

Bu kurs, [Luca Palmieri](https://www.lpalmieri.com/) tarafından yazıldı; kendisi [Mainmatter](https://mainmatter.com/rust-consulting/)'da
Principal Engineering Consultant olarak çalışıyor.\
Luca 2018'den beri Rust ile çalışıyor; önce TrueLayer'da, ardından AWS'te.\
Luca, Rust ile backend uygulamaları geliştirmeyi öğrenmek için başvurulacak kaynak olan
["Zero to Production in Rust"](https://zero2prod.com) kitabının yazarıdır.\
Ayrıca aralarında
[`cargo-chef`](https://github.com/LukeMathWalker/cargo-chef),
[Pavex](https://pavex.dev) ve [`wiremock`](https://github.com/LukeMathWalker/wiremock-rs)'un da bulunduğu çeşitli açık kaynak Rust projelerinin yazarı ve bakımcısıdır.

## Lisans {#license}

Copyright © 2024- Mainmatter GmbH (https://mainmatter.com),
[PolyForm-Noncommercial-1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0) lisansı altında yayımlanmıştır.
