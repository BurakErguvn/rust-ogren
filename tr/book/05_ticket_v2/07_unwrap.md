# Unwrap Etmek {#unwrapping}

`Ticket::new` artık geçersiz girdilerde panic'lemek yerine bir `Result` döndürüyor.\
Bu, çağıran için ne anlama geliyor?

## Başarısızlıklar (Örtük Olarak) Yok Sayılamaz {#failures-cant-be-implicitly-ignored}

İstisnaların aksine, Rust'ın `Result`'ı seni **hataları çağrı noktasında ele almaya** zorlar.\
`Result` döndüren bir fonksiyonu çağırırsan, Rust hata durumunu örtük olarak yok saymana izin vermez.

```rust
fn parse_int(s: &str) -> Result<i32, ParseIntError> {
    // ...
}

// Bu derlenmez: hata durumunu ele almıyoruz.
// Başarı değerini "unwrap etmek" ya da hatayı ele almak için ya `match`
// ya da `Result`'ın sağladığı combinator'lardan birini kullanmalıyız.
let number = parse_int("42") + 2;
```

## Bir `Result` Aldın. Şimdi Ne Olacak? {#you-got-a-result-now-what}

`Result` döndüren bir fonksiyonu çağırdığında iki temel seçeneğin vardır:

- İşlem başarısız olduysa panic'le.
  Bu, `unwrap` ya da `expect` metotlarından biri kullanılarak yapılır.
  ```rust
  // `parse_int` bir `Err` döndürürse panic'ler.
  let number = parse_int("42").unwrap();
  // `expect` özel bir panic mesajı belirtmeni sağlar.
  let number = parse_int("42").expect("Failed to parse integer");
  ```
- Hata durumunu açıkça ele almak için bir `match` ifadesiyle `Result`'ı destructure et (parçalarına ayır).
  ```rust
  match parse_int("42") {
      Ok(number) => println!("Parsed number: {}", number),
      Err(err) => eprintln!("Error: {}", err),
  }
  ```
