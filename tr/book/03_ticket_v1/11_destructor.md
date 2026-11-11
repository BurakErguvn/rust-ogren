# Destructor'lar (Yıkıcılar) {#destructors}

Heap'i tanıtırken, ayırdığın belleği serbest bırakmaktan senin sorumlu olduğundan bahsetmiştik.\
Borrow checker'ı tanıtırken de Rust'ta belleği nadiren doğrudan yönetmen gerektiğini söylemiştik.

Bu iki iddia ilk bakışta çelişkili görünebilir.
**Kapsamları** ve **destructor**'ları tanıtarak bunların nasıl bir araya geldiğine bakalım.

## Kapsamlar {#scopes}

Bir değişkenin **kapsamı**, Rust kodunda o değişkenin geçerli, yani **canlı** olduğu bölgedir.

Bir değişkenin kapsamı, bildirimiyle başlar.
Aşağıdakilerden biri gerçekleştiğinde sona erer:

1. değişkenin bildirildiği blok (yani `{}` arasındaki kod) sona erdiğinde
   ```rust
   fn main() {
      // `x` burada henüz kapsamda değil
      let y = "Hello".to_string();
      let x = "World".to_string(); // <-- x'in kapsamı burada başlar...
      let h = "!".to_string(); //   |
   } //  <-------------- ...ve burada biter
   ```
2. değişkenin ownership'i (sahipliği) başka birine (ör. bir fonksiyona ya da başka bir değişkene) devredildiğinde
   ```rust
   fn compute(t: String) {
      // Bir şey yap [...]
   }

   fn main() {
       let s = "Hello".to_string(); // <-- s'nin kapsamı burada başlar...
                   //                    | 
       compute(s); // <------------------- ..ve burada biter
                   //   çünkü `s`, `compute`'a move edilir (taşınır)
   }
   ```

## Destructor'lar {#destructors-1}

Bir değerin sahibi kapsam dışına çıktığında, Rust onun **destructor**'ını çağırır.\
Destructor, o değerin kullandığı kaynakları—özellikle de ayırdığı belleği—temizlemeye çalışır.

Bir değerin destructor'ını, değeri `std::mem::drop`'a geçirerek elle çağırabilirsin.\
Bu yüzden Rust geliştiricilerinin, bir değerin kapsam dışına çıktığını ve destructor'ının çağrıldığını
belirtmek için sıklıkla "o değer **drop edildi**" dediğini duyarsın.

### Drop Noktalarını Görselleştirmek {#visualizing-drop-points}

Derleyicinin bizim için yaptığını "açıkça yazmak" için `drop`'a açık çağrılar ekleyebiliriz. Önceki örneğe dönersek:

```rust
fn main() {
   let y = "Hello".to_string();
   let x = "World".to_string();
   let h = "!".to_string();
}
```

Şuna eşdeğerdir:

```rust
fn main() {
   let y = "Hello".to_string();
   let x = "World".to_string();
   let h = "!".to_string();
   // Değişkenler, bildirim sırasının tersine göre drop edilir
   drop(h);
   drop(x);
   drop(y);
}
```

Bunun yerine, `s`'nin ownership'inin `compute`'a devredildiği ikinci örneğe bakalım:

```rust
fn compute(s: String) {
   // Bir şey yap [...]
}

fn main() {
   let s = "Hello".to_string();
   compute(s);
}
```

Şuna eşdeğerdir:

```rust
fn compute(t: String) {
    // Bir şey yap [...]
    drop(t); // <-- `t`'nin bu noktadan önce drop ya da move
             //     edilmediğini varsayarsak, derleyici `t` kapsam
             //     dışına çıktığında burada `drop`'u çağırır
}

fn main() {
    let s = "Hello".to_string();
    compute(s);
}
```

Farka dikkat et: `main` içinde `compute` çağrıldıktan sonra `s` artık geçerli olmasa da `main` içinde
bir `drop(s)` yoktur.
Bir değerin ownership'ini bir fonksiyona devrettiğinde, **onu temizleme sorumluluğunu da devretmiş olursun**.

Bu, bir değerin destructor'ının **en fazla[^leak] bir kez** çağrılmasını garanti eder ve tasarım gereği
[double free bug'larını](https://owasp.org/www-community/vulnerabilities/Doubly_freeing_memory) önler.

### Drop Sonrası Kullanım {#use-after-drop}

Bir değeri drop edildikten sonra kullanmaya çalışırsan ne olur?

```rust
let x = "Hello".to_string();
drop(x);
println!("{}", x);
```

Bu kodu derlemeye çalışırsan bir hata alırsın:

```rust
error[E0382]: use of moved value: `x`
 --> src/main.rs:4:20
  |
3 |     drop(x);
  |          - value moved here
4 |     println!("{}", x);
  |                    ^ value used here after move
```

Drop, üzerinde çağrıldığı değeri **tüketir**; yani değer, çağrıdan sonra artık geçerli değildir.\
Bu nedenle derleyici onu kullanmanı engeller ve böylece [use-after-free bug'larının](https://owasp.org/www-community/vulnerabilities/Using_freed_memory) önüne geçer.

### Referansları Drop Etmek {#dropping-references}

Bir değişken bir referans içeriyorsa ne olur?\
Örneğin:

```rust
let x = 42i32;
let y = &x;
drop(y);
```

`drop(y)` çağrıldığında... hiçbir şey olmaz.\
Bu kodu gerçekten derlemeye çalışırsan bir uyarı alırsın:

```text
warning: calls to `std::mem::drop` with a reference 
         instead of an owned value does nothing
 --> src/main.rs:4:5
  |
4 |     drop(y);
  |     ^^^^^-^
  |          |
  |          argument has type `&i32`
  |
```

Bu, daha önce söylediğimiz şeye dayanıyor: destructor'ı yalnızca bir kez çağırmak istiyoruz.\
Aynı değere birden fazla referansın olabilir—bunlardan biri kapsam dışına çıktığında işaret ettikleri değerin
destructor'ını çağırsaydık, diğerlerine ne olurdu?
Artık geçerli olmayan bir bellek konumuna başvururlardı; bu, [**dangling pointer**](https://en.wikipedia.org/wiki/Dangling_pointer) (sarkan pointer)
denen şeydir ve [**use-after-free bug**'larının](https://owasp.org/www-community/vulnerabilities/Using_freed_memory) yakın bir akrabasıdır.
Rust'ın ownership sistemi bu tür bug'ları tasarım gereği ortadan kaldırır.

[^leak]: Rust, destructor'ların çalışacağını garanti etmez. Örneğin, açıkça
[bellek sızdırmayı](../07_threads/03_leak.md) seçersen çalışmazlar.
