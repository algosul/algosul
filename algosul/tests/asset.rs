use algosul::macros::from_dir;
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
from_dir!(asset, "rc");
#[test]
fn main() {
    let locale = asset::lang::zh_CN_toml;
    println!("{locale}");
}
