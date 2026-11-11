# Ticket Yönetimi {#ticket-management}

Önceki bölümde `Ticket`'ı yalıtılmış bir ortamda modelledik: alanlarını ve kısıtlarını tanımladık, bunları
Rust'ta en iyi nasıl temsil edeceğimizi öğrendik, ama `Ticket`'ın daha büyük bir sisteme nasıl oturduğunu düşünmedik.
Bu bölümde `Ticket` etrafında sade bir iş akışı kuracak, ticket'ları saklamak ve geri almak için (oldukça basit)
bir yönetim sistemi ortaya koyacağız.

Bu görev bize yeni Rust kavramlarını keşfetme fırsatı verecek; örneğin:

- Stack'te ayrılmış diziler
- Büyüyebilen bir dizi türü olan `Vec`
- Koleksiyonlar üzerinde gezinmek için `Iterator` ve `IntoIterator`
- Bir koleksiyonun parçalarıyla çalışmak için slice'lar (dilimler) (`&[T]`)
- Referansların ne kadar süre geçerli olduğunu tanımlamak için lifetime'lar (yaşam süreleri)
- İki anahtar-değer veri yapısı: `HashMap` ve `BTreeMap`
- Bir `HashMap`'teki anahtarları karşılaştırmak için `Eq` ve `Hash`
- Bir `BTreeMap` ile çalışmak için `Ord` ve `PartialOrd`
- Bir koleksiyondaki elemanlara erişmek için `Index` ve `IndexMut`
