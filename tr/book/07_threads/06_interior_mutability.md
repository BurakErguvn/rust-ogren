# Interior Mutability (İç Değiştirilebilirlik) {#interior-mutability}

Bir an durup `Sender`'ın `send` metodunun imzası üzerine düşünelim:

```rust
impl<T> Sender<T> {
    pub fn send(&self, t: T) -> Result<(), SendError<T>> {
        // [...]
    }
}
```

`send` argüman olarak `&self` alır.\
Ama açıkça bir değişikliğe yol açıyor: kanala yeni bir mesaj ekliyor.
Daha da ilginci, `Sender` klonlanabilir: farklı thread'lerden (iş parçacıklarından) kanalın durumunu **aynı anda**
değiştirmeye çalışan birden fazla `Sender` instance'ımız (örnek) olabilir.

Bu istemci-sunucu mimarisini kurmak için kullandığımız temel özellik bu. Ama neden çalışıyor?
Rust'ın borrowing (ödünç alma) kurallarını ihlal etmiyor mu? _Immutable_ (değiştirilemez) bir referans üzerinden nasıl değişiklik yapıyoruz?

## Immutable Değil, Paylaşılan Referanslar {#shared-rather-than-immutable-references}

Borrow checker'ı tanıttığımızda, Rust'ta sahip olabileceğimiz iki referans türünü şöyle adlandırmıştık:

- immutable referanslar (`&T`)
- mutable (değiştirilebilir) referanslar (`&mut T`)

Onları şöyle adlandırmak daha doğru olurdu:

- paylaşılan referanslar (`&T`)
- dışlayıcı referanslar (`&mut T`)

Immutable/mutable, vakaların büyük çoğunluğunda işe yarayan bir zihinsel modeldir ve Rust'a başlamak için
harika bir modeldir. Ama az önce gördüğün gibi hikâyenin tamamı bu değil: `&T`, gösterdiği verinin
immutable olduğunu aslında garanti etmez.\
Yine de endişelenme: Rust hâlâ vaatlerini tutuyor.
Sadece terimler ilk bakışta göründüğünden biraz daha incelikli.

## `UnsafeCell` {#unsafecell}

Bir tür, paylaşılan bir referans üzerinden veriyi değiştirmene izin veriyorsa, **interior mutability** ile uğraşıyorsun demektir.

Varsayılan olarak Rust derleyicisi paylaşılan referansların immutable olduğunu varsayar. Bu varsayıma dayanarak **kodunu optimize eder**.\
Derleyici, kodunu daha hızlı hale getirmek için işlemlerin sırasını değiştirebilir, değerleri önbelleğe alabilir ve her türlü sihri yapabilir.

Veriyi bir `UnsafeCell` içine sarmalayarak derleyiciye "Hayır, bu paylaşılan referans aslında mutable" diyebilirsin.\
Interior mutability'ye izin veren bir tür gördüğün her seferde, doğrudan ya da dolaylı olarak
`UnsafeCell`'in işin içinde olduğundan emin olabilirsin.\
`UnsafeCell`, raw pointer'lar (ham pointer'lar) ve `unsafe` kod kullanarak paylaşılan referanslar üzerinden veriyi değiştirebilirsin.

Yine de açık olalım: `UnsafeCell`, borrow checker'ı görmezden gelmeni sağlayan sihirli bir değnek değildir!\
`unsafe` kod da Rust'ın borrowing ve aliasing kurallarına tabidir.
Güvenliği Rust'ın tür sisteminde doğrudan ifade edilemeyen **güvenli soyutlamalar** oluşturmak için yararlanabileceğin
(ileri düzey) bir araçtır. `unsafe` anahtar kelimesini her kullandığında derleyiciye şunu söylüyorsun:
"Ne yaptığımı biliyorum, invariant'larını (değişmezlerini) ihlal etmeyeceğim, bana güven."

Bir `unsafe` fonksiyonu her çağırdığında, onun **güvenlik ön koşullarını** açıklayan bir dokümantasyon olacaktır:
yani hangi koşullar altında `unsafe` bloğunu çalıştırmanın güvenli olduğu. `UnsafeCell` için olanları
[`std`'nin dokümantasyonunda](https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html) bulabilirsin.

Bu kursta `UnsafeCell`'i doğrudan kullanmayacağız, `unsafe` kod da yazmayacağız.
Ama orada olduğunu, neden var olduğunu ve Rust'ta her gün kullandığın türlerle nasıl ilişkili olduğunu
bilmek önemli.

## Önemli Örnekler {#key-examples}

Interior mutability'den yararlanan birkaç önemli `std` türüne göz atalım.\
Bunlar Rust kodunda oldukça sık karşılaşacağın türlerdir, özellikle de kullandığın bazı kütüphanelerin
kaputunun altına göz atarsan.

### Referans Sayımı {#reference-counting}

`Rc`, referans sayımlı bir pointer'dır.\
Bir değeri sarmalar ve o değere kaç referans olduğunu takip eder.
Son referans drop edildiğinde değer serbest bırakılır.\
Bir `Rc` içine sarmalanmış değer immutable'dır: ona yalnızca paylaşılan referanslar alabilirsin.

```rust
use std::rc::Rc;

let a: Rc<String> = Rc::new("My string".to_string());
// String verisine yalnızca bir referans var.
assert_eq!(Rc::strong_count(&a), 1);

// `clone` çağırdığımızda string verisi kopyalanmaz!
// Bunun yerine `Rc`'nin referans sayısı artırılır.
let b = Rc::clone(&a);
assert_eq!(Rc::strong_count(&a), 2);
assert_eq!(Rc::strong_count(&b), 2);
// ^ Hem `a` hem `b` aynı string verisini gösterir
//   ve aynı referans sayacını paylaşır.
```

`Rc`, paylaşılan referansların referans sayısını artırıp azaltabilmesi için içeride `UnsafeCell` kullanır.

### `RefCell` {#refcell}

`RefCell`, Rust'taki en yaygın interior mutability örneklerinden biridir.
Elinde `RefCell`'in kendisine yalnızca immutable bir referans olsa bile, `RefCell` içine sarmalanmış
değeri değiştirmene izin verir.

Bu, **çalışma zamanında borrow checking** ile yapılır.
`RefCell`, içerdiği değere yönelik referansların sayısını (ve türünü) çalışma zamanında takip eder.
Değer zaten immutable olarak ödünç alınmışken onu mutable olarak ödünç almaya çalışırsan
program panic'ler; böylece Rust'ın ödünç alma kurallarının her zaman uygulanması sağlanır.

```rust
use std::cell::RefCell;

let x = RefCell::new(42);

let y = x.borrow(); // Immutable ödünç alma
let z = x.borrow_mut(); // Panic'ler! Etkin bir immutable ödünç alma var.
```
