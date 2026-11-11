mod ticket {
    struct Ticket {
        title: String,
        description: String,
        status: String,
    }

    impl Ticket {
        fn new(title: String, description: String, status: String) -> Ticket {
            if title.is_empty() {
                panic!("Title cannot be empty");
            }
            if title.len() > 50 {
                panic!("Title cannot be longer than 50 bytes");
            }
            if description.is_empty() {
                panic!("Description cannot be empty");
            }
            if description.len() > 500 {
                panic!("Description cannot be longer than 500 bytes");
            }
            if status != "To-Do" && status != "In Progress" && status != "Done" {
                panic!("Only `To-Do`, `In Progress`, and `Done` statuses are allowed");
            }

            Ticket {
                title,
                description,
                status,
            }
        }
    }
}

// TODO: Bu egzersizde **istisnai olarak** hem `ticket` modülünü hem de `tests` modülünü
//  değiştireceksin.
#[cfg(test)]
mod tests {
    // TODO: Aşağıdaki use deyimiyle ilgili derleyici hatalarını gidermek için üst modüle
    //  gerekli `pub` belirleyicilerini ekle.
    use super::ticket::Ticket;

    // Yine de dikkatli ol! use deyimini derlenir hale getirmek için görünürlüğü
    // değiştirdikten sonra bu fonksiyonun derlenmesini istemiyoruz!
    // Gerçekten derlenmediğini doğruladıktan sonra onu yorum satırına al.
    fn should_not_be_possible() {
        let ticket = Ticket::new("A title".into(), "A description".into(), "To-Do".into());

        // Bu egzersizi çalıştırmaya çalıştığında şu hatayı görüyor olmalısın:
        //
        // error[E0616]: field `description` of struct `Ticket` is private
        //    |
        //    |              assert_eq!(ticket.description, "A description");
        //    |                         ^^^^^^^^^^^^^^^^^^
        //
        // TODO: Aşağıdakinin derlenmediğini doğruladıktan sonra, sonraki egzersize
        //   geçmek için o satırı yorum satırına al!
        assert_eq!(ticket.description, "A description");
    }

    fn encapsulation_cannot_be_violated() {
        // Bu da, yukarıda karşılaşılana benzer bir hatayla, imkânsız olmalı.
        // (Derleme hatasını yalnızca önceki testteki hatalı satırı yorum satırına
        // aldıktan sonra verecek - bir sonraki derleme aşaması!)
        //
        // Bu, artık bir `Ticket` instance'ı elde etmenin tek yolunun `Ticket::new` olduğunu kanıtlar.
        // Geçersiz bir başlık ya da açıklamayla bir ticket oluşturmak imkânsız!
        //
        // TODO: Aşağıdakinin derlenmediğini doğruladıktan sonra, sonraki egzersize
        //   geçmek için o satırları yorum satırına al!
        let ticket = Ticket {
            title: "A title".into(),
            description: "A description".into(),
            status: "To-Do".into(),
        };
    }
}
