# Bağımlılıklar {#dependencies}

Bir paket, diğer paketleri `Cargo.toml` dosyasının `[dependencies]` kısmında listeleyerek onlara bağımlı olabilir.\
Bir bağımlılığı belirtmenin en yaygın yolu adını ve sürümünü vermektir:

```toml
[dependencies]
thiserror = "1"
```

Bu, `thiserror`'ı **minimum** `1.0.0` sürümüyle paketine bir bağımlılık olarak ekler.
`thiserror`, Rust'ın resmi paket registry'si olan [crates.io](https://crates.io)'dan çekilir.
`cargo build` çalıştırdığında `cargo` birkaç aşamadan geçer:

- Bağımlılık çözümleme
- Bağımlılıkları indirme
- Projeni derleme (kendi kodun ve bağımlılıklar)

Projende bir `Cargo.lock` dosyası varsa ve manifest dosyaların değişmemişse bağımlılık çözümleme atlanır.
Lockfile, başarılı bir bağımlılık çözümleme turunun ardından `cargo` tarafından otomatik olarak oluşturulur: projende
kullanılan tüm bağımlılıkların tam sürümlerini içerir ve farklı derlemelerde (ör. CI'da) aynı sürümlerin tutarlı bir şekilde
kullanılmasını sağlamak için kullanılır. Birden çok geliştiriciyle bir proje üzerinde çalışıyorsan,
`Cargo.lock` dosyasını sürüm kontrol sistemine commit'lemelisin.

`cargo update` kullanarak `Cargo.lock` dosyasını tüm bağımlılıklarının en son (uyumlu) sürümleriyle güncelleyebilirsin.

### Path Bağımlılıkları {#path-dependencies}

Bir bağımlılığı bir **path** (yol) kullanarak da belirtebilirsin. Bu, birden çok yerel paket üzerinde çalışırken kullanışlıdır.

```toml
[dependencies]
my-library = { path = "../my-library" }
```

Path, bağımlılığı bildiren paketin `Cargo.toml` dosyasına göredir.

### Diğer Kaynaklar {#other-sources}

Bağımlılıkları nereden alabileceğin ve onları `Cargo.toml` dosyanda nasıl belirteceğin hakkında daha fazla
ayrıntı için [Cargo dokümantasyonuna](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) göz at.

## Geliştirme Bağımlılıkları {#dev-dependencies}

Yalnızca geliştirme için gereken bağımlılıkları da belirtebilirsin—yani bunlar yalnızca `cargo test`
çalıştırdığında çekilir.\
Bunlar `Cargo.toml` dosyanın `[dev-dependencies]` kısmına yazılır:

```toml
[dev-dependencies]
static_assertions = "1.1.0"
```

Testlerimizi kısaltmak için kitap boyunca bunlardan birkaçını kullandık.
