// TODO: testlerin geçmesi için aşağıdaki fonksiyon imzasını düzelt.
//  Derleyici hata mesajını mutlaka oku—Rust derleyicisi bu kursta senin eşli programlama
//  partnerin ve seni çoğu zaman doğru yöne sevk edecek!
//
// Girdi parametreleri dönüş türüyle aynı türde olmalı.
fn compute(a, b) -> u32 {
    // Fonksiyon gövdesine dokunma.
    a + b * 2
}

#[cfg(test)]
mod tests {
    use crate::compute;

    #[test]
    fn case() {
        assert_eq!(compute(1, 2), 5);
    }
}