// TODO: bu bir orphan rule ihlali örneğidir.
//  Yabancı bir trait'i (`PartialEq`, `std`'den) yabancı bir tür
//  (`u32`, `std`'den) için implement ediyoruz.
//  Neye benzediğine aşina olmak için derleyici hatasına bak.
//  Ardından aşağıdaki kodu sil ve bir sonraki egzersize geç.

impl PartialEq for u32 {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}
