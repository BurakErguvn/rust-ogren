# Söz Dizimi {#syntax}

<div class="warning">

İleri atlama!\
Bu kısma başlamadan önce bir önceki kısmın egzersizini tamamla.\
Egzersiz, [kursun GitHub repo'sunda](https://github.com/mainmatter/100-exercises-to-learn-rust) `exercises/01_intro/00_welcome` içinde bulunuyor.\
Kursa başlamak ve çözümlerini doğrulamak için [`wr`](00_welcome.md#wr-the-workshop-runner) aracını kullan.

</div>

Önceki görev bir egzersiz sayılmaz bile, ama seni şimdiden epey Rust **söz dizimiyle** tanıştırdı.
Önceki egzersizde kullanılan Rust söz diziminin her bir ayrıntısını ele almayacağız.
Bunun yerine, ayrıntılara takılmadan ilerlemeye _yetecek kadarını_ ele alacağız.\
Adım adım!

## Yorumlar {#comments}

Tek satırlık yorumlar için `//` kullanabilirsin:

```rust
// Bu tek satırlık bir yorumdur
// Ardından başka bir tek satırlık yorum geliyor
```

## Fonksiyonlar {#functions}

Rust'ta fonksiyonlar `fn` anahtar kelimesiyle tanımlanır; ardından fonksiyonun adı, girdi parametreleri ve
dönüş türü gelir.
Fonksiyonun gövdesi süslü parantezler `{}` içine alınır.

Önceki egzersizde `greeting` fonksiyonunu gördün:

```rust
// `fn` <fonksiyon_adı> ( <girdi parametreleri> ) -> <dönüş_türü> { <gövde> }
fn greeting() -> &'static str {
    // TODO: beni düzelt 👇
    "I'm ready to __!"
}
```

`greeting`'in girdi parametresi yoktur ve bir string slice'a referans (`&'static str`) döndürür.

### Dönüş Türü {#return-type}

Fonksiyon hiçbir şey döndürmüyorsa (yani Rust'ın unit type'ı (birim türü) olan `()`
döndürüyorsa) dönüş türü imzadan çıkarılabilir.
`test_welcome` fonksiyonunda olan da buydu:

```rust
fn test_welcome() {
    assert_eq!(greeting(), "I'm ready to learn Rust!");
}
```

Yukarıdaki kod şuna eşdeğerdir:

```rust
// Unit dönüş türünü açıkça yazıyoruz
//                   👇
fn test_welcome() -> () {
    assert_eq!(greeting(), "I'm ready to learn Rust!");
}
```

### Değer Döndürmek {#returning-values}

Bir fonksiyondaki son ifade örtük olarak döndürülür:

```rust
fn greeting() -> &'static str {
    // Bu, fonksiyondaki son ifadedir
    // Bu nedenle değeri `greeting` tarafından döndürülür
    "I'm ready to learn Rust!"
}
```

Bir değeri erken döndürmek için `return` anahtar kelimesini de kullanabilirsin:

```rust
fn greeting() -> &'static str {
    // Satırın sonundaki noktalı virgüle dikkat et!
    return "I'm ready to learn Rust!";
}
```

Mümkün olduğunda `return` anahtar kelimesini yazmamak idiomatik kabul edilir.

### Girdi Parametreleri {#input-parameters}

Girdi parametreleri, fonksiyonun adından sonra gelen `()` parantezlerinin içinde bildirilir.\
Her parametre adıyla bildirilir; adın ardından iki nokta `:`, onun ardından da parametrenin türü gelir.

Örneğin aşağıdaki `greet` fonksiyonu, `&str` türünde (bir "string slice") bir `name` parametresi alır:

```rust
// Bir girdi parametresi
//        👇
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

Birden fazla girdi parametresi varsa virgülle ayrılmaları gerekir.

### Tür Belirtimleri {#type-annotations}

Birkaç kez "türlerden" bahsettiğimize göre, bunu açıkça söyleyelim: Rust **statik türlü bir dildir**.\
Rust'taki her bir değerin bir türü vardır ve bu tür derleme zamanında derleyici tarafından bilinmelidir.

Türler, **statik analizin** bir biçimidir.\
Bir türü, derleyicinin programındaki her değere iliştirdiği bir **etiket** olarak düşünebilirsin. Etikete
bağlı olarak derleyici farklı kurallar uygulayabilir—ör. bir sayıya bir string ekleyemezsin, ama iki sayıyı
birbiriyle toplayabilirsin.
Doğru kullanıldığında türler, çalışma zamanı bug'larının bütün sınıflarını önleyebilir.
