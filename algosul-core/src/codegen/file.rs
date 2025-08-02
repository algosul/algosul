use syn::{File, Item, ItemMod};
pub trait FileExt {
    fn add_item(&mut self, item: Item) -> &mut Self;
    fn add_mod(&mut self, item_mod: ItemMod) -> &mut Self {
        self.add_item(Item::Mod(item_mod))
    }
}
impl FileExt for File {
    fn add_item(&mut self, item: Item) -> &mut Self {
        self.items.push(item);
        self
    }
}
