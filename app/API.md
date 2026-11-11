# API sözleşmesi (backend ↔ frontend)

`backend-dev` ve `frontend-dev` bu dosyaya göre paralel çalışır. Değişiklik gerekirse
ajan raporunda önerir; orkestratör bu dosyayı günceller. Tüm gövdeler JSON (UTF-8).
Hata: uygun HTTP kodu + `{ "error": "<Türkçe mesaj>" }`.

## Kimlikler

- **Sayfa kimliği** (`page_id`): `book/src` altındaki yol, `.md` olmadan:
  `02_basic_calculator/04_panics`, `going_further`.
- **Egzersiz kimliği**: `{chapter}/{exercise}` iki yol parçası olarak:
  `02_basic_calculator/04_panics`. Sayfa ↔ egzersiz eşlemesi isimle birebir
  (`exercises/<page_id>/` dizini varsa egzersiz vardır).
- Dosya yolları her zaman egzersiz kökünden göreli, `/` ayraçlı: `src/lib.rs`, `tests/integration.rs`.

## İçerik

### `GET /api/toc`
```json
{
  "chapters": [
    {
      "id": "02_basic_calculator",
      "title": "Basit Bir Hesap Makinesi",
      "pages": [
        {
          "id": "02_basic_calculator/04_panics",
          "title": "Panic'ler",
          "translated": true,
          "exercise": { "completed": false, "score": null, "failed_submits": 1 }
        }
      ]
    },
    {
      "id": "going_further",
      "title": "Daha ileri",
      "pages": [ { "id": "going_further", "title": "Daha ileri", "translated": false, "exercise": null } ]
    }
  ],
  "progress": { "completed": 3, "total": 98, "score_sum": 270 }
}
```
- Sıra `book/src/SUMMARY.md` sırasıdır. Bölümün ilk satırı (`00_intro.md`, `00_welcome.md`)
  bölüm başlığını verir ve aynı zamanda bölümün ilk sayfasıdır (pages[0]).
  Kök seviyedeki `going_further.md` tek sayfalı bir bölüm olarak gelir.
- `title`: Türkçe sayfa varsa onun ilk `# ` başlığı (sondaki `{#slug}` ve satır içi kod
  backtick'leri hariç değil — backtick'ler korunur, frontend inline kod olarak gösterir);
  yoksa SUMMARY.md'deki İngilizce başlık.
- `exercise`: egzersizi olmayan sayfada `null`.

### `GET /api/pages/{page_id}`  (ör. `/api/pages/02_basic_calculator/04_panics`)
```json
{
  "id": "02_basic_calculator/04_panics",
  "title": "Panic'ler",
  "translated": true,
  "markdown": "# Panic'ler {#panics}\n...",
  "prev": "02_basic_calculator/03_if_else",
  "next": "02_basic_calculator/05_factorial",
  "exercise": { "completed": false, "score": null, "failed_submits": 1 },
  "notes": [ { "before": "structure", "markdown": "**Bunlarla uğraşmana gerek yok!** ..." } ]
}
```
- `notes`: uygulamaya özel notlar (çevirinin parçası değildir, `tr-check` kapsamı dışında).
  Kaynak `tr/notes/<page_id>.json` (`{ "notes": [ { "before": <başlık id'si | null>, "markdown": "..." } ] }`);
  dosya yoksa `[]`. Frontend her notu "Uygulama notu" başlıklı belirgin bir kutu olarak,
  `before` id'li başlığın hemen önünde (`null` ise sayfanın en başında, H1'den sonra) gösterir.
- `markdown`: `tr/book/<page_id>.md`, yoksa `book/src/<page_id>.md`, ham haliyle.
  Göreli linkler (`04_panics.md#slug`, `../03_ticket_v1/01_struct.md`) frontend tarafından
  uygulama içi linklere çevrilir. Görsel yolları backend tarafından değiştirilmez;
  `GET /api/book-assets/{path}` `book/src/{path}` dosyalarını sunar.
- `prev`/`next`: SUMMARY sırasına göre, uçlarda `null`.

## Egzersiz

### `GET /api/exercises/{chapter}/{exercise}/files`
Çalışma kopyası yoksa önce oluşturulur.
```json
{
  "id": "02_basic_calculator/04_panics",
  "workspace_root_uri": "file:///abs/path/app/workspace",
  "exercise_root_uri": "file:///abs/path/app/workspace/exercises/02_basic_calculator/04_panics",
  "files": [
    { "path": "src/lib.rs", "content": "...", "editable": true, "deletable": false, "language": "rust", "has_hidden_tests": true }
  ],
  "hidden_test_files": 1,
  "progress": { "completed": false, "score": null, "failed_submits": 0 }
}
```
- `files`: `src/**` ve `tests/**` altındaki `.rs` dosyaları, `src/lib.rs` (veya `src/main.rs`)
  önce, sonra alfabetik. Hepsi düzenlenebilir.
- **Gizli testler:** test kodu öğrenciye gösterilmez (çözüm ipucu vermesin diye).
  - `.rs` dosyalarındaki üst seviye `#[cfg(test)]` öğeleri (tipik olarak `mod tests { ... }`,
    önündeki yorumlarla birlikte) `content`'ten çıkarılır; o dosyada `has_hidden_tests: true`.
  - `tests/` altındaki dosyalar `files`'ta hiç listelenmez; sayısı `hidden_test_files`'ta.
  - `PUT` görünen içeriği alır; backend diskteki dosyayı "görünen içerik + gizli test bloğu
    (dosya sonunda)" olarak yazar. Görünen satır numaraları diskteki ile aynıdır.
  - İstisna: test kodu öğrencinin düzenlemesi gereken bir yer içeriyorsa (`todo!()`, `TODO`,
    `__` yer tutucusu vb.) o egzersizde testler görünür kalır.
    Ayrıca talimatı öğrenciyi testlere yönlendiren egzersizler backend'deki `ALWAYS_VISIBLE`
    listesinde (ör. `04_traits/12_copy`, `08_futures/07_cancellation`).
  - Her dosyada `deletable` yalnızca öğrencinin oluşturduğu `src/**.rs` dosyalarında `true`.
  - Her diagnostic'te `hidden` alanı bulunur (görünenlerde `false`).
- Hiç düzenlenmemiş bir çalışma kopyası, kaynağı veya çevirisi değiştiyse bu istekte
  kendiliğinden yenilenir; düzenlenmiş kopyalara dokunulmaz.
- URI'ler LSP içindir; frontend Monaco modellerini `exercise_root_uri + "/" + path` ile açar.

- **Cargo.toml düzenleme:** kaynakları `Cargo.toml` düzenlemeyi isteyen egzersizlerde
  (`02_basic_calculator/08_overflow`, `05_ticket_v2/11_dependencies`, `05_ticket_v2/12_thiserror`,
  `08_futures/08_outro`; backend yorumlarda `Cargo.toml`/bağımlılık geçmesinden tespit eder) `files` ayrıca şunları içerir:
  - egzersizin kendi dosyası: `{ "path": "Cargo.toml", "language": "toml", ... }`
  - çalışma alanı kökü (yalnızca yorum "root"/kök Cargo.toml'u istiyorsa):
    `{ "path": "@workspace/Cargo.toml", "language": "toml", ... }`. Backend bu dosyayı
    yeniden üretirken öğrencinin `[profile.*]` değişikliklerini korur; `members` vb. backend'e
    aittir. Bu egzersizin "Sıfırla"sı kök dosyayı varsayılana döndürür.
  - Her dosyada `language`: `"rust"` | `"toml"`.
- **Yeni dosya:** frontend `src/` altında yeni `.rs` dosyası oluşturabilir: var olmayan bir
  yola `PUT` dosyayı oluşturur (ör. `05_ticket_v2/10_packages` için `src/lib.rs`).
  `DELETE /api/exercises/{chapter}/{exercise}/files/{path}` → `204`; yalnızca öğrencinin
  oluşturduğu (orijinalde olmayan) `src/**.rs` dosyaları silinebilir, diğerleri `403`.

### `PUT /api/exercises/{chapter}/{exercise}/files/{path}`
Gövde: `{ "content": "..." }` → `204`. Yol `src/` veya `tests/` altında bir `.rs` dosyası
olmalı (ya da yukarıdaki Cargo.toml yollarından biri); `..`, mutlak yol vb. → `400`.

### `POST /api/exercises/{chapter}/{exercise}/test`  ve  `.../submit`
Gövde boş. İstemci önce açık dosyaları `PUT` ile kaydeder. Aynı anda tek iş: meşgulse
`409 { "error": "Başka bir test çalışıyor, lütfen bekle." }`.
```json
{
  "kind": "test",
  "status": "passed",
  "tests": [ { "name": "tests::test_x", "passed": false, "output": "thread ... panicked at ..." } ],
  "tests_passed": 2,
  "tests_total": 3,
  "diagnostics": [
    {
      "file": "src/lib.rs",
      "line": 5, "column": 9, "end_line": 5, "end_column": 14,
      "severity": "error",
      "message": "mismatched types",
      "rendered": "error[E0308]: mismatched types\n --> src/lib.rs:5:9\n..."
    }
  ],
  "log": "ham cargo çıktısı (kısaltılmış, ≤64 KB)",
  "duration_ms": 4210,
  "progress": { "completed": false, "score": null, "failed_submits": 1 }
}
```
- `status`: `passed` | `failed` (testler çalıştı, en az biri başarısız) | `compile_error` |
  `timeout`. `passed` için en az bir testin çalışmış ve hepsinin geçmiş olması gerekir
  (`tests_total == 0` ise `failed`). İstisna: orijinal `exercises/` kaynaklarında hiç
  `#[test]`/`#[tokio::test]` yoksa (ör. `03_ticket_v1/03_modules`), temiz derleme `passed` sayılır.
- Test adları: birim testleri libtest adıyla (`tests::by_zero`), `tests/*.rs` entegrasyon
  testleri `<hedef>::<ad>` biçiminde (`integration::ok_it`). Ignored testler listeye ve
  `tests_total`'a girmez.
- Gizli test kodunda oluşan diagnostic'lerde `"hidden": true`; `line`/`column` alanları
  `null`, `rendered` gizli kodu göstermeyen kısa hali (başlık + `note`/`help` satırları).
  Test çıktılarındaki ve `log`'daki gizli kaynak alıntıları da mümkün olduğunca çıkarılır.
- `duration_ms`: derleme dahil toplam süre.
- `line`/`column` 1 tabanlı. `diagnostics` yalnızca egzersize ait dosyalar (diğer crate'ler
  hariç); hem error hem warning.
- **Puan** (yalnızca `submit`): egzersiz tamamlanmamışsa, başarısız teslim `failed_submits`'i
  1 artırır; başarılı teslim egzersizi tamamlar ve
  `score = max(100 − 10 × failed_submits, 50)` olarak sabitlenir. Tamamlanmış egzersizdeki
  sonraki teslimler puanı değiştirmez (yine sonuç döner). `test` puanı hiç etkilemez.
  `timeout` ve `compile_error` başarısız teslim sayılır.

### `POST /api/exercises/{chapter}/{exercise}/reset`
Çalışma kopyasındaki egzersiz dosyalarını Türkçe sürümden yeniden oluşturur; ilerleme
(puan) **değişmez**. Yanıt: `GET .../files` ile aynı.

### `GET /api/progress`
```json
{
  "completed": 3, "total": 98, "score_sum": 270,
  "chapters": [ { "id": "02_basic_calculator", "completed": 3, "total": 11, "score_sum": 270 } ]
}
```
- `chapters` yalnızca egzersizi olan bölümleri içerir (`going_further` yok).

## Kaldırılan egzersizler

`tr/removed-exercises.json` (`{ "removed": ["01_intro/00_welcome", ...] }`) içindeki egzersizler
uygulamada yoktur: sayfanın `exercise` alanı `null`, egzersiz uçları `404`, toplamlarda
(`total`) sayılmaz. Anlatım sayfası kalır. Frontend de derleme anında bu dosyayı okur ve bu
sayfalar için `/egzersiz/...` rotası üretmez.

## LSP

### `WS /api/lsp/{chapter}/{exercise}`
- Saf köprü: WebSocket **text** mesajı = tek bir JSON-RPC mesajı (Content-Length başlığı
  olmadan). Backend stdio tarafında `Content-Length` çerçevelemesini ekler/çıkarır.
- Bağlantı başına bir `rust-analyzer` süreci, çalışma dizini `app/workspace`. İstemci
  `initialize` içinde `rootUri = workspace_root_uri` gönderir. Bağlantı kapanınca süreç
  (süreç grubuyla) öldürülür.
- Egzersiz çalışma kopyası yoksa bağlantıdan önce oluşturulur.

## Statik dosyalar ve yönlendirme

- Frontend: Next.js `output: 'export'`, `trailingSlash: true`. Rotalar:
  `/` (kaldığın yerden devam / içindekiler), `/ders/<page_id>/` (anlatım),
  `/egzersiz/<page_id>/` (IDE). Dinamik rotalar `generateStaticParams` ile derleme anında
  `book/src/SUMMARY.md`'den üretilir (Node yalnızca derlemede gerekir).
- Backend `WEB_DIST` (`app/web/out`) dizinini `/` altında sunar; `/api/*` dışındaki
  bilinmeyen yollar için `404.html`.
- Geliştirmede: `next dev` (3001) → `/api/*` isteklerini `localhost:3000`'e rewrite eder
  (yalnızca development). WebSocket için frontend `NEXT_PUBLIC_API_ORIGIN` ile doğrudan
  `ws://localhost:3000` kullanabilir.
