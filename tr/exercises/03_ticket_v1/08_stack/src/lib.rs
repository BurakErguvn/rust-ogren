// TODO: bu kısımda öğrendiklerine dayanarak `todo!()`'yu ilgili türün
//  doğru **stack boyutuyla** değiştir.
#[cfg(test)]
mod tests {
    use std::mem::size_of;

    #[test]
    fn u16_size() {
        assert_eq!(size_of::<u16>(), todo!());
    }

    #[test]
    fn i32_size() {
        assert_eq!(size_of::<i32>(), todo!());
    }

    #[test]
    fn bool_size() {
        assert_eq!(size_of::<bool>(), todo!());
    }
}
