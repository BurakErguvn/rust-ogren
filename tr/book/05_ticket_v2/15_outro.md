# Toparlarken {#wrapping-up}

Domain modelleme söz konusu olduğunda şeytan ayrıntılarda gizlidir.\
Rust, domain'inin kısıtlamalarını doğrudan tür sisteminde temsil etmene yardımcı olacak geniş bir araç yelpazesi sunar,
ama bunu doğru yapmak ve idiomatik görünen kod yazmak biraz pratik gerektirir.

Bölümü `Ticket` modelimizde son bir iyileştirmeyle kapatalım.\
İlgili kısıtlamaları kapsüllemek için `Ticket`'taki alanların her biri için yeni bir tür tanıtacağız.\
Biri bir `Ticket` alanına her eriştiğinde, geçerli olduğu garanti edilen bir değer geri alır—yani `String` yerine
bir `TicketTitle`. Kodun başka yerlerinde başlığın boş olmasından endişelenmek zorunda kalmazlar:
bir `TicketTitle`'a sahip oldukları sürece, onun **oluşturulma biçimi gereği** geçerli olduğunu bilirler.

Bu, kodunu daha güvenli ve daha ifade gücü yüksek hale getirmek için Rust'ın tür sistemini nasıl kullanabileceğinin
yalnızca bir örneğidir.

## İleri Okuma {#further-reading}

- [Parse et, doğrulama yapma](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/)
- [Domain invariant'larını (değişmezlerini) garanti etmek için türleri kullanmak](https://www.lpalmieri.com/posts/2020-12-11-zero-to-production-6-domain-modelling/)
