# Bir Ticket'ı Modellemek {#modelling-a-ticket}

İlk bölüm sana Rust'ın ilkel türlerinin, operatörlerinin ve temel kontrol akışı yapılarının bir kısmını
iyice kavratmış olmalı.\
Bu bölümde bir adım daha ileri gidip Rust'ı gerçekten eşsiz kılan şeyi ele alacağız: **ownership** (sahiplik).\
Ownership, Rust'ın garbage collector (çöp toplayıcı) olmadan hem bellek açısından güvenli hem de performanslı olmasını sağlayan şeydir.

Bölüm boyunca örnek olarak, bir yazılım projesinde bug'ları, özellikleri ya da görevleri takip etmek için kullanacağın türden
(JIRA benzeri) bir ticket kullanacağız.\
Onu Rust'ta modellemeyi deneyeceğiz. Bu ilk sürüm olacak—bölümün sonunda ne mükemmel ne de çok idiomatik
olacak. Yine de yeterince zorlayıcı olacak!\
İlerlemek için birkaç yeni Rust kavramını öğrenmen gerekecek; örneğin:

- `struct`'lar, Rust'ta özel türler tanımlamanın yollarından biri
- Ownership, referanslar ve borrowing (ödünç alma)
- Bellek yönetimi: stack, heap, pointer'lar (işaretçiler), veri düzeni, destructor'lar (yıkıcılar)
- Modüller ve görünürlük
- String'ler
