# Async Rust {#async-rust}

Rust'ta eşzamanlı programlar yazmanın tek yolu thread'ler (iş parçacıkları) değil.\
Bu bölümde başka bir yaklaşımı keşfedeceğiz: **asenkron programlama**.

Özellikle şunlarla tanışacaksın:

- Zahmetsizce asenkron kod yazmak için `async`/`.await` anahtar kelimeleri
- Henüz tamamlanmamış olabilecek hesaplamaları temsil etmek için `Future` trait'i
- Asenkron kod çalıştırmak için en popüler runtime olan `tokio`
- Rust'ın asenkron modelinin iş birliğine dayalı doğası ve bunun kodunu nasıl etkilediği
