// Negatif olmayan bir `n` tam sayısı verildiğinde, `n`'in faktöriyeli olan `n!` değerini
// döndüren `factorial` adında bir fonksiyon tanımla.
//
// `n`'in faktöriyeli, `n`'e kadar olan tüm pozitif tam sayıların çarpımı olarak tanımlanır.
// Örneğin `5!` ("beş faktöriyel" diye okunur) `5 * 4 * 3 * 2 * 1` eder, yani `120`.
// `0!`, `1` olarak tanımlanmıştır.
//
// `factorial(0)`'ın `1`, `factorial(1)`'in `1`,
// `factorial(2)`'nin `2` döndürmesini bekliyoruz, ve bu böyle devam ediyor.
//
// Yalnızca öğrendiklerini kullan! Henüz döngü yok, bu yüzden özyineleme kullanman gerekecek!

#[cfg(test)]
mod tests {
    use crate::factorial;

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
