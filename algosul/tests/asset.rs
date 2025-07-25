// const ARGS: FormatArgs = FormatArgs::new(["name"]);
// #[derive(Debug, Serialize, Deserialize)]
// struct Asset {
//     text:  FormatText,
//     image: Image,
// }
// #[derive(Debug, Serialize, Deserialize)]
// struct Asset {
//     #[asset(format_text(args = ["name"]))]
//     text:  FormatText,
//     image: Image,
// }
mod assets {
    use algosul_derive::from_dir;
    from_dir!(pub mod lang from "rc/lang" filter [include ["*.toml"] exclude []]);
    from_dir!(pub mod images from "rc/images" filter [include ["*.png"] exclude []]);
}
#[test]
fn main() {
    let locale = assets::lang::en_US;
    println!("{locale}");
}
