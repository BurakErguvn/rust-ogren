# Söz dizimi (fixture) {#syntax}

<div class="warning">

Bu dosya yalnızca **mock API** için elle yazılmış bir test fixture'ıdır; gerçek çeviri değildir.\
Callout, dipnot, gizli satır ve bağlantı çözümlemesi burada denenir.

</div>

["Değişkenler" bölümünde](02_variables.md) yazdığın `speed` fonksiyonuna geri dönelim.
Büyük ihtimalle şuna benziyordu:

```rust
fn speed(start: u32, end: u32, time_elapsed: u32) -> u32 {
    let distance = end - start;
    distance / time_elapsed
}
```

Keskin bir gözün varsa bir sorunu fark etmiş olabilirsin[^one]: `time_elapsed` sıfır olursa ne olur?

Bunu [Rust playground'da](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=36e5ddbe3b3f741dfa9f74c956622bac) deneyebilirsin!\
Program şu hata mesajıyla sonlanır:

```text
thread 'main' panicked at src/main.rs:3:5:
attempt to divide by zero
```

Buna **panic** denir.\
Panic, Rust'ın bir şeylerin o kadar ters gittiğini bildirme yoludur ki program çalışmaya devam edemez; bu **kurtarılamaz bir hatadır**[^catching].

## `panic!` makrosu {#the-panic-macro}

`panic!` makrosunu[^macro] çağırarak bilerek panic tetikleyebilirsin:

```rust
# fn helper() {}
fn main() {
    # helper();
    panic!("This is a panic!");
    // Aşağıdaki satır asla çalışmayacak
    let x = 1 + 2;
}
```

Rust'ta kurtarılabilir hatalarla çalışmak için başka mekanizmalar da var, [bunları sonra göreceğiz](../05_ticket_v2/06_fallibility.md).
Şimdilik panic'leri kaba ama basit bir geçici çözüm olarak kullanacağız. Ayrıca [panic başlığı](../02_basic_calculator/04_panics.md#the-panic-macro) ve
[aynı sayfadaki başlık](#the-panic-macro) bağlantıları da çalışmalı.

| Terim | Anlam |
| --- | --- |
| panic | kurtarılamaz hata |
| `unwrap` | sonuç yoksa panic'ler |

## Daha fazla okuma {#further-reading}

- [`panic!` makrosu dokümantasyonu](https://doc.rust-lang.org/std/macro.panic.html)

[^one]: `speed`'in bir sorunu daha var, yakında ona da değineceğiz. Fark edebiliyor musun?

[^catching]: Bir panic'i yakalamayı deneyebilirsin ama bu, çok özel durumlara ayrılmış son çare olmalıdır.

[^macro]: Sonunda `!` varsa bu bir makro çağrısıdır. Şimdilik makroları baharatlı fonksiyonlar olarak düşün.
