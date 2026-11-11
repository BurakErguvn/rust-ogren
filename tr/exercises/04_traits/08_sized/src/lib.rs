pub fn example() {
    // Bir str'nin (ya da başka herhangi bir DST'nin) boyutunu `std::mem::size_of`
    // aracılığıyla almaya çalışmak bir derleme zamanı hatasıyla sonuçlanır.
    //
    // TODO: Aşağıdaki satırı yorum satırına al ve bir sonraki egzersize geç.
    std::mem::size_of::<str>();
}
