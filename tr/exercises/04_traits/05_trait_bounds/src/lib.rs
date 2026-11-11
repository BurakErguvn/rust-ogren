// TODO: `min`'in başarıyla derlenmesi için ona gerekli trait bound'ları ekle.
//   İhtiyaç duyabileceğin trait'ler hakkında daha fazla bilgi için `std::cmp` modülünün
//   dokümantasyonuna bak.
//
// Not: derleyiciyi memnun edecek farklı trait bound'lar var, ama bunların _semantiği_ farklıdır.
// Bu farkları kursun ilerleyen kısımlarında, sıralı koleksiyonlardan (örneğin BTreeMap) bahsederken
// ele alacağız.

/// İki değerden küçük olanı döndür.
pub fn min<T>(left: T, right: T) -> T {
    if left <= right {
        left
    } else {
        right
    }
}
