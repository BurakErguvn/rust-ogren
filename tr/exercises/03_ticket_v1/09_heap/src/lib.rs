pub struct Ticket {
    title: String,
    description: String,
    status: String,
}

// TODO: bu kısımda öğrendiklerine dayanarak `todo!()`'yu ilgili türün
//  doğru **stack boyutuyla** değiştir.
#[cfg(test)]
mod tests {
    use super::Ticket;
    use std::mem::size_of;

    #[test]
    fn string_size() {
        assert_eq!(size_of::<String>(), todo!());
    }

    #[test]
    fn ticket_size() {
        // Bu biraz aldatıcı bir soru!
        // Bu sefer "sezgisel" cevap doğru cevap çıkıyor,
        // ama genel olarak struct'ların bellek düzeni daha karmaşık bir konudur.
        // Merak ediyorsan, daha fazla bilgi için The Rust Reference'ın "Type layout" kısmına
        // göz at: https://doc.rust-lang.org/reference/type-layout.html
        assert_eq!(size_of::<Ticket>(), todo!());
    }
}
