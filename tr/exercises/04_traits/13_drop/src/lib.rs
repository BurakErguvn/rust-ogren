// TODO: "Drop bombası" denen şeyi implement et: üzerinde belirli bir işlem yapılmadıkça
//  drop edildiğinde panic'leyen bir tür.
//  Beklenen API'yi aşağıdaki testlerde görebilirsin.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_drop_bomb() {
        let bomb = DropBomb::new();
        // Bomba drop edildiğinde panic'lemeli
    }

    #[test]
    fn test_defused_drop_bomb() {
        let mut bomb = DropBomb::new();
        bomb.defuse();
        // Bomba etkisiz hale getirildiği için
        // drop edildiğinde panic'lememeli
    }
}
