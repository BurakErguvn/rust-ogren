/// Bir yolculuğun başlangıç ve bitiş noktaları ile yolculuğu tamamlamanın ne kadar sürdüğü
/// verildiğinde, yolculuğun ortalama hızını hesapla.
fn speed(start: u32, end: u32, time_elapsed: u32) -> u32 {
    // TODO: `time_elapsed` 0 ise özel bir mesajla panic'le

    (end - start) / time_elapsed
}

#[cfg(test)]
mod tests {
    use crate::speed;

    #[test]
    fn case1() {
        assert_eq!(speed(0, 10, 10), 1);
    }

    #[test]
    // 👇 `#[should_panic]` belirtimiyle, test edilen kodun panic'lemesini beklediğimizi
    //    assert edebiliriz. `expected` kullanarak panic mesajını da kontrol edebiliriz.
    //    Bunların hepsi Rust'ın yerleşik test framework'ünün bir parçası!
    #[should_panic(expected = "The journey took no time at all. That's impossible!")]
    fn by_zero() {
        speed(0, 10, 0);
    }
}
