# Mutable İndeksleme {#mutable-indexing}

`Index` salt okunur erişime izin verir. Elde ettiğin değeri değiştirmene
izin vermez.

## `IndexMut` {#indexmut}

Mutability'ye (değiştirilebilirliğe) izin vermek istiyorsan `IndexMut` trait'ini implement etmen gerekir.

```rust
// Biraz basitleştirilmiş hali
pub trait IndexMut<Idx>: Index<Idx>
{
    // Zorunlu metot
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output;
}
```

`IndexMut`, _ek_ bir yeteneğin kilidini açtığı için yalnızca tür zaten `Index`'i implement ediyorsa
implement edilebilir.
