# Variant'lar Veri Tutabilir {#variants-can-hold-data}

```rust
enum Status {
    ToDo,
    InProgress,
    Done,
}
```

`Status` enum'umuz, genellikle **C tarzı enum** denen şeydir.\
Her variant basit bir etikettir, biraz isimlendirilmiş bir sabite benzer. Bu tür enum'ları C, C++, Java, C#, Python gibi
birçok programlama dilinde bulabilirsin.

Yine de Rust enum'ları daha da ileri gidebilir. **Her variant'a veri iliştirebiliriz**.

## Variant'lar {#variants}

Diyelim ki o anda bir ticket üzerinde çalışan kişinin adını saklamak istiyoruz.\
Bu bilgiye yalnızca ticket üzerinde çalışılıyorsa sahip oluruz. Yapılacak bir ticket'ta ya da
tamamlanmış bir ticket'ta bu bilgi bulunmazdı.
Bunu, `InProgress` variant'ına bir `String` alanı iliştirerek modelleyebiliriz:

```rust
enum Status {
    ToDo,
    InProgress {
        assigned_to: String,
    },
    Done,
}
```

`InProgress` artık **struct benzeri bir variant**.\
Söz dizimi aslında bir struct tanımlarken kullandığımız söz dizimini yansıtır—yalnızca enum'un içine, bir variant olarak "satır içine yerleştirilmiştir".

## Variant Verisine Erişmek {#accessing-variant-data}

Bir `Status` instance'ı (örneği) üzerinde `assigned_to`'ya erişmeye çalışırsak,

```rust
let status: Status = /* */;

// Bu derlenmez
println!("Assigned to: {}", status.assigned_to);
```

derleyici bizi durdurur:

```text
error[E0609]: no field `assigned_to` on type `Status`
 --> src/main.rs:5:40
  |
5 |     println!("Assigned to: {}", status.assigned_to);
  |                                        ^^^^^^^^^^^ unknown field
```

`assigned_to` **variant'a özgüdür**; tüm `Status` instance'larında mevcut değildir.\
`assigned_to`'ya erişmek için **pattern matching** (desen eşleştirme) kullanmamız gerekir:

```rust
match status {
    Status::InProgress { assigned_to } => {
        println!("Assigned to: {}", assigned_to);
    },
    Status::ToDo | Status::Done => {
        println!("ToDo or Done");
    }
}
```

## Binding'ler {#bindings}

`Status::InProgress { assigned_to }` match pattern'inde `assigned_to` bir **binding**'dir (bağlama).\
`Status::InProgress` variant'ını **destructure** ediyoruz (parçalarına ayırıyoruz) ve `assigned_to` alanını,
adı yine `assigned_to` olan yeni bir değişkene bağlıyoruz.\
İstersek alanı farklı bir değişken adına da bağlayabilirdik:

```rust
match status {
    Status::InProgress { assigned_to: person } => {
        println!("Assigned to: {}", person);
    },
    Status::ToDo | Status::Done => {
        println!("ToDo or Done");
    }
}
```
