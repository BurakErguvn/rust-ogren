# Thread'ler (İş Parçacıkları) {#threads}

Rust'ın büyük vaatlerinden biri _fearless concurrency_ (korkusuz eşzamanlılık): güvenli, eşzamanlı programlar yazmayı kolaylaştırmak.
Şimdiye kadar bunun pek bir örneğini görmedik. Bugüne kadar yaptığımız tüm işler tek thread'liydi.
Bunu değiştirme zamanı!

Bu bölümde ticket store'unu çok thread'li hale getireceğiz.\
Rust'ın temel eşzamanlılık özelliklerinin çoğuna değinme fırsatı bulacağız; örneğin:

- `std::thread` modülü ile thread'ler
- Kanallar ile mesaj iletimi
- `Arc`, `Mutex` ve `RwLock` ile paylaşılan durum
- Rust'ın eşzamanlılık garantilerini kodlayan trait'ler: `Send` ve `Sync`

Ayrıca çok thread'li sistemler için çeşitli tasarım desenlerini ve bunların bazı trade-off'larını (ödünleşimlerini) da tartışacağız.
