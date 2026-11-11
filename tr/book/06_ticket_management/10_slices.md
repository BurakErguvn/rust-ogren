# Slice'lar (Dilimler) {#slices}

Bir `Vec`'in bellek düzenine geri dönelim:

```rust
let mut numbers = Vec::with_capacity(3);
numbers.push(1);
numbers.push(2);
```

```text
      +---------+--------+----------+
Stack | pointer | length | capacity | 
      |  |      |   2    |    3     |
      +--|------+--------+----------+
         |
         |
         v
       +---+---+---+
Heap:  | 1 | 2 | ? |
       +---+---+---+
```

`String`'in kılık değiştirmiş bir `Vec<u8>`'den ibaret olduğunu zaten belirtmiştik.\
Bu benzerlik, aklına şu soruyu getirmeli: "`Vec` için `&str`'nin karşılığı nedir?"

## `&[T]` {#t}

`[T]`, `T` türündeki elemanlardan oluşan bitişik bir serinin bir **slice**'ıdır.\
En çok, ödünç alınmış biçimi olan `&[T]` şeklinde kullanılır.

Bir `Vec`'ten slice referansı oluşturmanın çeşitli yolları vardır:

```rust
let numbers = vec![1, 2, 3];
// İndeks söz dizimiyle
let slice: &[i32] = &numbers[..];
// Bir metot ile
let slice: &[i32] = numbers.as_slice();
// Ya da elemanların bir alt kümesi için
let slice: &[i32] = &numbers[1..];
```

`Vec`, hedef tür olarak `[T]`'yi kullanarak `Deref` trait'ini implement eder; böylece slice metotlarını deref coercion
sayesinde doğrudan bir `Vec` üzerinde kullanabilirsin:

```rust
let numbers = vec![1, 2, 3];
// Sürpriz, sürpriz: `iter`, `Vec` üzerinde tanımlı bir metot değil!
// `&[T]` üzerinde tanımlı bir metot, ama deref coercion sayesinde
// onu bir `Vec` üzerinde çağırabilirsin.
let sum: i32 = numbers.iter().sum();
```

### Bellek Düzeni {#memory-layout}

Bir `&[T]`, tıpkı `&str` gibi bir **fat pointer**'dır.\
Slice'ın ilk elemanını gösteren bir pointer'dan (işaretçiden) ve slice'ın uzunluğundan oluşur.

Üç elemanlı bir `Vec`'in varsa:

```rust
let numbers = vec![1, 2, 3];
```

ve ardından bir slice referansı oluşturursan:

```rust
let slice: &[i32] = &numbers[1..];
```

şu bellek düzenini elde edersin:

```text
                  numbers                          slice
      +---------+--------+----------+      +---------+--------+
Stack | pointer | length | capacity |      | pointer | length |
      |    |    |   3    |    4     |      |    |    |   2    |
      +----|----+--------+----------+      +----|----+--------+
           |                                    |  
           |                                    |
           v                                    | 
         +---+---+---+---+                      |
Heap:    | 1 | 2 | 3 | ? |                      |
         +---+---+---+---+                      |
               ^                                |
               |                                |
               +--------------------------------+
```

### `&Vec<T>` Ya Da `&[T]` {#vect-vs-t}

Bir fonksiyona bir `Vec`'e ait immutable (değiştirilemez) bir referans geçirmen gerektiğinde, `&Vec<T>` yerine `&[T]`'yi tercih et.\
Bu, fonksiyonun, arkasında mutlaka bir `Vec` bulunması gerekmeyen her türlü slice'ı kabul etmesine olanak tanır.

Örneğin, o zaman bir `Vec`'teki elemanların bir alt kümesini geçirebilirsin.
Ama iş bununla da bitmiyor—bir **dizi slice'ı** da geçirebilirsin:

```rust
let array = [1, 2, 3];
let slice: &[i32] = &array;
```

Dizi slice'ları ve `Vec` slice'ları aynı türdür: bitişik bir eleman serisini gösteren fat pointer'lardır.
Diziler söz konusu olduğunda pointer heap yerine stack'i gösterir, ama slice'ı kullanmak açısından
bunun bir önemi yoktur.
