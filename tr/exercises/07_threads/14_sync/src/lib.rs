// `Sync` üzerine alıştırma yapılacak pek bir şey yok, sadece akılda tutulması gereken bir şey.
fn outro() -> &'static str {
    "I have a good understanding of __!"
}

#[cfg(test)]
mod tests {
    use crate::outro;

    #[test]
    fn test_outro() {
        assert_eq!(outro(), "I have a good understanding of Send and Sync!");
    }
}
