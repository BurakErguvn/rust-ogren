# İki Yönlü İletişim {#two-way-communication}

Mevcut istemci-sunucu implementation'ımızda iletişim tek yönde akıyor: istemciden sunucuya.\
İstemcinin, sunucunun mesajı alıp almadığını, onu başarıyla çalıştırıp çalıştırmadığını ya da başarısız olup olmadığını bilmesinin hiçbir yolu yok.
Bu ideal değil.

Bu sorunu çözmek için iki yönlü bir iletişim sistemi getirebiliriz.

## Yanıt Kanalı {#response-channel}

Sunucunun istemciye bir yanıt geri göndermesinin bir yoluna ihtiyacımız var.\
Bunu yapmanın çeşitli yolları var, ama en basit seçenek, istemcinin sunucuya gönderdiği mesajın içine
bir `Sender` kanalı koymaktır. Sunucu mesajı işledikten sonra bu kanalı kullanarak istemciye
bir yanıt geri gönderebilir.

Bu, mesaj iletimi primitive'leri üzerine kurulmuş Rust uygulamalarında oldukça yaygın bir kalıptır.
