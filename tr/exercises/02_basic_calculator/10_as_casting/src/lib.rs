// TODO: bu kısımda öğrendiklerine dayanarak `todo!()`'yu
//  dönüşümden sonraki doğru değerle değiştir.

#[cfg(test)]
mod tests {

    #[test]
    fn u16_to_u32() {
        let v: u32 = todo!();
        assert_eq!(47u16 as u32, v);
    }

    #[test]
    fn u8_to_i8() {
        // Derleyici, 255 değerinin bir i8'e sığamayacağını bilecek kadar akıllıdır,
        // bu yüzden doğrudan hata verir. Bu (kötü) dönüşümü mümkün kılmak için
        // bu güvenlik önlemini kasıtlı olarak devre dışı bırakıyoruz.
        // Derleyici bunu yalnızca değer bir literal olduğu için fark edebiliyor.
        // Bir değişken kullansaydık, derleyici bunu derleme zamanında
        // yakalayamazdı.
        #[allow(overflowing_literals)]
        let x = { 255 as i8 };

        // Bunu yukarıdakiyle tamamen aynı ifadeyi kullanarak çözebilirdin,
        // ama bu, egzersizin amacını boşa çıkarır. Bunun yerine, `u8`'e dönüştürüldüğünde
        // `255`'e eşdeğer olan gerçek bir `i8` değeri kullan.
        let y: i8 = todo!();

        assert_eq!(x, y);
    }

    #[test]
    fn bool_to_u8() {
        let v: u8 = todo!();
        assert_eq!(true as u8, v);
    }
}
