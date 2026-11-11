// Aşağıdaki alanlara sahip `Order` adında bir struct tanımla:
// - `price`, işaretsiz bir tam sayı
// - `quantity`, işaretsiz bir tam sayı
//
// Ayrıca `is_available` adında bir metodu olmalı; bu metot miktar 0'dan büyükse `true`,
// aksi halde `false` döndürmeli.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_is_available() {
        let order = Order {
            price: 100,
            quantity: 10,
        };
        assert!(order.is_available());
    }

    #[test]
    fn test_order_is_not_available() {
        let order = Order {
            price: 100,
            quantity: 0,
        };
        assert!(!order.is_available());
    }
}
