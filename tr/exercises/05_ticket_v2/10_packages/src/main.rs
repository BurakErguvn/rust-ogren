// Bu bir `main.rs` dosyası, bu yüzden `cargo` onu bir binary target'ın kökü olarak yorumlar.

// TODO: bu bozuk import'u düzelt. `src` dizininde yeni bir kütüphane target'ı oluştur.
//   Kütüphane target'ı, argüman almayan ve hiçbir şey döndürmeyen `hello_world` adında
//   public bir fonksiyon sunmalı.
use packages::hello_world;

// Bu, binary'nin giriş noktasıdır.
fn main() {
    hello_world();
}
