# Modüller {#modules}

Az önce tanımladığın `new` metodu, `Ticket`'ın alan değerleri üzerinde bazı **kısıtlamaları** zorunlu kılmaya çalışıyor.
Peki bu invariant'lar (değişmezler) gerçekten zorunlu kılınıyor mu? Bir geliştiricinin `Ticket::new`'dan geçmeden
bir `Ticket` oluşturmasını ne engelliyor?

Gerçek bir **kapsülleme** elde etmek için iki yeni kavramı tanıman gerekiyor: **görünürlük** ve **modüller**.
Modüllerle başlayalım.

## Modül Nedir? {#what-is-a-module}

Rust'ta **modül**, birbiriyle ilişkili kodu ortak bir ad alanı (yani modülün adı) altında bir araya getirmenin bir yoludur.\
Modülleri iş başında zaten gördün: kodunun doğruluğunu sınayan birim testleri, `tests` adlı
farklı bir modülde tanımlanır.

```rust
#[cfg(test)]
mod tests {
    // [...]
}
```

## Satır İçi Modüller {#inline-modules}

Yukarıdaki `tests` modülü bir **satır içi modül** örneğidir: modül bildirimi (`mod tests`) ve modülün
içeriği (`{ ... }` içindeki her şey) yan yanadır.

## Modül Ağacı {#module-tree}

Modüller iç içe olabilir ve böylece bir **ağaç** yapısı oluşturur.\
Ağacın kökü, diğer tüm modülleri içeren en üst düzey modül olan **crate**'in kendisidir.
Bir kütüphane crate'i için kök modül genellikle `src/lib.rs` dosyasıdır (konumu özelleştirilmemişse).
Kök modül **crate kökü** olarak da bilinir.

Crate kökünün alt modülleri olabilir; bunların da kendi alt modülleri olabilir ve bu böyle devam eder.

## Harici Modüller Ve Dosya Sistemi {#external-modules-and-the-filesystem}

Satır içi modüller küçük kod parçaları için kullanışlıdır, ama projen büyüdükçe kodunu birden fazla
dosyaya bölmek isteyeceksin. Üst modülde, `mod` anahtar kelimesini kullanarak bir alt modülün varlığını bildirirsin.

```rust
mod dog;
```

Ardından, modülün implementation'ını içeren dosyayı bulmak Rust'ın derleme aracı olan
`cargo`'nun görevidir.\
Modülün crate'inin kökünde bildirildiyse (ör. `src/lib.rs` ya da `src/main.rs`),
`cargo` dosyanın şu iki şekilden biriyle adlandırılmasını bekler:

- `src/<module_name>.rs`
- `src/<module_name>/mod.rs`

Modülün başka bir modülün alt modülüyse, dosyanın adı şöyle olmalıdır:

- `[..]/<parent_module>/<module_name>.rs`
- `[..]/<parent_module>/<module_name>/mod.rs`

Ör. `dog`, `animals`'ın bir alt modülüyse `src/animals/dog.rs` ya da `src/animals/dog/mod.rs`.

`mod` anahtar kelimesini kullanarak yeni bir modül bildirdiğinde, IDE'n bu dosyaları otomatik olarak oluşturmana yardımcı olabilir.

## Öğe Yolları Ve `use` Deyimleri {#item-paths-and-use-statements}

Aynı modülde tanımlanmış öğelere herhangi bir özel söz dizimi olmadan erişebilirsin. Sadece adlarını kullanırsın.

```rust
struct Ticket {
    // [...]
}

// Burada `Ticket`'ı herhangi bir şekilde nitelemeye gerek yok
// çünkü aynı modüldeyiz
fn mark_ticket_as_done(ticket: Ticket) {
    // [...]
}
```

Farklı bir modüldeki bir varlığa erişmek istiyorsan durum böyle değildir.\
Erişmek istediğin varlığı gösteren bir **yol** kullanman gerekir.

Yolu çeşitli şekillerde oluşturabilirsin:

- geçerli crate'in kökünden başlayarak, ör. `crate::module_1::MyStruct`
- üst modülden başlayarak, ör. `super::my_function`
- geçerli modülden başlayarak, ör. `sub_module_1::MyStruct`

Hem `crate` hem de `super` birer **anahtar kelimedir**.\
`crate` geçerli crate'in köküne, `super` ise geçerli modülün üst modülüne karşılık gelir.

Bir türe her başvurmak istediğinde tam yolu yazmak zahmetli olabilir.
İşini kolaylaştırmak için, varlığı kapsama almak üzere bir `use` deyimi ekleyebilirsin.

```rust
// `MyStruct`'ı kapsama al
use crate::module_1::module_2::MyStruct;

// Artık `MyStruct`'a doğrudan başvurabilirsin
fn a_function(s: MyStruct) {
     // [...]
}
```

### Star Import'lar (Yıldızlı Import'lar) {#star-imports}

Bir modüldeki tüm öğeleri tek bir `use` deyimiyle de import edebilirsin.

```rust
use crate::module_1::module_2::*;
```

Buna **star import** denir.\
Genellikle önerilmez, çünkü geçerli ad alanını kirletebilir; bu da her adın nereden geldiğini anlamayı
zorlaştırır ve ad çakışmalarına yol açabilir.\
Yine de birim testleri yazarken olduğu gibi bazı durumlarda kullanışlı olabilir. Test modüllerimizin çoğunun,
üst modüldeki (test edilen modül) tüm öğeleri kapsama almak için bir `use super::*;` deyimiyle başladığını
fark etmiş olabilirsin.

## Modül Ağacını Görselleştirmek {#visualizing-the-module-tree}

Projenin modül ağacını gözünde canlandırmakta zorlanıyorsan, onu görselleştirmek için
[`cargo-modules`](https://crates.io/crates/cargo-modules) kullanmayı deneyebilirsin!

Kurulum talimatları ve kullanım örnekleri için onların dokümantasyonuna başvur.
