use tokio::net::TcpListener;

// TODO: gelen TCP bağlantılarını kabul eden ve alınan veriyi istemciye geri gönderen
//  bir echo sunucusu yaz.
//  `echo`, bir bağlantıyı işlemeyi bitirdiğinde dönmemeli; yeni bağlantıları kabul etmeye
//  devam etmeli.
//
// İpucu: echo sunucusunu implement etmek için `tokio`'nun struct'larına ve metotlarına dayanmalısın.
// Özellikle:
// - gelen bir sonraki bağlantıyı işlemek için `tokio::net::TcpListener::accept`
// - soketten bir okuyucu ve bir yazıcı elde etmek için `tokio::net::TcpStream::split`
// - veriyi okuyucudan yazıcıya kopyalamak için `tokio::io::copy`
pub async fn echo(listener: TcpListener) -> Result<(), anyhow::Error> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_echo() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(echo(listener));

        let requests = vec!["hello", "world", "foo", "bar"];

        for request in requests {
            let mut socket = tokio::net::TcpStream::connect(addr).await.unwrap();
            let (mut reader, mut writer) = socket.split();

            // İsteği gönder
            writer.write_all(request.as_bytes()).await.unwrap();
            // Soketin yazma tarafını kapat
            writer.shutdown().await.unwrap();

            // Yanıtı oku
            let mut buf = Vec::with_capacity(request.len());
            reader.read_to_end(&mut buf).await.unwrap();
            assert_eq!(&buf, request.as_bytes());
        }
    }
}
