# Kapanış {#outro}

Rust'ın asenkron modeli oldukça güçlüdür, ama beraberinde ek karmaşıklık da
getirir. Araçlarını tanımaya zaman ayır: `tokio`'nun dokümantasyonunun derinlerine dal
ve ondan en iyi şekilde yararlanmak için primitive'lerine aşina ol.

Ayrıca, Rust'ın asenkron hikâyesini sadeleştirmek ve "tamamlamak" için dil ve `std`
düzeyinde süren çalışmalar olduğunu da unutma. Bu eksik parçaların bazıları yüzünden
günlük işlerinde bazı pürüzlerle karşılaşabilirsin.

Çoğunlukla acısız bir async deneyimi için birkaç öneri:

- **Bir runtime seç ve ona bağlı kal.**\
  Bazı primitive'ler (ör. zamanlayıcılar, I/O) runtime'lar arasında taşınabilir değildir. Runtime'ları
  karıştırmaya çalışmak büyük olasılıkla başını ağrıtır. Runtime'dan bağımsız kod
  yazmaya çalışmak kod tabanının karmaşıklığını önemli ölçüde artırabilir. Elinden geliyorsa
  bundan kaçın.
- **Henüz kararlı bir `Stream`/`AsyncIterator` arayüzü yok.**\
  Bir `AsyncIterator`, kavramsal olarak, yeni öğeleri asenkron olarak üreten bir
  iterator'dır. Süren bir tasarım çalışması var, ama (henüz) bir uzlaşı yok.
  `tokio` kullanıyorsan, başvuracağın arayüz olarak [`tokio_stream`](https://docs.rs/tokio-stream/latest/tokio_stream/)
  kullan.
- **Buffer'lamaya dikkat et.**\
  Genellikle zor fark edilen bug'ların nedenidir. Daha fazla ayrıntı için
  ["Barbara battles buffered streams"](https://rust-lang.github.io/wg-async/vision/submitted_stories/status_quo/barbara_battles_buffered_streams.html)
  yazısına göz at.
- **Asenkron task'ler (görevler) için scoped thread'lerin bir karşılığı yok**.\
  Daha fazla ayrıntı için ["The scoped task trilemma"](https://without.boats/blog/the-scoped-task-trilemma/)
  yazısına göz at.

Bu uyarıların seni korkutmasına izin verme: asenkron Rust, temel servislere güç vermek için
_devasa_ ölçekte (ör. AWS, Meta) etkili bir şekilde kullanılıyor.\
Rust'ta ağ uygulamaları geliştirmeyi planlıyorsan ona hâkim olman gerekecek.
