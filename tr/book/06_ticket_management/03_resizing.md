# Yeniden Boyutlandırma {#resizing}

`Vec`'in "büyüyebilen" bir vektör türü olduğunu söyledik, ama bu ne anlama geliyor?
Zaten maksimum kapasitesine ulaşmış bir `Vec`'e eleman eklemeye çalışırsan ne olur?

```rust
let mut numbers = Vec::with_capacity(3);
numbers.push(1);
numbers.push(2);
numbers.push(3); // Maksimum kapasiteye ulaşıldı
numbers.push(4); // Burada ne olur?
```

`Vec` kendini **yeniden boyutlandırır**.\
Allocator'dan (bellek ayırıcıdan) yeni (daha büyük) bir heap belleği parçası ister, elemanları oraya kopyalar ve eski belleği serbest bırakır.

Bu işlem pahalı olabilir, çünkü yeni bir allocation (bellek ayırma) yapılmasını ve mevcut tüm elemanların kopyalanmasını gerektirir.

## `Vec::with_capacity` {#vecwith_capacity}

Bir `Vec`'te kaç eleman saklayacağın hakkında kabaca bir fikrin varsa, önceden yeterli bellek ayırmak için
`Vec::with_capacity` metodunu kullanabilirsin.\
Bu, `Vec` büyüdüğünde yeni bir allocation yapılmasını önleyebilir, ama gerçek kullanımı olduğundan fazla tahmin edersen bellek israf edebilir.

Duruma göre değerlendir.
