// TODO: `spawn` ve `join` kullanarak `sum` fonksiyonunun çok thread'li
//  bir sürümünü implement et.
//  Bir tam sayı vektörü verildiğinde, vektörü iki yarıya böl ve
//  her yarıyı ayrı bir thread'de topla.

// Uyarı: Fonksiyonun *nasıl* implement edildiğini test edemeyiz,
// yalnızca doğru sonucu ürettiğini doğrulayabiliriz.
// Bu testi sadece `v.iter().sum()` döndürerek _geçebilirdin_,
// ama bu, egzersizin amacını boşa çıkarırdı.
//
// İpucu: spawn edilen thread'lerin vektörün slice'larını doğrudan
// _ödünç almasını_ sağlayamayacaksın. Orijinal vektörün her yarısı için
// bellek ayırarak yeni vektörler oluşturman gerekecek. Bunun neden
// gerekli olduğunu bir sonraki egzersizde göreceğiz.
use std::thread;

pub fn sum(v: Vec<i32>) -> i32 {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(sum(vec![]), 0);
    }

    #[test]
    fn one() {
        assert_eq!(sum(vec![1]), 1);
    }

    #[test]
    fn five() {
        assert_eq!(sum(vec![1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn nine() {
        assert_eq!(sum(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]), 45);
    }

    #[test]
    fn ten() {
        assert_eq!(sum(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]), 55);
    }
}
