# Scoped Thread'ler {#scoped-threads}

Şimdiye kadar tartıştığımız tüm lifetime (yaşam süresi) sorunlarının ortak bir kaynağı var:
spawn edilen thread (iş parçacığı) üst thread'inden daha uzun yaşayabilir.\
Bu sorunu **scoped thread'ler** kullanarak aşabiliriz.

```rust
let v = vec![1, 2, 3];
let midpoint = v.len() / 2;

std::thread::scope(|scope| {
    scope.spawn(|| {
        let first = &v[..midpoint];
        println!("Here's the first half of v: {first:?}");
    });
    scope.spawn(|| {
        let second = &v[midpoint..];
        println!("Here's the second half of v: {second:?}");
    });
});

println!("Here's v: {v:?}");
```

Neler olduğunu açalım.

## `scope` {#scope}

`std::thread::scope` fonksiyonu yeni bir **kapsam** oluşturur.\
`std::thread::scope` girdi olarak tek argümanlı bir closure alır: bu argüman bir `Scope` instance'ıdır (örnek).

## Scoped Spawn'lar {#scoped-spawns}

`Scope` bir `spawn` metodu sunar.\
`std::thread::spawn`'dan farklı olarak, bir `Scope` kullanılarak spawn edilen tüm thread'ler, kapsam sona erdiğinde
**otomatik olarak join edilir**.

Önceki örneği `std::thread::spawn`'a "çevirseydik",
şöyle görünürdü:

```rust
let v = vec![1, 2, 3];
let midpoint = v.len() / 2;

let handle1 = std::thread::spawn(|| {
    let first = &v[..midpoint];
    println!("Here's the first half of v: {first:?}");
});
let handle2 = std::thread::spawn(|| {
    let second = &v[midpoint..];
    println!("Here's the second half of v: {second:?}");
});

handle1.join().unwrap();
handle2.join().unwrap();

println!("Here's v: {v:?}");
```

## Ortamdan Ödünç Alma {#borrowing-from-the-environment}

Ne var ki çevrilmiş örnek derlenmez: derleyici, `&v`'nin lifetime'ı
`'static` olmadığı için spawn edilen thread'lerimizden kullanılamayacağından
yakınır.

`std::thread::scope` ile bu bir sorun değildir—**ortamdan güvenle ödünç alabilirsin**.

Örneğimizde `v`, spawn noktalarından önce oluşturulur.
Yalnızca `scope` döndükten _sonra_ drop edilir. Öte yandan,
`scope` içinde spawn edilen tüm thread'lerin `scope` dönmeden _önce_ bitmesi garantidir;
dolayısıyla dangling referanslar (sarkan referanslar) oluşması riski yoktur.

Derleyici yakınmayacak!
