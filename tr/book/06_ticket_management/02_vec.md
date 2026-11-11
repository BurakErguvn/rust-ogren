# Vektörler {#vectors}

Dizilerin gücü aynı zamanda zayıflıklarıdır: boyutlarının önceden, derleme zamanında bilinmesi gerekir.
Boyutu yalnızca çalışma zamanında bilinen bir dizi oluşturmaya çalışırsan derleme hatası alırsın:

```rust
let n = 10;
let numbers: [u32; n];
```

```text
error[E0435]: attempt to use a non-constant value in a constant
 --> src/main.rs:3:20
  |
2 | let n = 10;
3 | let numbers: [u32; n];
  |                    ^ non-constant value
```

Diziler ticket yönetim sistemimiz için işe yaramaz—derleme zamanında kaç ticket saklamamız gerekeceğini bilmiyoruz.
İşte burada `Vec` devreye girer.

## `Vec` {#vec}

`Vec`, standart kütüphane tarafından sağlanan, büyüyebilen bir dizi türüdür.\
`Vec::new` fonksiyonunu kullanarak boş bir vektör oluşturabilirsin:

```rust
let mut numbers: Vec<u32> = Vec::new();
```

Ardından `push` metodunu kullanarak vektöre eleman eklersin:

```rust
numbers.push(1);
numbers.push(2);
numbers.push(3);
```

Yeni değerler vektörün sonuna eklenir.\
Değerleri oluşturma anında biliyorsan, `vec!` makrosunu kullanarak ilk değerleri atanmış bir vektör de oluşturabilirsin:

```rust
let numbers = vec![1, 2, 3];
```

## Elemanlara Erişmek {#accessing-elements}

Elemanlara erişim söz dizimi dizilerdekiyle aynıdır:

```rust
let numbers = vec![1, 2, 3];
let first = numbers[0];
let second = numbers[1];
let third = numbers[2];
```

İndeks `usize` türünde olmalıdır.\
`Option<&T>` döndüren `get` metodunu da kullanabilirsin:

```rust
let numbers = vec![1, 2, 3];
assert_eq!(numbers.get(0), Some(&1));
// Sınırların dışındaki bir indekse erişmeye çalışırsan
// panic yerine bir `None` alırsın.
assert_eq!(numbers.get(3), None);
```

Erişimin sınırları, tıpkı dizilerdeki eleman erişiminde olduğu gibi denetlenir. Erişim O(1) karmaşıklığa sahiptir.

## Bellek Düzeni {#memory-layout}

`Vec`, heap'te ayrılmış bir veri yapısıdır.\
Bir `Vec` oluşturduğunda, elemanları saklamak için heap'te bellek ayırır.

Aşağıdaki kodu çalıştırırsan:

```rust
let mut numbers = Vec::with_capacity(3);
numbers.push(1);
numbers.push(2);
```

şu bellek düzenini elde edersin:

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

`Vec` üç şeyi takip eder:

- Ayırdığın heap bölgesini gösteren **pointer** (işaretçi).
- Vektörün **uzunluğu**, yani vektörde kaç eleman olduğu.
- Vektörün **kapasitesi**, yani heap'te ayrılan alana sığabilecek eleman sayısı.

Bu düzen tanıdık gelmeli: `String` ile tamamen aynı!\
Bu bir tesadüf değil: `String`, perde arkasında bir byte vektörü, yani `Vec<u8>` olarak tanımlanmıştır:

```rust
pub struct String {
    vec: Vec<u8>,
}
```
