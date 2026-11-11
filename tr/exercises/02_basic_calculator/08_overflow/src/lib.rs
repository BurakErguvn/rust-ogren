// `dev` profilini, overflow durumunda wrap around yapacak şekilde özelleştir.
// Doğru söz dizimini bulmak için Cargo'nun dokümantasyonuna bak:
// https://doc.rust-lang.org/cargo/reference/profiles.html
//
// Daha sonra açıklayacağımız nedenlerle, özelleştirmenin egzersizin `Cargo.toml`'unda değil,
// repo'nun kökündeki `Cargo.toml`'da yapılması gerekiyor.

pub fn factorial(n: u32) -> u32 {
    let mut result = 1;
    for i in 1..=n {
        result *= i;
    }
    result
}

#[cfg(test)]
mod tests {
    use crate::factorial;

    #[test]
    fn twentieth() {
        // 20! 2432902008176640000 eder, bu da bir u32'ye sığmayacak kadar büyük
        // Varsayılan dev profiliyle, `cargo test` çalıştırdığında bu panic'ler
        // Biz bunun yerine wrap around yapmasını istiyoruz
        assert_eq!(factorial(20), 2_192_834_560);
        //                           ☝️
        // Okunabilirliği artırmak için alt çizgi kullanan büyük bir sayı literal'i!
    }

    #[test]
    fn first() {
        assert_eq!(factorial(0), 1);
    }

    #[test]
    fn second() {
        assert_eq!(factorial(1), 1);
    }

    #[test]
    fn third() {
        assert_eq!(factorial(2), 2);
    }

    #[test]
    fn fifth() {
        assert_eq!(factorial(5), 120);
    }
}
