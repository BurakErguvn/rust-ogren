# `Sync` {#sync}

Bu bölümü bitirmeden önce Rust'ın standart kütüphanesindeki bir başka önemli trait'ten bahsedelim: `Sync`.

`Sync`, tıpkı `Send` gibi bir auto trait'tir.\
Thread'ler (iş parçacıkları) arasında güvenli bir şekilde **paylaşılabilen** tüm türler tarafından otomatik olarak implement edilir.

Başka bir deyişle: `&T` `Send` ise `T` Sync'tir.

## `T: Sync`, `T: Send` Anlamına Gelmez {#t-sync-doesnt-imply-t-send}

`T`'nin `Send` olmadan `Sync` olabileceğini belirtmek önemlidir.\
Örneğin: `MutexGuard` `Send` değildir, ama `Sync`'tir.

`Send` değildir, çünkü kilit onu alan thread üzerinde bırakılmalıdır; dolayısıyla `MutexGuard`'ın farklı bir
thread üzerinde drop edilmesini istemeyiz.\
Ama `Sync`'tir, çünkü bir `&MutexGuard`'ı başka bir thread'e vermenin kilidin nerede bırakılacağı üzerinde hiçbir etkisi yoktur.

## `T: Send`, `T: Sync` Anlamına Gelmez {#t-send-doesnt-imply-t-sync}

Tersi de doğrudur: `T`, `Sync` olmadan `Send` olabilir.\
Örneğin: `RefCell<T>` `Send`'dir (`T` `Send` ise), ama `Sync` değildir.

`RefCell<T>` çalışma zamanında borrow checking yapar, ama ödünç almaları takip etmek için kullandığı sayaçlar thread-safe değildir.
Bu yüzden birden fazla thread'in bir `&RefCell` tutması bir data race'e (veri yarışı) yol açar ve potansiyel olarak
birden fazla thread aynı veriye mutable (değiştirilebilir) referanslar elde eder. Bu nedenle `RefCell` `Sync` değildir.\
Öte yandan `Send` sorun değildir, çünkü bir `RefCell`'i başka bir thread'e gönderdiğimizde, içerdiği veriye yönelik
geride hiçbir referans bırakmayız; dolayısıyla eşzamanlı mutable erişim riski yoktur.
