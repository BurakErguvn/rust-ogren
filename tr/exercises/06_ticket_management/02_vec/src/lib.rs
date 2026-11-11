// Bir `n` sayısı verildiğinde, Fibonacci serisindeki `n+1`. sayıyı döndür.
//
// Fibonacci serisi şöyle tanımlanır:
//
// - Serinin ilk sayısı 0'dır.
// - Serinin ikinci sayısı 1'dir.
// - Sonraki her sayı, kendinden önceki iki sayının toplamıdır.
//
// Yani seri şöyle ilerler: 0, 1, 1, 2, 3, 5, 8, 13, 21 ve böyle devam eder.
//
// `fibonacci(0)`'ın `0`, `fibonacci(1)`'in `1`, `fibonacci(2)`'nin `1` döndürmesini
// bekliyoruz ve bu böyle devam ediyor.
pub fn fibonacci(n: u32) -> u32 {
    // TODO: `fibonacci` fonksiyonunu implement et
    //
    // İpucu: daha önce hesapladığın sonuçları birkaç kez yeniden hesaplamak zorunda
    // kalmamak için onları bir `Vec` kullanarak önbelleğe al (memoize et).
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::fibonacci;

    #[test]
    fn first() {
        assert_eq!(fibonacci(0), 0);
    }

    #[test]
    fn second() {
        assert_eq!(fibonacci(1), 1);
    }

    #[test]
    fn third() {
        assert_eq!(fibonacci(2), 1);
    }

    #[test]
    fn tenth() {
        assert_eq!(fibonacci(10), 55);
    }

    #[test]
    fn thirtieth() {
        assert_eq!(fibonacci(30), 832040);
    }
}
