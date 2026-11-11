# Veri Sızdırma {#leaking-data}

Spawn edilen thread'lere (iş parçacıklarına) referans geçirmeyle ilgili asıl endişe use-after-free bug'larıdır:
zaten serbest bırakılmış bir bellek bölgesini gösteren bir pointer (işaretçi) aracılığıyla veriye erişmek.\
Heap'te ayrılmış verilerle çalışıyorsan, Rust'a o belleği asla geri almayacağını söyleyerek
bu sorundan kaçınabilirsin: kasıtlı olarak **bellek sızdırmayı**
seçersin.

Bu, örneğin Rust'ın standart kütüphanesindeki `Box::leak` metoduyla yapılabilir:

```rust
// Bir `u32`'yi `Box` içine sarmalayarak heap'te bellek ayır.
let x = Box::new(41u32);
// `Box::leak` kullanarak Rust'a heap'te ayrılmış bu belleği
// asla serbest bırakmayacağını söyle. Böylece bir 'static referans geri alabilirsin.
let static_ref: &'static mut u32 = Box::leak(x);
```

## Veri Sızıntısı Süreç Kapsamlıdır {#data-leakage-is-process-scoped}

Veri sızdırmak tehlikelidir: bellek sızdırmaya devam edersen, sonunda
bellek tükenir ve bir out-of-memory hatasıyla çökersin.

```rust
// Bunu bir süre çalışır halde bırakırsan,
// sonunda kullanılabilir belleğin tamamını tüketir.
fn oom_trigger() {
    loop {
        let v: Vec<usize> = Vec::with_capacity(1024);
        v.leak();
    }
}
```

Öte yandan, `leak` metoduyla sızdırılan bellek gerçekten unutulmuş değildir.\
İşletim sistemi her bellek bölgesini ondan sorumlu sürece eşleyebilir.
Süreç sonlandığında işletim sistemi o belleği geri alır.

Bunu aklında tutarak, şu durumlarda bellek sızdırmak kabul edilebilir olabilir:

- Sızdırman gereken bellek miktarı sınırlıysa/önceden biliniyorsa ya da
- Sürecin kısa ömürlüyse ve sonlanmadan önce kullanılabilir belleğin tamamını tüketmeyeceğinden
  eminsen

Kullanım senaryon buna izin veriyorsa, "Bırak işletim sistemi halletsin" son derece geçerli bir bellek yönetimi stratejisidir.
