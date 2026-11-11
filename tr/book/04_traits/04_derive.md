# Derive Macro'lar {#derive-macros}

`Ticket` için `PartialEq` implement etmek biraz sıkıcıydı, değil mi?
Struct'ın her alanını elle karşılaştırman gerekti.

## Destructuring (Parçalara Ayırma) Söz Dizimi {#destructuring-syntax}

Üstelik bu implementation kırılgandır: struct tanımı değişirse
(örneğin yeni bir alan eklenirse), `PartialEq` implementation'ını güncellemeyi hatırlaman gerekir.

Struct'ı alanlarına **destructure ederek** bu riski azaltabilirsin:

```rust
impl PartialEq for Ticket {
    fn eq(&self, other: &Self) -> bool {
        let Ticket {
            title,
            description,
            status,
        } = self;
        // [...]
    }
}
```

`Ticket`'ın tanımı değişirse derleyici hata verir ve destructuring'inin artık
eksiksiz olmadığından şikâyet eder.\
Değişken shadowing'ini (gölgelemesini) önlemek için struct alanlarını yeniden adlandırabilirsin de:

```rust
impl PartialEq for Ticket {
    fn eq(&self, other: &Self) -> bool {
        let Ticket {
            title,
            description,
            status,
        } = self;
        let Ticket {
            title: other_title,
            description: other_description,
            status: other_status,
        } = other;
        // [...]
    }
}
```

Destructuring, araç kutunda bulunması gereken kullanışlı bir kalıptır, ama
bunu yapmanın daha da pratik bir yolu var: **derive macro**'lar.

## Makrolar {#macros}

Önceki egzersizlerde zaten birkaç makroyla karşılaştın:

- Test case'lerinde `assert_eq!` ve `assert!`
- Konsola yazdırmak için `println!`

Rust makroları **kod üreteçleridir**.\
Sağladığın girdiye göre yeni Rust kodu üretirler ve üretilen bu kod daha sonra programının geri kalanıyla
birlikte derlenir. Bazı makrolar Rust'ın standart kütüphanesinde yerleşik olarak bulunur, ama kendi makrolarını
da yazabilirsin. Bu kursta kendi makromuzu oluşturmayacağız, ama ["İleri Okuma" kısmında](#further-reading)
bazı faydalı kaynaklar bulabilirsin.

### İnceleme {#inspection}

Bazı IDE'ler, üretilen kodu incelemen için bir makroyu genişletmene olanak tanır. Bu mümkün değilse
[`cargo-expand`](https://github.com/dtolnay/cargo-expand) kullanabilirsin.

### Derive Macro'lar {#derive-macros-1}

**Derive macro**, Rust makrolarının özel bir türüdür. Bir struct'ın üzerine bir **attribute** (öznitelik) olarak yazılır.

```rust
#[derive(PartialEq)]
struct Ticket {
    title: String,
    description: String,
    status: String
}
```

Derive macro'lar, özel türler için yaygın (ve "bariz") trait'lerin implementation'ını otomatikleştirmek için kullanılır.
Yukarıdaki örnekte `PartialEq` trait'i `Ticket` için otomatik olarak implement edilir.
Makroyu genişletirsen, üretilen kodun elle yazdığın kodla işlevsel olarak eşdeğer olduğunu,
ama okunmasının biraz daha zahmetli olduğunu görürsün:

```rust
#[automatically_derived]
impl ::core::cmp::PartialEq for Ticket {
    #[inline]
    fn eq(&self, other: &Ticket) -> bool {
        self.title == other.title 
            && self.description == other.description
            && self.status == other.status
    }
}
```

Derleyici, mümkün olduğunda trait'leri derive etmen için seni yönlendirir.

## İleri Okuma {#further-reading}

- [The little book of Rust macros](https://veykril.github.io/tlborm/)
- [Proc macro workshop](https://github.com/dtolnay/proc-macro-workshop)
