// TODO: Bu egzersizin başarıyla derlenmesi için gereken (derive edilebilir) bir trait
//   implementation'ı eksik. Düzelt!
//
// # `Debug`'a kısa bir giriş
//
// `Debug`, bir Rust türünün hata ayıklamaya uygun bir gösterimini döndürür (adı da buradan
// gelir).
// `assert_eq!`, `Ticket`'ın `Debug` implement etmesini gerektirir, çünkü assertion başarısız olduğunda
// karşılaştırmanın her iki tarafını da terminale yazdırmaya çalışır.
// Karşılaştırılan tür `Debug` implement etmiyorsa, onları nasıl temsil edeceğini bilemez!

#[derive(PartialEq)]
struct Ticket {
    title: String,
    description: String,
    status: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partial_eq() {
        let title = "title";
        let description = "description";
        let status = "To-Do";
        let ticket1 = Ticket {
            title: title.to_string(),
            description: description.to_string(),
            status: status.to_string(),
        };
        let ticket2 = Ticket {
            title: title.to_string(),
            description: description.to_string(),
            status: status.to_string(),
        };
        assert_eq!(ticket1, ticket2);
    }

    #[test]
    fn test_description_not_matching() {
        let title = "title";
        let status = "To-Do";
        let ticket1 = Ticket {
            title: title.to_string(),
            description: "description".to_string(),
            status: status.to_string(),
        };
        let ticket2 = Ticket {
            title: title.to_string(),
            description: "description2".to_string(),
            status: status.to_string(),
        };
        assert_ne!(ticket1, ticket2);
    }

    #[test]
    fn test_title_not_matching() {
        let status = "To-Do";
        let description = "description";
        let ticket1 = Ticket {
            title: "title".to_string(),
            description: description.to_string(),
            status: status.to_string(),
        };
        let ticket2 = Ticket {
            title: "title2".to_string(),
            description: description.to_string(),
            status: status.to_string(),
        };
        assert_ne!(ticket1, ticket2);
    }

    #[test]
    fn test_status_not_matching() {
        let title = "title";
        let description = "description";
        let ticket1 = Ticket {
            title: title.to_string(),
            description: description.to_string(),
            status: "status".to_string(),
        };
        let ticket2 = Ticket {
            title: title.to_string(),
            description: description.to_string(),
            status: "status2".to_string(),
        };
        assert_ne!(ticket1, ticket2);
    }
}
