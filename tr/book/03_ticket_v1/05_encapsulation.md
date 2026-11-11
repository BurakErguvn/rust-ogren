# Kapsülleme {#encapsulation}

Artık modüller ve görünürlük hakkında temel bir anlayışa sahip olduğumuza göre, **kapsüllemeye** geri dönelim.\
Kapsülleme, bir nesnenin iç temsilini gizleme pratiğidir. En yaygın olarak, nesnenin durumu üzerinde
bazı **invariant**'ları (değişmezleri) zorunlu kılmak için kullanılır.

`Ticket` struct'ımıza dönersek:

```rust
struct Ticket {
    title: String,
    description: String,
    status: String,
}
```

Tüm alanlar public yapılırsa kapsülleme olmaz.\
Alanların herhangi bir anda değiştirilebileceğini ve türlerinin izin verdiği herhangi bir değere
ayarlanabileceğini varsaymalısın. Bir ticket'ın boş bir başlığa ya da anlamsız bir duruma
sahip olabileceği ihtimalini dışlayamazsın.

Daha katı kuralları zorunlu kılmak için alanları private tutmalıyız[^newtype].
Ardından bir `Ticket` instance'ı (örneği) ile etkileşim kurmak için public metotlar sağlayabiliriz.
Bu public metotlar, invariant'larımızı korumaktan sorumlu olacaktır (ör. bir başlık boş olmamalıdır).

En az bir alan private ise, struct instance oluşturma söz dizimini kullanarak doğrudan bir `Ticket` instance'ı
oluşturmak artık mümkün değildir:

```rust
// Bu çalışmaz!
let ticket = Ticket {
    title: "Build a ticket system".into(),
    description: "A Kanban board".into(),
    status: "Open".into()
};
```

Bunu görünürlükle ilgili önceki egzersizde iş başında gördün.\
Şimdi bir ya da daha fazla public **constructor** (yapıcı) sağlamamız gerekiyor—yani modülün dışından, struct'ın
yeni bir instance'ını oluşturmak için kullanılabilen statik metotlar ya da fonksiyonlar.\
Neyse ki elimizde zaten bir tane var: [önceki bir egzersizde](02_validation.md) implement ettiğimiz `Ticket::new`.

## Erişim Metotları {#accessor-methods}

Özetle:

- Tüm `Ticket` alanları private
- Oluşturma sırasında doğrulama kurallarımızı zorunlu kılan public bir constructor olan `Ticket::new`'yu sağlıyoruz

Bu iyi bir başlangıç, ama yeterli değil: bir `Ticket` oluşturmanın yanı sıra onunla etkileşim kurmamız da gerekiyor.
Peki alanlar private ise onlara nasıl erişebiliriz?

**Erişim metotları** sağlamamız gerekiyor.\
Erişim metotları, bir struct'ın private bir alanının (ya da alanlarının) değerini okumanı sağlayan public metotlardır.

Rust'ta, bazı diğer dillerde olduğu gibi erişim metotlarını senin için üretmenin yerleşik bir yolu yoktur.
Onları kendin yazman gerekir—sadece sıradan metotlardır.

[^newtype]: Ya da türlerini daha hassas hale getirmek; bu, [ileride](../05_ticket_v2/15_outro.md) inceleyeceğimiz bir tekniktir.
