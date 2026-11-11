# Ownership (Sahiplik) {#ownership}

Önceki egzersizi bu kursun şimdiye kadar öğrettiklerini kullanarak çözdüysen,
erişim metotların muhtemelen şöyle görünüyordur:

```rust
impl Ticket {
    pub fn title(self) -> String {
        self.title
    }

    pub fn description(self) -> String {
        self.description
    }

    pub fn status(self) -> String {
        self.status
    }
}
```

Bu metotlar derlenir ve testlerin geçmesi için yeterlidir, ama gerçek dünyadaki bir senaryoda seni pek ileri götürmezler.
Şu kod parçasını düşün:

```rust
if ticket.status() == "To-Do" {
    // `println!` makrosunu henüz ele almadık,
    // ama şimdilik konsola (şablonlu) bir mesaj
    // yazdırdığını bilmen yeterli
    println!("Your next task is: {}", ticket.title());
}
```

Derlemeye çalışırsan bir hata alırsın:

```text
error[E0382]: use of moved value: `ticket`
  --> src/main.rs:30:43
   |
25 |     let ticket = Ticket::new(/* */);
   |         ------ move occurs because `ticket` has type `Ticket`, 
   |                which does not implement the `Copy` trait
26 |     if ticket.status() == "To-Do" {
   |               -------- `ticket` moved due to this method call
...
30 |         println!("Your next task is: {}", ticket.title());
   |                                           ^^^^^^ 
   |                                value used here after move
   |
note: `Ticket::status` takes ownership of the receiver `self`, 
      which moves `ticket`
  --> src/main.rs:12:23
   |
12 |         pub fn status(self) -> String {
   |                       ^^^^
```

Tebrikler, bu senin ilk borrow checker hatan!

## Rust'ın Ownership Sisteminin Avantajları {#the-perks-of-rusts-ownership-system}

Rust'ın ownership sistemi şunları garanti etmek için tasarlanmıştır:

- Veri okunurken asla değiştirilmez
- Veri değiştirilirken asla okunmaz
- Veriye, yok edildikten sonra asla erişilmez

Bu kısıtlamalar, Rust derleyicisinin bir alt sistemi olan ve Rust topluluğunda sıklıkla şakalara ve
internet geyiklerine konu olan **borrow checker** tarafından zorunlu kılınır.

Ownership, Rust'ta temel bir kavramdır ve dili eşsiz kılan şeydir.
Ownership, Rust'ın **performanstan ödün vermeden bellek güvenliği** sağlamasına olanak tanır.
Rust için şunların hepsi aynı anda geçerlidir:

1. Çalışma zamanında çalışan bir garbage collector (çöp toplayıcı) yoktur
2. Bir geliştirici olarak belleği nadiren doğrudan yönetmen gerekir
3. Dangling pointer'lara (sarkan pointer'lara), double free'lere ve bellekle ilgili diğer bug'lara yol açamazsın

Python, JavaScript ve Java gibi diller sana 2. ve 3. maddeleri sağlar, ama 1. maddeyi sağlamaz.\
C ya da C++ gibi diller sana 1. maddeyi sağlar, ama ne 2. ne de 3. maddeyi sağlar.

Geçmişine bağlı olarak 3. madde biraz gizemli gelebilir: "dangling pointer" nedir?
"Double free" nedir? Neden tehlikelidirler?\
Endişelenme: bu kavramları kursun geri kalanında daha ayrıntılı ele alacağız.

Ama şimdilik, Rust'ın ownership sistemi içinde nasıl çalışılacağını öğrenmeye odaklanalım.

## Sahip {#the-owner}

Rust'ta her değerin, derleme zamanında statik olarak belirlenen bir **sahibi** vardır.
Herhangi bir anda her değerin yalnızca tek bir sahibi olur.

## Move Semantics (Taşıma Semantiği) {#move-semantics}

Ownership devredilebilir.

Örneğin bir değere sahipsen, ownership'i başka bir değişkene devredebilirsin:

```rust
let a = "hello, world".to_string(); // <- String'in sahibi `a`
let b = a;  // <- String'in sahibi artık `b`
```

Rust'ın ownership sistemi tür sistemine gömülüdür: her fonksiyon, argümanlarıyla _nasıl_ etkileşim kurmak
istediğini imzasında bildirmek zorundadır.

Şimdiye kadar tüm metotlarımız ve fonksiyonlarımız argümanlarını **tüketti**: onların ownership'ini aldılar.
Örneğin:

```rust
impl Ticket {
    pub fn description(self) -> String {
        self.description
    }
}
```

`Ticket::description`, üzerinde çağrıldığı `Ticket` instance'ının (örneğinin) ownership'ini alır.\
Buna **move semantics** denir: değerin (`self`) ownership'i çağırandan çağrılana **move edilir** (taşınır)
ve çağıran onu artık kullanamaz.

Bu, daha önce gördüğümüz hata mesajında derleyicinin kullandığı dilin ta kendisi:

```text
error[E0382]: use of moved value: `ticket`
  --> src/main.rs:30:43
   |
25 |     let ticket = Ticket::new(/* */);
   |         ------ move occurs because `ticket` has type `Ticket`, 
   |                which does not implement the `Copy` trait
26 |     if ticket.status() == "To-Do" {
   |               -------- `ticket` moved due to this method call
...
30 |         println!("Your next task is: {}", ticket.title());
   |                                           ^^^^^^ 
   |                                 value used here after move
   |
note: `Ticket::status` takes ownership of the receiver `self`, 
      which moves `ticket`
  --> src/main.rs:12:23
   |
12 |         pub fn status(self) -> String {
   |                       ^^^^
```

Özellikle, `ticket.status()` çağrıldığında gerçekleşen olayların sırası şöyledir:

- `Ticket::status`, `Ticket` instance'ının ownership'ini alır
- `Ticket::status`, `self`'ten `status`'u çıkarır ve `status`'un ownership'ini çağırana geri devreder
- `Ticket` instance'ının geri kalanı (`title` ve `description`) atılır

`ticket`'ı `ticket.title()` aracılığıyla yeniden kullanmaya çalıştığımızda derleyici şikâyet eder: `ticket` değeri artık yok,
ona artık sahip değiliz, bu yüzden onu artık kullanamayız.

_Kullanışlı_ erişim metotları oluşturmak için **referanslarla** çalışmaya başlamamız gerekiyor.

## Borrowing (Ödünç Alma) {#borrowing}

Bir değişkenin değerini, ownership'ini almadan okuyabilen metotlara sahip olmak istenir.\
Aksi halde programlama oldukça kısıtlı olurdu. Rust'ta bu, **borrowing** aracılığıyla yapılır.

Bir değeri her ödünç aldığında, ona bir **referans** elde edersin.\
Referanslar ayrıcalıklarıyla etiketlenir[^refine]:

- Immutable (değiştirilemez) referanslar (`&`) değeri okumana izin verir, ama değiştirmene izin vermez
- Mutable (değiştirilebilir) referanslar (`&mut`) değeri okumana ve değiştirmene izin verir

Rust'ın ownership sisteminin hedeflerine dönersek:

- Veri okunurken asla değiştirilmez
- Veri değiştirilirken asla okunmaz

Bu iki özelliği garanti etmek için Rust'ın referanslara bazı kısıtlamalar getirmesi gerekir:

- Aynı değere aynı anda hem mutable hem de immutable bir referansın olamaz
- Aynı değere aynı anda birden fazla mutable referansın olamaz
- Değer ödünç alınmışken sahibi onu değiştiremez
- Mutable referans olmadığı sürece istediğin kadar immutable referansın olabilir

Bir bakıma, immutable bir referansı değer üzerindeki "salt okunur" bir kilit gibi,
mutable bir referansı ise "okuma-yazma" kilidi gibi düşünebilirsin.

Bu kısıtlamaların hepsi derleme zamanında borrow checker tarafından zorunlu kılınır.

### Söz Dizimi {#syntax}

Pratikte bir değeri nasıl ödünç alırsın?\
Bir **değişkenin önüne** `&` ya da `&mut` ekleyerek onun değerini ödünç alırsın.
Yine de dikkatli ol! Aynı semboller (`&` ve `&mut`) bir **türün önünde** farklı bir anlama gelir:
farklı bir türü, yani orijinal türe bir referansı belirtirler.

Örneğin:

```rust
struct Configuration {
    version: u32,
    active: bool,
}

fn main() {
    let config = Configuration {
        version: 1,
        active: true,
    };
    // `b`, `config`'in `version` alanına bir referanstır.
    // `b`'nin türü `&u32`'dir, çünkü bir `u32` değerine
    // referans içerir.
    // `&` operatörünü kullanarak `config.version`'ı ödünç alıp
    // bir referans oluşturuyoruz.
    // Aynı sembol (`&`), bağlama göre farklı anlam!
    let b: &u32 = &config.version;
    //     ^ Tür belirtimi gerekli değil,
    //       sadece neler olup bittiğini açıklamak için orada
}
```

Aynı kavram fonksiyon argümanları ve dönüş türleri için de geçerlidir:

```rust
// `f`, argüman olarak bir `u32`'ye mutable bir referans alır;
// bu referans `number` adına bağlanır
fn f(number: &mut u32) -> &u32 {
    // [...]
}
```

## Nefes Al, Nefes Ver {#breathe-in-breathe-out}

Rust'ın ownership sistemi başta biraz bunaltıcı olabilir.\
Ama endişelenme: pratik yaptıkça sana doğal gelmeye başlayacak.\
Ve bu bölümün geri kalanında, hatta kursun geri kalanında bolca pratik yapacaksın!
Her kavrama birden fazla kez geri döneceğiz; böylece onlara aşina olmanı ve nasıl çalıştıklarını
gerçekten anlamanı sağlayacağız.

Bu bölümün sonlarına doğru, Rust'ın ownership sisteminin _neden_ bu şekilde tasarlandığını açıklayacağız.
Şimdilik _nasıl_ kısmını anlamaya odaklan. Her derleyici hatasını bir öğrenme fırsatı olarak gör!

[^refine]: Bu, başlangıç için harika bir zihinsel model, ama resmin _tamamını_ yansıtmıyor.
Referanslarla ilgili anlayışımızı [kursun ilerleyen kısımlarında](../07_threads/06_interior_mutability.md) daha hassas hale getireceğiz.
