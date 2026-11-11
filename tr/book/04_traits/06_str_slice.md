# String Slice'lar {#string-slices}

Önceki bölümler boyunca kodda `"To-Do"` ya da `"A ticket description"` gibi epeyce **string literal**
kullanıldığını gördün.
Bunların ardından her zaman bir `.to_string()` ya da `.into()` çağrısı geliyordu. Nedenini anlama zamanı!

## String Literal'ler {#string-literals}

Bir string literal'i, ham metni çift tırnak içine alarak tanımlarsın:

```rust
let s = "Hello, world!";
```

`s`'in türü `&str`; yani bir **string slice**'a (dilime) **referans**.

## Bellek Düzeni {#memory-layout}

`&str` ve `String` farklı türlerdir—birbirlerinin yerine kullanılamazlar.\
Bir `String`'in bellek düzenini
[önceki incelememizden](../03_ticket_v1/09_heap.md) hatırlayalım.
Şunu çalıştırırsak:

```rust
let mut s = String::with_capacity(5);
s.push_str("Hello");
```

bellekte şu senaryoyu elde ederiz:

```text
      +---------+--------+----------+
Stack | pointer | length | capacity | 
      |  |      |   5    |    5     |
      +--|------+--------+----------+
         |
         |
         v
       +---+---+---+---+---+
Heap:  | H | e | l | l | o |
       +---+---+---+---+---+
```

Hatırlarsan, bir `&String`'in bellekte nasıl yerleştiğini de
[incelemiştik](../03_ticket_v1/10_references_in_memory.md):

```text
     --------------------------------------
     |                                    |         
+----v----+--------+----------+      +----|----+
| pointer | length | capacity |      | pointer |
|    |    |   5    |    5     |      |         |
+----|----+--------+----------+      +---------+
     |        s                          &s 
     |       
     v       
   +---+---+---+---+---+
   | H | e | l | l | o |
   +---+---+---+---+---+
```

`&String`, `String`'in meta verisinin saklandığı bellek konumunu gösterir.\
Pointer'ı (işaretçiyi) takip edersek heap'te ayrılmış veriye ulaşırız. Daha açık söylemek gerekirse, string'in ilk byte'ına, yani `H`'ye ulaşırız.

Peki `s`'in bir **alt string**'ini temsil eden bir tür isteseydik? Örneğin `Hello` içindeki `ello`?

## String Slice'lar {#string-slices-1}

Bir `&str`, bir string'e açılan bir **görünüm**; başka bir yerde saklanan bir UTF-8 byte serisine bir **referans**tır.
Örneğin bir `String`'den şöyle bir `&str` oluşturabilirsin:

```rust
let mut s = String::with_capacity(5);
s.push_str("Hello");
// `String`'den bir string slice referansı oluştur,
// ilk byte'ı atlayarak.
let slice: &str = &s[1..];
```

Bellekte şöyle görünür:

```text
                    s                              slice
      +---------+--------+----------+      +---------+--------+
Stack | pointer | length | capacity |      | pointer | length |
      |    |    |   5    |    5     |      |    |    |   4    |
      +----|----+--------+----------+      +----|----+--------+
           |        s                           |  
           |                                    |
           v                                    | 
         +---+---+---+---+---+                  |
Heap:    | H | e | l | l | o |                  |
         +---+---+---+---+---+                  |
               ^                                |
               |                                |
               +--------------------------------+
```

`slice`, stack'te iki bilgi saklar:

- Slice'ın ilk byte'ına bir pointer.
- Slice'ın uzunluğu.

`slice` verinin sahibi değildir, yalnızca onu gösterir: `String`'in heap'te ayrılmış verisine bir **referans**tır.\
`slice` drop edildiğinde heap'te ayrılmış veri serbest bırakılmaz, çünkü verinin sahibi hâlâ `s`'tir.
`slice`'ın bir `capacity` alanı olmamasının nedeni de budur: verinin sahibi değildir, bu yüzden veri için
ne kadar alan ayrıldığını bilmesine gerek yoktur; yalnızca referans verdiği veriyle ilgilenir.

## `&str` Ve `&String` {#str-vs-string}

Genel bir kural olarak, metinsel veriye bir referansa ihtiyaç duyduğunda `&String` yerine `&str` kullan.\
`&str` daha esnektir ve Rust kodunda genellikle daha idiomatik kabul edilir.

Bir metot `&String` döndürüyorsa, bir yerlerde referansını döndürdüğün metinle **tam olarak eşleşen**,
heap'te ayrılmış bir UTF-8 metin bulunduğunu vaat etmiş olursun.\
Bir metot `&str` döndürüyorsa ise çok daha fazla özgürlüğün olur: yalnızca _bir yerlerde_ bir miktar
metin verisi bulunduğunu ve bunun bir alt kümesinin ihtiyacın olanla eşleştiğini, bu yüzden ona bir referans döndürdüğünü söylemiş olursun.
