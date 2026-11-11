// `self` çift ise `true`, aksi halde `false` döndüren bir `is_even` metoduna sahip, `IsEven` adlı
// bir trait tanımla.
//
// Ardından trait'i `u32` ve `i32` için implement et.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u32_is_even() {
        assert!(42u32.is_even());
        assert!(!43u32.is_even());
    }

    #[test]
    fn test_i32_is_even() {
        assert!(42i32.is_even());
        assert!(!43i32.is_even());
        assert!(0i32.is_even());
        assert!(!(-1i32).is_even());
    }
}
