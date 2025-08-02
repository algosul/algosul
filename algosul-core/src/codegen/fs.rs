use std::{error::Error, path::Path};

pub trait FromPath: Sized {
    type Error: Error;
    fn from_path(path: impl AsRef<Path>) -> Result<Self, Self::Error>;
}
// impl FromPath for ItemMod {
//     type Error = io::Error;
//
//     fn from_path(path: impl AsRef<Path>) -> Result<Self, Self::Error> {
//         let item =
//         for entry in read_dir(path)? {
//             let entry = entry?;
//
//         }
//         Ok(())
//     }
// }
