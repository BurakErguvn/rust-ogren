// Destructor'lar için düzgün bir egzersiz yazabilmek için biraz daha altyapıya ihtiyacımız var.
// Trait'leri ve interior mutability'yi ele aldıktan sonra, sonraki bir bölümde bu kavrama
// geri döneceğiz.
fn outro() -> &'static str {
    "I have a basic understanding of __!"
}

#[cfg(test)]
mod tests {
    use crate::outro;

    #[test]
    fn test_outro() {
        assert_eq!(outro(), "I have a basic understanding of destructors!");
    }
}
