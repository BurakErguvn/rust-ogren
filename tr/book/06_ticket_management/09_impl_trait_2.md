# Argüman Konumunda `impl Trait` {#impl-trait-in-argument-position}

Önceki kısımda, `impl Trait`'in bir türü adını belirtmeden döndürmek için nasıl kullanılabileceğini gördük.\
Aynı söz dizimi **argüman konumunda** da kullanılabilir:

```rust
fn print_iter(iter: impl Iterator<Item = i32>) {
    for i in iter {
        println!("{}", i);
    }
}
```

`print_iter`, `i32`'lerden oluşan bir iterator alır ve her elemanı yazdırır.\
**Argüman konumunda** kullanıldığında `impl Trait`, trait bound'u olan bir generic parametreye eşdeğerdir:

```rust
fn print_iter<T>(iter: T) 
where
    T: Iterator<Item = i32>
{
    for i in iter {
        println!("{}", i);
    }
}
```

## Dezavantajlar {#downsides}

Genel bir kural olarak, argüman konumunda `impl Trait` yerine generic'leri tercih et.\
Generic'ler, çağıranın turbofish söz dizimini (`::<>`) kullanarak argümanın türünü açıkça belirtmesine olanak tanır;
bu da belirsizliği gidermek için faydalı olabilir. `impl Trait` için durum böyle değildir.
