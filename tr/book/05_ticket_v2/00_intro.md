# Bir Ticket'ı Modellemek, 2. Kısım {#modelling-a-ticket-pt-2}

Önceki bölümlerde üzerinde çalıştığımız `Ticket` struct'ı iyi bir başlangıç,
ama hâlâ "Ben acemi bir Rustacean'ım!" diye bağırıyor.

Bu bölümü, Rust'ta domain modelleme becerilerimizi geliştirmek için kullanacağız.
Bu süreçte birkaç yeni kavramı daha tanıtmamız gerekecek:

- `enum`'lar: Rust'ın veri modelleme için en güçlü özelliklerinden biri
- `Option` türü: null olabilen değerleri modellemek için
- `Result` türü: kurtarılabilir hataları modellemek için
- `Debug` ve `Display` trait'leri: yazdırmak için
- `Error` trait'i: hata türlerini işaretlemek için
- `TryFrom` ve `TryInto` trait'leri: başarısız olabilen dönüşümler için
- Rust'ın paket sistemi: kütüphane nedir, binary nedir, üçüncü taraf crate'ler nasıl kullanılır
