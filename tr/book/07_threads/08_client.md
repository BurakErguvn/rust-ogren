# Özel Bir `Client` Türü {#a-dedicated-client-type}

İstemci tarafındaki tüm etkileşimler oldukça düşük seviyeliydi: bir yanıt kanalını
elle oluşturman, komutu kurman, onu sunucuya göndermen ve
ardından yanıtı almak için yanıt kanalı üzerinde `recv` çağırman gerekiyor.

Bu, soyutlanarak gizlenebilecek bir sürü boilerplate kodudur ve bu egzersizde
tam olarak bunu yapacağız.
