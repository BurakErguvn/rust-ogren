# Çeviri Stil Rehberi

Tüm `translator` ajanları ve `translation-qa` bu rehbere ve `tr/glossary.json`'a uyar.
Sözlük ile bu rehber çelişirse sözlük kazanır; ikisi de bir durumu kapsamıyorsa
orkestratöre sor, kendi başına yeni bir terim kararı verme.

## 1. Temel ilke: 1:1

- Anlam ve yapı birebir korunur: paragraf paragraf, cümle cümle. **Ekleme, çıkarma,
  özetleme, "uyarlama" yok.** Orijinal `wr` CLI'dan bahsediyorsa çeviri de bahseder.
- Türkçe doğal okunmalı; kelime kelime değil anlam anlam çevir. Cümleyi Türkçe söz dizimine
  göre yeniden kur, ama bilgi kaybetme ve bilgi ekleme.
- Ton: orijinal gibi samimi ve doğrudan. Okura **"sen"** diye hitap edilir
  ("Şimdi `if` ifadesine bakalım", "fonksiyonu çağırabilirsin").
- Mizah ve kelime oyunları anlamı koruyacak şekilde Türkçeleştirilir; çevrilemiyorsa
  düz anlamı verilir.

## 2. Markdown yapısı (birebir korunur)

- Başlık sayısı ve seviyeleri, liste yapısı, tablo yapısı, blok alıntılar, `<div class="warning">`
  blokları, satır sonundaki `\` (satır kırma) aynı yerde kalır.
- **Başlıklar** çevrilir ve sonuna orijinal İngilizce başlığın slug'ı eklenir, böylece
  `sayfa.md#slug` linkleri kırılmaz:
  `## Function arguments are variables` → `## Fonksiyon argümanları birer değişkendir {#function-arguments-are-variables}`
  **Başlıklarda her kelimenin ilk harfi büyüktür** (bağlaçlar ve İngilizce kalan terimler dahil;
  Türkçe büyük harf kuralıyla: i → İ, ı → I): `## Fonksiyon Argümanları Birer Değişkendir`,
  `# Türler, 1. Kısım`, `## Otomatik Type Coercion (Örtük Tür Dönüşümü) Yok`.
  Inline kod (`` `while` ``), kod adları (`panic!` makro adı düz metinde de küçük harf kalır),
  kesme işaretinden sonraki ek (`Panic'ler`) ve `{#slug}` değişmez.
  Slug kuralı (GitHub/mdBook): küçük harf, harf/rakam/boşluk/`-`/`_` dışındakiler silinir,
  boşluklar `-` olur. `` `if` clauses `` → `if-clauses`.
- **Link hedefleri** (`(...)` içi) asla değişmez; link metni çevrilir.
  Link metni inline kod ise (`` [`u32`](...) ``) aynen kalır.
- **Dipnotlar**: `[^isim]` etiketleri aynen kalır, dipnot metni çevrilir.
- **Inline kod** (`` `...` ``) asla çevrilmez.
- Görseller, HTML etiketleri ve öznitelikleri aynen kalır.

## 3. Kod blokları

- ```` ```rust ```` blokları: **yalnızca yorumlar** çevrilir. Kod, string literal'ler
  (`println!` metinleri dahil), tanımlayıcılar, boşluk/girinti aynen kalır.
- ```` ```text ````, ```` ```bash ````, ```` ```toml ```` ve dili belirtilmemiş bloklar:
  hiç dokunulmaz (derleyici çıktısı, komutlar).
- Gizli satırlar (`# ` ile başlayan rust satırları) aynen kalır.

## 4. Egzersiz dosyaları (`.rs`)

- Çevrilen: `//`, `///`, `//!`, `/* */`, `/** */` yorumlarının **içeriği**.
- Değişmeyen: string literal'ler (panic/assert mesajları, `#[should_panic(expected = "...")]`),
  tanımlayıcılar, kod, boşluk ve girinti, yorum işaretleri, `TODO:` gibi etiketler
  (`// TODO: ...` → `// TODO: ...` etiketi korunur, sonrası çevrilir), emoji ve işaretler (`👇`).
- Yorum içindeki inline kod (`` `speed` ``) çevrilmez.
- Egzersiz yorumlarında `keep_gloss` açıklaması **verilmez** (açıklama anlatım sayfasında verilir).
- Satır sayısı değişebilir (Türkçe uzunsa satır bölünebilir), ama yorum dışı her token
  aynı sırada kalmalı. `tr-check` aracı bunu doğrular; geçmeyen dosya teslim edilmez.
- Satır genişliği: orijinal yorum bloğunun yaklaşık genişliğini (≈100 karakter) koru.

## 5. Terimler

`tr/glossary.json` her terim için bir politika tanımlar:

| Politika | Anlam | Örnek |
|---|---|---|
| `keep` | İngilizce kalır, açıklama eklenmez | `struct`, `trait`, `enum`, `crate` |
| `keep_gloss` | İngilizce kalır; **her sayfadaki ilk geçişte** Türkçe karşılık verilir: `**ownership** (sahiplik)` — kalın yalnızca İngilizce terimde, açıklama dışında | ownership (sahiplik) |
| `translate` | Türkçe karşılığı kullanılır | function → fonksiyon, macro → makro |

- Uygulamada sayfalar arasında serbest gezinildiği için her sayfa kendi başına anlaşılır
  olmalı; açıklama sayfa başına bir kez verilir. Orijinalde terim zaten kalın değilse
  kalınlık eklenmez, yalnızca `ownership (sahiplik)` yazılır.
- Bir terimin `note` alanındaki kalıplar (ör. `lock` notundaki "read-write lock → ...")
  **bağlayıcıdır**.
- `en` alanındaki köşeli/parantezli niteleyiciler (`branch (git)`, `runtime (execution)`)
  yalnızca anlamları ayırır; çeviride yazılmaz.
- **Fiiller:** İngilizce kalan fiillerin Türkçe biçimi sözlüğün `suffix` alanında verilir ve
  aynen kullanılır (implement etmek, spawn etmek, derive etmek; panic'lemek, commit'lemek,
  hash'lemek). Sözlükte olmayan bir fiil için varsayılan "X etmek"tir; raporda listele.
- Sözlükte olmayan bir Rust/programlama terimiyle karşılaşırsan raporunda listele.
- İngilizce terimlere ek **kesme işaretiyle** ve terimin **İngilizce okunuşuna** göre gelir:
  trait'ler, trait'i (treyt), struct'ı (strakt), enum'u, crate'ler, closure'lar, lifetime'ı.
  Inline kodda ek backtick'in dışına yazılır: `` `Ticket`'ı ``, `` `u32`'yi ``.
  **Rakamla biten adlarda** ek, rakamın **Türkçe okunuşuna** göre gelir:
  `` `u16`'nın `` (on altı), `` `i32`'dir `` (otuz iki), `` `u8`'in `` (sekiz), `` `u64`'ün `` (altmış dört).
  Sözlükteki `suffix` alanı tercih edilen ek örneklerini verir.
- Tutarlılık: aynı kavram bütün kitap boyunca aynı şekilde yazılır.
- Kalın yazılan İngilizce terimde ek kalının dışında kalır: `**task**'lerini`, `**destructor**'ları`.
- Türkçeye çevrilen (`translate`) bir terimin sayfadaki ilk geçişinde İngilizcesi parantez içinde
  verilebilir (`zaman aşımı (timeout)`); bu ekleme sayılmaz.
- Kitap boyunca "ticket store" / "the store" düz metinde **store** olarak kalır
  (`store'u`, `store'dan`, `ticket store'u`); `TicketStore` tür adı aynen.

## 6. Yazım

- Türkçe karakterler eksiksiz (ç, ğ, ı, İ, ö, ş, ü). TDK yazımı.
- Teknik fiiller için yerleşik kullanım: derlemek, çalıştırmak, çağırmak, döndürmek,
  tanımlamak, ödünç almak. "implement" **çevrilmez**: implement etmek / implementation.
  "uygulama" yalnızca application (program) anlamındadır.
- TDK yazımı: metot (metodu, metotlar).
- Sayılar ve birimler orijinal biçimde (`2^32`, `u8`).
