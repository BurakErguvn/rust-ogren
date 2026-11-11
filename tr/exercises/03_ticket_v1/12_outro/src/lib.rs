// TODO: Yeni bir `Order` türü tanımla.
//   Üç bilgiyi takip etmeli: `product_name`, `quantity` ve `unit_price`.
//   Ürün adı boş olamaz ve 300 byte'tan uzun olamaz.
//   Miktar kesinlikle sıfırdan büyük olmalı.
//   Birim fiyat sent cinsindendir ve kesinlikle sıfırdan büyük olmalı.
//   Order, siparişin toplam fiyatını döndüren `total` adında bir metot içermeli.
//   Order, her alan için setter'lar ve getter'lar sağlamalı.
//
// Testler bu sefer farklı bir yerde—`tests` klasöründe.
// `tests` klasörü `cargo` için özel bir konumdur. `cargo`, **entegrasyon testlerini** orada arar.
// Entegrasyon burada çok belirli bir anlama sahip: bu testler projenin **public API'sini** test eder.
// Türlerinin ve metotlarının görünürlüğüne dikkat etmen gerekecek; entegrasyon
// testleri private ya da `pub(crate)` öğelere erişemez.
