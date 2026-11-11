// TODO: `example`'ın derlenmesi için `From` trait'ini `WrappingU32` türü için implement et.

pub struct WrappingU32 {
    value: u32,
}

fn example() {
    let wrapping: WrappingU32 = 42.into();
    let wrapping = WrappingU32::from(42);
}
