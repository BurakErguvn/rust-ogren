// TODO: `title` ve `description` erişim metotları aracılığıyla her döndürüldüğünde
//   normalize edilmiş olmalıdır—yani baştaki ve sondaki boşluk karakterleri kaldırılmalıdır.
//   Rust'ın standart kütüphanesinde bu konuda yardımcı olabilecek bir metot var, ama onu
//   `String` dokümantasyonunda bulamazsın.
//   Nerede tanımlandığını ve nasıl kullanılacağını bulabilir misin?

pub struct Ticket {
    title: String,
    description: String,
    status: String,
}

impl Ticket {
    pub fn title(&self) -> &str {
        todo!()
    }

    pub fn description(&self) -> &str {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let ticket = Ticket {
            title: "   A title ".to_string(),
            description: " A description   ".to_string(),
            status: "To-Do".to_string(),
        };

        assert_eq!("A title", ticket.title());
        assert_eq!("A description", ticket.description());
    }
}
