// TODO: Yeni bir trait tanımla: `Power`. Bu trait'in, `self`'i `n`'inci kuvvetine yükselten
//  bir `power` metodu olsun.
//  Trait tanımı ve implementation'ları, testlerin derlenip geçmesi için
//  yeterli olmalıdır.
//
// Öneri: tüm durumları tek seferde ele alan generic bir implementation yazmak cazip
// gelebilir. Ancak bu oldukça karmaşıktır ve ek crate'lerin (yani `num-traits`)
// kullanılmasını gerektirir.
// O durumda bile, son derece generic bir implementation'ın karmaşıklığından kaçınmak için
// bunun yerine basit bir makro kullanmak tercih edilebilir. Bu konuda daha fazla bilgi
// edinmek istiyorsan "Little book of Rust macros" (https://veykril.github.io/tlborm/)
// kitabına göz at.
// Ama bunu yapmak zorunda değilsin: üç ayrı implementation'ı elle yazmak tamamen
// kabul edilebilir. Daha ileri yalnızca merak ediyorsan git.

#[cfg(test)]
mod tests {
    use super::Power;

    #[test]
    fn test_power_u16() {
        let x: u32 = 2_u32.power(3u16);
        assert_eq!(x, 8);
    }

    #[test]
    fn test_power_u32() {
        let x: u32 = 2_u32.power(3u32);
        assert_eq!(x, 8);
    }

    #[test]
    fn test_power_ref_u32() {
        let x: u32 = 2_u32.power(&3u32);
        assert_eq!(x, 8);
    }
}
