// TODO: bu crate'teki modüllerin her birinde yapman gereken bir şey var!
mod description;
mod status;
mod title;

// Rust'ta yaygın bir kalıp, kodu birden çok (private) modüle bölmek ve ardından
// bu modüllerin public kısımlarını crate'in kökünde re-export etmektir.
//
// Bu, crate'in iç yapısını kullanıcılarından gizlerken kodunu istediğin gibi
// düzenlemene de olanak tanır.
pub use description::TicketDescription;
pub use status::Status;
pub use title::TicketTitle;

#[derive(Debug, PartialEq, Clone)]
// Artık alanları private yapmamız gerekmiyor!
// Her alan kendi doğrulama mantığını kapsüllediği için, `Ticket`'ı kullanan birinin
// alanları struct'ın invariant'larını bozacak şekilde değiştirmesi riski yoktur.
//
// Yine de dikkatli ol: birden çok alana yayılan invariant'ların olsaydı, bu
// invariant'ların hâlâ korunduğundan emin olman ve alanları yeniden private
// yapmaya geri dönmen gerekirdi.
pub struct Ticket {
    pub title: TicketTitle,
    pub description: TicketDescription,
    pub status: Status,
}
