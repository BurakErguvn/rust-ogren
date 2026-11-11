# Heap {#heap}

Stack harika, ama tüm sorunlarımızı çözemez. Boyutu derleme zamanında bilinmeyen veriler ne olacak?
Koleksiyonlar, string'ler ve diğer dinamik boyutlu veriler (tamamen) stack'te ayrılamaz.
İşte burada **heap** devreye girer.

## Heap Allocation'ları (Bellek Ayırma) {#heap-allocations}

Heap'i büyük bir bellek parçası—istersen devasa bir dizi—olarak gözünde canlandırabilirsin.\
Heap'te veri depolaman gerektiğinde, özel bir programdan, yani **allocator**'dan (bellek ayırıcı), senin için
heap'in bir alt kümesini ayırmasını istersin. Bu etkileşime (ve ayırdığın belleğe) **heap allocation** deriz.
Allocation başarılı olursa, allocator sana ayrılan bloğun başlangıcına işaret eden bir **pointer** (işaretçi) verir.

## Otomatik Serbest Bırakma Yok {#no-automatic-de-allocation}

Heap, stack'ten oldukça farklı bir yapıya sahiptir.\
Heap allocation'ları bitişik değildir; heap'in içinde herhangi bir yerde bulunabilirler.

```
+---+---+---+---+---+---+-...-+-...-+---+---+---+---+---+---+---+
|  Allocation 1 | Free  | ... | ... |  Allocation N |    Free   |
+---+---+---+---+---+---+ ... + ... +---+---+---+---+---+---+---+
```

Heap'in hangi bölümlerinin kullanımda, hangilerinin boş olduğunu takip etmek allocator'ın işidir.
Ancak allocator, ayırdığın belleği otomatik olarak serbest bırakmaz: bu konuda bilinçli davranman,
artık ihtiyaç duymadığın belleği **serbest bırakmak** için allocator'ı yeniden çağırman gerekir.

## Performans {#performance}

Heap'in esnekliğinin bir bedeli vardır: heap allocation'ları stack allocation'larından **daha yavaştır**.
Çok daha fazla kayıt tutma işi gerektirir!\
Performans optimizasyonu hakkında makaleler okursan, sıklıkla heap allocation'larını en aza indirmen
ve mümkün olduğunda stack'te ayrılmış verileri tercih etmen tavsiye edilir.

## `String`'in Bellek Düzeni {#strings-memory-layout}

`String` türünde yerel bir değişken oluşturduğunda,
Rust heap'te bellek ayırmak zorunda kalır[^empty]: içine ne kadar metin koyacağını önceden bilmez,
bu yüzden stack'te doğru miktarda yer ayıramaz.\
Ama bir `String` _tamamen_ heap'te ayrılmış değildir; bazı verileri stack'te de tutar. Özellikle:

- Ayırdığın heap bölgesine işaret eden **pointer**.
- String'in **uzunluğu**, yani string'de kaç byte olduğu.
- String'in **kapasitesi**, yani heap'te kaç byte ayrıldığı.

Bunu daha iyi anlamak için bir örneğe bakalım:

```rust
let mut s = String::with_capacity(5);
```

Bu kodu çalıştırırsan bellek şöyle düzenlenir:

```
      +---------+--------+----------+
Stack | pointer | length | capacity | 
      |  |      |   0    |    5     |
      +--|------+--------+----------+
         |
         |
         v
       +---+---+---+---+---+
Heap:  | ? | ? | ? | ? | ? |
       +---+---+---+---+---+
```

En fazla 5 byte metin tutabilen bir `String` istedik.\
`String::with_capacity` allocator'a gider ve 5 byte heap belleği ister. Allocator o bellek bloğunun
başlangıcına işaret eden bir pointer döndürür.\
Ancak `String` boştur. Stack'te bu bilgiyi uzunluk ile kapasiteyi birbirinden ayırarak takip ederiz:
bu `String` en fazla 5 byte tutabilir, ama şu anda 0 byte gerçek metin tutuyor.

`String`'e biraz metin eklersen durum değişir:

```rust
s.push_str("Hey");
```

```
      +---------+--------+----------+
Stack | pointer | length | capacity |
      |  |      |   3    |    5     |
      +--|  ----+--------+----------+
         |
         |
         v
       +---+---+---+---+---+
Heap:  | H | e | y | ? | ? |
       +---+---+---+---+---+
```

`s` artık 3 byte metin tutuyor. Uzunluğu 3'e güncellenir, ama kapasitesi 5 olarak kalır.
Heap'teki beş byte'ın üçü `H`, `e` ve `y` karakterlerini depolamak için kullanılır.

### `usize` {#usize}

Pointer'ı, uzunluğu ve kapasiteyi stack'te depolamak için ne kadar yere ihtiyacımız var?\
Bu, üzerinde çalıştığın makinenin **mimarisine** bağlıdır.

Makinendeki her bellek konumunun, genellikle işaretsiz bir tam sayı olarak temsil edilen bir
[**adresi**](https://en.wikipedia.org/wiki/Memory_address) vardır.
Adres alanının maksimum boyutuna (yani makinenin ne kadar belleği adresleyebildiğine) bağlı olarak,
bu tam sayının boyutu farklı olabilir. Çoğu modern makine ya 32 bit ya da 64 bit bir adres alanı kullanır.

Rust, bu mimariye özgü ayrıntıları `usize` türünü sağlayarak soyutlar:
makinende belleği adreslemek için gereken byte sayısı kadar büyük olan işaretsiz bir tam sayı.
32 bit bir makinede `usize`, `u32`'ye eşdeğerdir. 64 bit bir makinede `u64`'e karşılık gelir.

Rust'ta kapasite, uzunluk ve pointer'ların hepsi `usize` olarak temsil edilir[^equivalence].

### Heap İçin `std::mem::size_of` Yok {#no-stdmemsize_of-for-the-heap}

`std::mem::size_of`, bir türün stack'te kaplayacağı alan miktarını döndürür;
bu, **türün boyutu** olarak da bilinir.

> `String`'in heap'te yönettiği bellek buffer'ı ne olacak? O da
> `String`'in boyutunun bir parçası değil mi?

Hayır!\
O heap allocation'ı, `String`'in yönettiği bir **kaynaktır**.
Derleyici onu `String` türünün bir parçası olarak kabul etmez.

`std::mem::size_of`, `String`'de olduğu gibi bir türün yönetebileceği ya da pointer'lar aracılığıyla
başvurabileceği ek heap'te ayrılmış verileri bilmez (ya da umursamaz),
bu yüzden bu verilerin boyutunu takip etmez.

Ne yazık ki, belirli bir değerin çalışma zamanında ayırdığı heap belleği miktarını ölçmek için
`std::mem::size_of`'un bir eşdeğeri yoktur. Bazı türler heap kullanımlarını incelemek için
metotlar sağlayabilir (ör. `String`'in `capacity` metodu),
ama Rust'ta çalışma zamanındaki heap kullanımını öğrenmek için genel amaçlı bir "API" yoktur.\
Yine de programının heap kullanımını incelemek için bir bellek profiler aracı (ör. [DHAT](https://valgrind.org/docs/manual/dh-manual.html)
ya da [özel bir allocator](https://docs.rs/dhat/latest/dhat/)) kullanabilirsin.

[^empty]: **Boş** bir `String` oluşturursan (yani `String::new()`) `std` bellek ayırmaz.
Heap belleği, içine ilk kez veri eklediğinde ayrılır.

[^equivalence]: Bir pointer'ın boyutu işletim sistemine de bağlıdır.
Bazı ortamlarda bir pointer, bir bellek adresinden **daha büyüktür** (ör. [CHERI](https://web.archive.org/web/20240517051950/https://blog.acolyer.org/2019/05/28/cheri-abi/)).
Rust, pointer'ların bellek adresleriyle aynı boyutta olduğu yönünde basitleştirici bir varsayımda bulunur;
bu, karşılaşman muhtemel çoğu modern sistem için doğrudur.
