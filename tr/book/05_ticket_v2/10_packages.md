# Kütüphaneler Ve Binary'ler {#libraries-and-binaries}

`TicketNewError` için `Error` trait'ini implement etmek epey kod gerektirdi, değil mi?\
Elle yazılmış bir `Display` implementation'ı, artı bir `Error` impl bloğu.

[`thiserror`](https://docs.rs/thiserror/latest/thiserror/) kullanarak boilerplate'in bir kısmını kaldırabiliriz;
bu, özel hata türlerinin oluşturulmasını basitleştirmek için bir **procedural macro** sağlayan bir Rust crate'idir.\
Ama fazla ileri gidiyoruz: `thiserror` üçüncü taraf bir crate, ilk bağımlılığımız olurdu!

Bağımlılıklara dalmadan önce bir adım geri çekilip Rust'ın paket sistemi hakkında konuşalım.

## Paket Nedir? {#what-is-a-package}

Bir Rust paketi, bir `Cargo.toml` dosyasındaki `[package]` kısmıyla tanımlanır; bu dosya paketin **manifest**'i olarak da bilinir.
`[package]` içinde paketin adı ve sürümü gibi meta verilerini ayarlayabilirsin.

Hemen bu kısmın egzersizinin dizinindeki `Cargo.toml` dosyasına bir göz at!

## Crate Nedir? {#what-is-a-crate}

Bir paketin içinde, **target** (hedef) olarak da bilinen bir ya da daha fazla **crate** olabilir.\
En yaygın iki crate türü **binary crate**'ler ve **kütüphane crate**'leridir.

### Binary'ler {#binaries}

Binary, **çalıştırılabilir bir dosyaya** derlenebilen bir programdır.\
`main` adında bir fonksiyon içermelidir—programın giriş noktası. `main`, program çalıştırıldığında çağrılır.

### Kütüphaneler {#libraries}

Kütüphaneler ise kendi başlarına çalıştırılamaz. Bir kütüphaneyi _çalıştıramazsın_,
ama ona bağımlı olan başka bir paketten _onun kodunu import edebilirsin_.\
Bir kütüphane, diğer paketlerin bir **bağımlılık** olarak yararlanabileceği kodları (yani fonksiyonları, türleri vb.) bir araya getirir.

Şimdiye kadar çözdüğün tüm egzersizler, kendilerine bir test suite'i eklenmiş kütüphaneler olarak yapılandırılmıştı.

### Gelenekler {#conventions}

Rust paketleriyle ilgili aklında tutman gereken bazı gelenekler vardır:

- Paketin kaynak kodu genellikle `src` dizininde bulunur.
- Bir `src/lib.rs` dosyası varsa `cargo`, paketin bir kütüphane crate'i içerdiğini çıkarsar.
- Bir `src/main.rs` dosyası varsa `cargo`, paketin bir binary crate'i içerdiğini çıkarsar.

Bu varsayılanları, target'larını `Cargo.toml` dosyasında açıkça bildirerek override edebilirsin—daha fazla ayrıntı için
[`cargo`'nun dokümantasyonuna](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#cargo-targets) bak.

Unutma: bir paket birden çok crate içerebilse de yalnızca bir kütüphane crate'i içerebilir.
