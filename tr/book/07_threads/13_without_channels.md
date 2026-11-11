# Tasarım Değerlendirmesi {#design-review}

Bir an durup geride bıraktığımız yolculuğu gözden geçirelim.

## Kanal Üzerinden Sıraya Sokmayla Kilitsiz Tasarım {#lockless-with-channel-serialization}

Çok thread'li (çok iş parçacıklı) ticket store'unun ilk implementation'ında şunları kullandık:

- paylaşılan durumu tutmak için uzun ömürlü tek bir thread (sunucu)
- kendi thread'lerinden kanallar aracılığıyla ona istek gönderen birden fazla istemci.

Durumu kilitlemeye gerek yoktu, çünkü durumu değiştiren tek taraf sunucuydu. Bunun nedeni,
"gelen kutusu" kanalının gelen istekleri doğal olarak **sıraya sokmasıydı**: sunucu onları tek tek işliyordu.\
Bu yaklaşımın patch'leme (yamalama) davranışı söz konusu olduğundaki sınırlamalarını zaten tartıştık, ama
orijinal tasarımın performans etkilerini tartışmadık: sunucu, okumalar dahil, aynı anda yalnızca tek bir
isteği işleyebiliyordu.

## İnce Taneli Kilitleme {#fine-grained-locking}

Ardından, her ticket'ın kendi kilidiyle korunduğu ve istemcilerin bir ticket'ı okumak mı yoksa atomik olarak
değiştirmek mi istediklerine bağımsız olarak karar verip uygun kilidi aldıkları daha gelişmiş bir tasarıma geçtik.

Bu tasarım daha iyi bir paralelliğe izin verir (yani birden fazla istemci aynı anda ticket okuyabilir), ama
hâlâ temelde **sıralıdır**: sunucu komutları tek tek işler. Özellikle, kilitleri istemcilere
tek tek dağıtır.

Kanalları tamamen kaldırıp istemcilerin `TicketStore`'a doğrudan erişmesine izin verebilir, erişimi senkronize etmek için
yalnızca kilitlere güvenebilir miyiz?

## Kanalları Kaldırmak {#removing-channels}

Çözmemiz gereken iki sorun var:

- `TicketStore`'u thread'ler arasında paylaşmak
- Store'a erişimi senkronize etmek

### `TicketStore`'u Thread'ler Arasında Paylaşmak {#sharing-ticketstore-across-threads}

Tüm thread'lerin aynı duruma başvurmasını istiyoruz; aksi takdirde gerçekten çok thread'li bir sistemimiz olmaz—sadece
birden fazla tek thread'li sistemi paralel olarak çalıştırıyor oluruz.\
Bu sorunla, bir kilidi thread'ler arasında paylaşmaya çalıştığımızda zaten karşılaştık: bir `Arc` kullanabiliriz.

### Store'a Erişimi Senkronize Etmek {#synchronizing-access-to-the-store}

Kanalların sağladığı sıraya sokma sayesinde hâlâ kilitsiz olan bir etkileşim var: store'a bir ticket eklemek
(ya da store'dan bir ticket kaldırmak).\
Kanalları kaldırırsak, `TicketStore`'un kendisine erişimi senkronize etmek için (başka) bir kilit getirmemiz gerekir.

Bir `Mutex` kullanırsak, her ticket için ek bir `RwLock` kullanmanın anlamı kalmaz: `Mutex` zaten
store'un tamamına erişimi sıraya sokacağı için ticket'ları zaten paralel olarak okuyamayız.\
Bunun yerine bir `RwLock` kullanırsak, ticket'ları paralel olarak okuyabiliriz. Yalnızca bir ticket eklerken
ya da kaldırırken tüm okumaları duraklatmamız gerekir.

Bu yoldan gidelim ve bizi nereye götürdüğünü görelim.
