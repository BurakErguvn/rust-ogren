# Panic'ler {#panics}

["Variables" kısmı](02_variables.md) için yazdığın `speed` fonksiyonuna geri dönelim.
Muhtemelen şuna benzer bir şeydi:

```rust
fn speed(start: u32, end: u32, time_elapsed: u32) -> u32 {
    let distance = end - start;
    distance / time_elapsed
}
```

Keskin bir gözün varsa bir sorunu fark etmiş olabilirsin[^one]: `time_elapsed` sıfır olursa ne olur?

Bunu
[Rust playground'unda](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=36e5ddbe3b3f741dfa9f74c956622bac) deneyebilirsin!\
Program şu hata mesajıyla sonlanır:

```text
thread 'main' panicked at src/main.rs:3:5:
attempt to divide by zero
```

Buna **panic** denir.\
Panic, Rust'ın bir şeylerin programın çalışmaya devam edemeyeceği kadar ters gittiğini bildirme yoludur;
panic **kurtarılamaz bir hatadır**[^catching]. Sıfıra bölme de böyle bir hata olarak sınıflandırılır.

## panic! Makrosu {#the-panic-macro}

`panic!` makrosunu[^macro] çağırarak kasıtlı olarak bir panic oluşturabilirsin:

```rust
fn main() {
    panic!("This is a panic!");
    // Aşağıdaki satır asla çalıştırılmayacak
    let x = 1 + 2;
}
```

Rust'ta kurtarılabilir hatalarla çalışmak için başka mekanizmalar da var; bunları [daha sonra ele alacağız](../05_ticket_v2/06_fallibility.md).
Şimdilik, kaba ama basit bir geçici çözüm olarak panic'lerle idare edeceğiz.

## İleri Okuma {#further-reading}

- [panic! makrosunun dokümantasyonu](https://doc.rust-lang.org/std/macro.panic.html)

[^one]: `speed` ile ilgili, yakında ele alacağımız başka bir sorun daha var. Onu fark edebilir misin?

[^catching]: Bir panic'i yakalamayı deneyebilirsin, ama bu, çok özel durumlara ayrılmış son çare bir girişim
olmalıdır.

[^macro]: Ardından bir `!` geliyorsa, bu bir makro çağrısıdır. Şimdilik makroları baharatlı fonksiyonlar olarak düşün. Onları
kursun ilerleyen kısımlarında daha ayrıntılı ele alacağız.
