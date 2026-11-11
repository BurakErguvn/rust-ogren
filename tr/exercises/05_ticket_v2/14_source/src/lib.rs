use crate::status::Status;

// İlk egzersizlerden birinde modüllerin nasıl bildirildiğini gördük, ama
// onları ayrı dosyalara nasıl çıkaracağımızı görmedik.
// Şimdi bunu düzeltelim!
//
// En basit durumda, yani çıkarılan modül tek bir dosyadan oluşuyorsa, modülle aynı
// ada sahip yeni bir dosya oluşturup modülün içeriğini oraya taşımak yeterlidir.
// Modül dosyası, modülü bildiren dosyayla aynı dizine yerleştirilmelidir.
// Bu durumda o dosya `src/lib.rs` olduğundan, `status.rs` `src` dizinine yerleştirilmelidir.
mod status;

// TODO: Durum string'i geçersiz olduğunda kullanılmak üzere `TicketNewError`'a yeni bir hata
//   variant'ı ekle. O variant'tan bir hata üzerinde `source` çağrıldığında, `None` yerine
//   bir `ParseStatusError` döndürmeli.

#[derive(Debug, thiserror::Error)]
pub enum TicketNewError {
    #[error("Title cannot be empty")]
    TitleCannotBeEmpty,
    #[error("Title cannot be longer than 50 bytes")]
    TitleTooLong,
    #[error("Description cannot be empty")]
    DescriptionCannotBeEmpty,
    #[error("Description cannot be longer than 500 bytes")]
    DescriptionTooLong,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Ticket {
    title: String,
    description: String,
    status: Status,
}

impl Ticket {
    pub fn new(title: String, description: String, status: String) -> Result<Self, TicketNewError> {
        if title.is_empty() {
            return Err(TicketNewError::TitleCannotBeEmpty);
        }
        if title.len() > 50 {
            return Err(TicketNewError::TitleTooLong);
        }
        if description.is_empty() {
            return Err(TicketNewError::DescriptionCannotBeEmpty);
        }
        if description.len() > 500 {
            return Err(TicketNewError::DescriptionTooLong);
        }

        // TODO: Durum string'ini parse ederek bir `Status` enum'una dönüştür.

        Ok(Ticket {
            title,
            description,
            status,
        })
    }
}

#[cfg(test)]
mod tests {
    use common::{valid_description, valid_title};
    use std::error::Error;

    use super::*;

    #[test]
    fn invalid_status() {
        let err = Ticket::new(valid_title(), valid_description(), "invalid".into()).unwrap_err();
        assert_eq!(
            err.to_string(),
            "`invalid` is not a valid status. Use one of: ToDo, InProgress, Done"
        );
        assert!(err.source().is_some());
    }
}
