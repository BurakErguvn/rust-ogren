struct Ticket {
    title: String,
    description: String,
    status: String,
}

impl Ticket {
    // TODO: `new` fonksiyonunu implement et.
    //  Şu gereksinimler karşılanmalı:
    //   - Yalnızca `To-Do`, `In Progress` ve `Done` durumlarına izin verilir.
    //   - `title` ve `description` alanları boş olmamalı.
    //   - `title` en fazla 50 byte uzunluğunda olmalı.
    //   - `description` en fazla 500 byte uzunluğunda olmalı.
    //  Gereksinimlerden herhangi biri karşılanmazsa metot panic'lemeli.
    //  Gereken panic mesajlarını testlerde bulabilirsin.
    //
    // Önceki egzersizlerde öğrendiklerini ve bazı `String` metotlarını kullanman
    // gerekecek. En uygun seçenekleri bulmak için Rust'ın standart kütüphanesinin
    // dokümantasyonunu kullan -> https://doc.rust-lang.org/std/string/struct.String.html
    fn new(title: String, description: String, status: String) -> Self {
        todo!();
        Self {
            title,
            description,
            status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::{overly_long_description, overly_long_title, valid_description, valid_title};

    #[test]
    #[should_panic(expected = "Title cannot be empty")]
    fn title_cannot_be_empty() {
        Ticket::new("".into(), valid_description(), "To-Do".into());
    }

    #[test]
    #[should_panic(expected = "Description cannot be empty")]
    fn description_cannot_be_empty() {
        Ticket::new(valid_title(), "".into(), "To-Do".into());
    }

    #[test]
    #[should_panic(expected = "Title cannot be longer than 50 bytes")]
    fn title_cannot_be_longer_than_fifty_chars() {
        Ticket::new(overly_long_title(), valid_description(), "To-Do".into());
    }

    #[test]
    #[should_panic(expected = "Description cannot be longer than 500 bytes")]
    fn description_cannot_be_longer_than_500_chars() {
        Ticket::new(valid_title(), overly_long_description(), "To-Do".into());
    }

    #[test]
    #[should_panic(expected = "Only `To-Do`, `In Progress`, and `Done` statuses are allowed")]
    fn status_must_be_valid() {
        Ticket::new(valid_title(), valid_description(), "Funny".into());
    }

    #[test]
    fn done_is_allowed() {
        Ticket::new(valid_title(), valid_description(), "Done".into());
    }

    #[test]
    fn in_progress_is_allowed() {
        Ticket::new(valid_title(), valid_description(), "In Progress".into());
    }
}
