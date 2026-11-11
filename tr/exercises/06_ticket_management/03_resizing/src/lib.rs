#[cfg(test)]
mod tests {
    #[test]
    fn resizing() {
        let mut v = Vec::with_capacity(2);
        v.push(1);
        v.push(2); // maksimum kapasiteye ulaşıldı
        assert_eq!(v.capacity(), 2);

        v.push(3); // kapasite aşıldı, yeniden boyutlandırılması gerekiyor

        // Yeni kapasitenin ne olacağını tahmin edebilir misin?
        // Dikkat: standart kütüphane, vektörü yeniden boyutlandırmak için kullanılan
        // algoritma hakkında hiçbir garanti vermez, dolayısıyla bu gelecekte değişebilir.
        assert_eq!(v.capacity(), todo!());
    }
}
