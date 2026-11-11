// Bu bir Rust dosyası. `.rs` uzantılı düz bir metin dosyasıdır.
//
// Çoğu modern programlama dili gibi Rust da yorumları destekler. Şu anda bir tanesine bakıyorsun!
// Yorumlar derleyici tarafından yok sayılır; kodu notlar ve açıklamalarla zenginleştirmek için
// onlardan yararlanabilirsin.
// Rust'ta yorum yazmanın, her birinin kendi amacı olan çeşitli yolları vardır.
// Şimdilik en yaygın olanıyla yetineceğiz: satır yorumu.
// `//` işaretinden satırın sonuna kadar olan her şey yorum olarak kabul edilir.

// Egzersizlerde, kod yazman gereken satırlara dikkatini çekmek için `TODO`, `todo!()` ya da `__`
// işaretleri bulunur.
// Egzersizi tamamlamak için bu işaretleri kendi kodunla değiştirmen gerekecek.
// Bazen tek bir satır kod yazmak yeterli olacak, bazen de daha uzun kod parçaları yazman
// gerekecek.
//
// Bir egzersizde 10 dakikadan fazla takılırsan bir eğitmene başvur! Yardım etmek için buradayız!
// Tüm egzersizlerin çözümlerini `solutions` git branch'inde de bulabilirsin.
fn greeting() -> &'static str {
    // TODO: beni düzelt 👇
    "I'm ready to __!"
}

// Çözümlerin bir dizi test tarafından otomatik olarak doğrulanacak.
// Bu testleri, bu egzersizin dizininin kökündeyken terminalinde `cargo test` komutunu çağırarak
// doğrudan çalıştırabilirsin. `wr` komutunun arka planda senin için yaptığı da budur.
//
// Rust, testleri kodunla yan yana yazmana olanak tanır.
// `#[cfg(test)]` attribute'u derleyiciye aşağıdaki kodu yalnızca testler çalıştırılırken
// (yani `cargo test` çalıştırdığında) derlemesini söyler.
// Attribute'lar ve test yazma hakkında kursun ilerleyen kısımlarında daha fazlasını öğreneceksin.
// Şimdilik, çözümlerinin doğruluğunu kontrol edecek testleri bulmak için `#[cfg(test)]`
// attribute'una bakman gerektiğini bilmen yeterli!
//
// ⚠️ **TESTLERİ DEĞİŞTİRME** ⚠️
// Testler çözümlerini doğrulamana yardımcı olmak için oradalar. Testlerin kendisini değil, yalnızca
// test edilen kodu değiştirmelisin.
#[cfg(test)]
mod tests {
    use crate::greeting;

    #[test]
    fn test_welcome() {
        assert_eq!(greeting(), "I'm ready to learn Rust!");
    }
}
