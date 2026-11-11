# Kilitler, `Send` ve `Arc` {#locks-send-and-arc}

Az önce implement ettiğin patch'leme (yamalama) stratejisinin büyük bir dezavantajı var: race condition'a açık.\
İki istemci aynı ticket için kabaca aynı anda patch gönderirse, sunucu onları rastgele bir sırayla uygular.
Patch'ini kuyruğa en son ekleyen, diğer istemcinin yaptığı değişikliklerin üzerine yazar.

## Sürüm Numaraları {#version-numbers}

Bunu bir **sürüm numarası** kullanarak düzeltmeyi deneyebiliriz.\
Her ticket'a oluşturulurken `0` olarak ayarlanan bir sürüm numarası atanır.\
Bir istemci her patch gönderdiğinde, istenen değişikliklerle birlikte ticket'ın güncel sürüm numarasını
da eklemelidir. Sunucu patch'i yalnızca sürüm numarası kendi sakladığıyla eşleşirse uygular.

Yukarıda anlatılan senaryoda sunucu ikinci patch'i reddederdi, çünkü sürüm numarası ilk patch tarafından
artırılmış olurdu ve dolayısıyla ikinci istemcinin gönderdiğiyle eşleşmezdi.

Bu yaklaşım dağıtık sistemlerde oldukça yaygındır (ör. istemci ve sunucuların belleği paylaşmadığı durumlarda)
ve **iyimser eşzamanlılık kontrolü** (optimistic concurrency control) olarak bilinir.\
Fikir şudur: çoğu zaman çakışma olmaz, bu yüzden yaygın durum için optimize edebiliriz.
Artık Rust hakkında, istersen bu stratejiyi bonus bir egzersiz olarak kendi başına implement edecek kadar bilgin var.

## Kilitleme {#locking}

Race condition'ı bir **kilit** getirerek de düzeltebiliriz.\
Bir istemci bir ticket'ı güncellemek istediğinde, önce onun üzerindeki kilidi almalıdır. Kilit etkinken
başka hiçbir istemci ticket'ı değiştiremez.

Rust'ın standart kütüphanesi iki farklı kilitleme primitive'i sunar: `Mutex<T>` ve `RwLock<T>`.\
`Mutex<T>` ile başlayalım. Adı **mut**ual **ex**clusion'dan (karşılıklı dışlama) gelir ve en basit kilit türüdür:
okuma ya da yazma için olmasına bakmaksızın veriye yalnızca bir thread'in (iş parçacığının) erişmesine izin verir.

`Mutex<T>` koruduğu veriyi sarmalar ve dolayısıyla verinin türü üzerinde generic'tir.\
Veriye doğrudan erişemezsin: tür sistemi seni önce `Mutex::lock` ya da
`Mutex::try_lock` kullanarak bir kilit almaya zorlar. İlki kilit alınana kadar bloklar; ikincisi kilit
alınamıyorsa hemen bir hatayla döner.\
Her iki metot da veriye deref edilen ve onu değiştirmeni sağlayan bir guard nesnesi döndürür. Guard drop edildiğinde
kilit bırakılır.

```rust
use std::sync::Mutex;

// Bir mutex kilidiyle korunan bir tam sayı
let lock = Mutex::new(0);

// Mutex üzerinde kilidi al
let mut guard = lock.lock().unwrap();

// Veriyi guard üzerinden değiştir,
// `Deref` implementation'ından yararlanarak
*guard += 1;

// Kilit, `data` kapsam dışına çıktığında bırakılır
// Bu, guard'ı drop ederek açıkça yapılabilir
// ya da guard kapsam dışına çıktığında örtük olarak gerçekleşebilir
drop(guard)
```

## Kilitleme Ayrıntı Düzeyi {#locking-granularity}

`Mutex`'imiz neyi sarmalamalı?\
En basit seçenek, `TicketStore`'un tamamını tek bir `Mutex` içine sarmalamak olurdu.\
Bu işe yarardı, ama sistemin performansını ciddi şekilde kısıtlardı: ticket'ları paralel olarak okuyamazdın,
çünkü her okuma kilidin bırakılmasını beklemek zorunda kalırdı.\
Buna **kaba taneli kilitleme** denir.

Her ticket'ın kendi kilidiyle korunduğu **ince taneli kilitleme** kullanmak daha iyi olurdu.
Bu sayede istemciler, aynı ticket'a erişmeye çalışmadıkları sürece ticket'larla paralel olarak çalışmaya devam edebilir.

```rust
// Her ticket için bir kilit içeren yeni yapı
struct TicketStore {
    tickets: BTreeMap<TicketId, Mutex<Ticket>>,
}
```

Bu yaklaşım daha verimlidir, ama bir dezavantajı vardır: `TicketStore`, sistemin çok thread'li doğasının
**farkında** olmak zorunda kalır; şimdiye kadar `TicketStore` thread'lerin varlığını mutlu bir şekilde görmezden geliyordu.\
Yine de buna girişelim.

## Kilidi Kim Tutuyor? {#who-holds-the-lock}

Tüm planın işlemesi için kilidin, ticket'ı değiştirmek isteyen istemciye verilmesi gerekir.\
İstemci daha sonra ticket'ı doğrudan değiştirebilir (sanki elinde bir `&mut Ticket` varmış gibi) ve işi bittiğinde kilidi bırakabilir.

Bu biraz çetrefilli.\
Bir kanal üzerinden `Mutex<Ticket>` gönderemeyiz, çünkü `Mutex` `Clone` değildir ve
onu `TicketStore`'un dışına move edemeyiz (taşıyamayız). Onun yerine `MutexGuard`'ı gönderebilir miyiz?

Fikri küçük bir örnekle test edelim:

```rust
use std::thread::spawn;
use std::sync::Mutex;
use std::sync::mpsc::sync_channel;

fn main() {
    let lock = Mutex::new(0);
    let (sender, receiver) = sync_channel(1);
    let guard = lock.lock().unwrap();

    spawn(move || {
        receiver.recv().unwrap();
    });

    // Guard'ı kanal üzerinden
    // başka bir thread'e göndermeyi dene
    sender.send(guard);
}
```

Derleyici bu koddan memnun değil:

```text
error[E0277]: `MutexGuard<'_, i32>` cannot be sent between 
              threads safely
   --> src/main.rs:10:7
    |
10  |   spawn(move || {
    |  _-----_^
    | | |
    | | required by a bound introduced by this call
11  | |     receiver.recv().unwrap();
12  | | });
    | |_^ `MutexGuard<'_, i32>` cannot be sent between threads safely
    |
    = help: the trait `Send` is not implemented for 
            `MutexGuard<'_, i32>`, which is required by 
            `{closure@src/main.rs:10:7: 10:14}: Send`
    = note: required for `Receiver<MutexGuard<'_, i32>>` 
            to implement `Send`
note: required because it's used within this closure
```

`MutexGuard<'_, i32>` `Send` değil: bu ne anlama geliyor?

## `Send` {#send}

`Send`, bir türün bir thread'den diğerine güvenli bir şekilde aktarılabileceğini belirten bir marker trait'tir.\
`Send` aynı zamanda `Sized` gibi bir auto trait'tir; derleyici tarafından, türünün tanımına göre
otomatik olarak implement edilir (ya da edilmez).\
`Send`'i kendi türlerin için elle de implement edebilirsin, ama bu `unsafe` gerektirir, çünkü türün
thread'ler arasında gönderilmesinin gerçekten güvenli olduğunu, derleyicinin otomatik olarak doğrulayamayacağı nedenlerle senin garanti etmen gerekir.

### Kanal Gereksinimleri {#channel-requirements}

`Sender<T>`, `SyncSender<T>` ve `Receiver<T>` ancak ve ancak `T` `Send` ise `Send`'dir.\
Bunun nedeni, thread'ler arasında değer göndermek için kullanılmalarıdır; değerin kendisi `Send` değilse,
onu thread'ler arasında göndermek güvenli olmaz.

### `MutexGuard` {#mutexguard}

`MutexGuard` `Send` değildir, çünkü `Mutex`'in kilidi implement etmek için kullandığı, altta yatan işletim sistemi
primitive'leri (bazı platformlarda) kilidin, onu alan thread ile aynı thread tarafından bırakılmasını şart koşar.\
Bir `MutexGuard`'ı başka bir thread'e gönderseydik, kilit farklı bir thread tarafından bırakılırdı; bu da
tanımsız davranışa yol açardı.

## Karşılaştığımız Zorluklar {#our-challenges}

Özetlersek:

- Bir kanal üzerinden `MutexGuard` gönderemeyiz. Yani sunucu tarafında kilitleyip ardından ticket'ı istemci
  tarafında değiştiremeyiz.
- Bir kanal üzerinden `Mutex` gönderebiliriz, çünkü koruduğu veri `Send` olduğu sürece o da `Send`'dir; `Ticket` için
  durum da budur.
  Öte yandan `Mutex`'i `TicketStore`'un dışına move edemeyiz, onu klonlayamayız da.

Bu açmazı nasıl çözebiliriz?\
Soruna farklı bir açıdan bakmamız gerekiyor.
Bir `Mutex`'i kilitlemek için owned (sahipli) bir değere ihtiyacımız yok. Paylaşılan bir referans yeterlidir, çünkü `Mutex` interior mutability (iç değiştirilebilirlik) kullanır:

```rust
impl<T> Mutex<T> {
    // `self` değil, `&self`!
    pub fn lock(&self) -> LockResult<MutexGuard<'_, T>> {
        // Implementation ayrıntıları
    }
}
```

Dolayısıyla istemciye paylaşılan bir referans göndermek yeterlidir.\
Ne var ki bunu doğrudan yapamayız, çünkü referansın `'static` olması gerekirdi ve durum bu değil.\
Bir bakıma "owned bir paylaşılan referansa" ihtiyacımız var. Görünen o ki Rust'ta tam da buna uyan bir tür var: `Arc`.

## İmdada `Arc` Yetişiyor {#arc-to-the-rescue}

`Arc`, **atomic reference counting** (atomik referans sayımı) ifadesinin kısaltmasıdır.\
`Arc` bir değeri sarmalar ve o değere kaç referans olduğunu takip eder.
Son referans drop edildiğinde değer serbest bırakılır.\
Bir `Arc` içine sarmalanmış değer immutable'dır (değiştirilemez): ona yalnızca paylaşılan referanslar alabilirsin.

```rust
use std::sync::Arc;

let data: Arc<u32> = Arc::new(0);
let data_clone = Arc::clone(&data);

// `Arc<T>`, `Deref<T>`'yi implement eder, bu yüzden deref coercion
// kullanarak bir `&Arc<T>`'yi `&T`'ye dönüştürebilir
let data_ref: &u32 = &data;
```

Bir déjà vu anı yaşıyorsan haklısın: `Arc`, interior mutability'den bahsederken tanıttığımız referans sayımlı pointer (işaretçi)
`Rc`'ye çok benziyor. Fark thread güvenliğindedir: `Rc` `Send` değildir, `Arc` ise öyledir.
Bu, referans sayısının nasıl implement edildiğine dayanır: `Rc` "normal" bir tam sayı kullanırken `Arc`,
thread'ler arasında güvenli bir şekilde paylaşılabilen ve değiştirilebilen **atomik** bir tam sayı kullanır.

## `Arc<Mutex<T>>` {#arcmutext}

`Arc`'ı `Mutex` ile eşleştirirsek nihayet şu özelliklere sahip bir tür elde ederiz:

- Thread'ler arasında gönderilebilir, çünkü:
  - `T` `Send` ise `Arc` `Send`'dir ve
  - `T` `Send` ise `Mutex` `Send`'dir.
  - `T`, `Send` olan `Ticket`'tır.
- Klonlanabilir, çünkü `T` ne olursa olsun `Arc` `Clone`'dur.
  Bir `Arc`'ı klonlamak referans sayısını artırır, veri kopyalanmaz.
- Sarmaladığı veriyi değiştirmek için kullanılabilir, çünkü `Arc`, `Mutex<T>`'ye paylaşılan bir
  referans almanı sağlar; bu referans da bir kilit almak için kullanılabilir.

Ticket store'u için kilitleme stratejisini implement etmek için gereken tüm parçalara sahibiz.

## İleri Okuma {#further-reading}

- Bu kursta atomik işlemlerin ayrıntılarına girmeyeceğiz, ama daha fazla bilgiyi
  [`std` dokümantasyonunda](https://doc.rust-lang.org/std/sync/atomic/index.html) ve
  ["Rust atomics and locks" kitabında](https://marabos.nl/atomics/) bulabilirsin.
