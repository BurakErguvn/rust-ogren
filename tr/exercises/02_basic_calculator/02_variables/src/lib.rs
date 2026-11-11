// 👇 `///` ile başlayan aşağıdaki satırlara **dokümantasyon yorumları** denir.
//    Dokümantasyonu kendilerinden sonra gelen öğeye iliştirirler. Bu durumda, `speed` fonksiyonuna.
//    Bu egzersizin dizininden `cargo doc --open` çalıştırırsan, Rust bu yorumlardan
//    HTML dokümantasyonu üretir ve tarayıcında açar.

/// Bir yolculuğun başlangıç ve bitiş noktaları ile yolculuğu tamamlamanın ne kadar sürdüğü
/// verildiğinde, ortalama hızı hesapla.
pub fn speed(start: u32, end: u32, time_elapsed: u32) -> u32 {
    // TODO: testlerin geçmesi için doğru değere sahip, `distance` adında bir değişken tanımla
    //  `distance`'ın türünü belirtmen gerekiyor mu? Neden gerekiyor ya da neden gerekmiyor?

    // Aşağıdaki satırı değiştirme
    distance / time_elapsed
}

#[cfg(test)]
mod tests {
    use crate::speed;

    #[test]
    fn case1() {
        assert_eq!(speed(0, 10, 10), 1);
    }

    #[test]
    fn case2() {
        assert_eq!(speed(10, 30, 10), 2);
    }

    #[test]
    fn case3() {
        assert_eq!(speed(10, 31, 10), 2);
    }
}
