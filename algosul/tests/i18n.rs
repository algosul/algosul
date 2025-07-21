use std::{fs::File, io::Read};

use algosul::i18n::{I18n, I18nData};
use algosul_marco::I18n;
use strfmt::strfmt;
#[derive(I18n)]
struct Text {
    #[i18n(ignore)]
    _ignore: (),
    #[i18n(rename = "name")]
    name:    String,
    #[i18n(format(name))]
    format:  String,
}
#[test]
fn main() {
    let lang =
        sys_locale::get_locale().unwrap_or_else(|| String::from("en-US"));
    println!("LANG = {lang}");
    let file_name = format!("tests/rc/lang/{lang}.toml");
    let mut file = File::open(file_name).unwrap();
    let mut toml = String::new();
    file.read_to_string(&mut toml).unwrap();
    let mut text = Text::i18n_from_toml(&toml).unwrap();
    text.check().unwrap();
    text.format = strfmt!(&text.format, name=>text.name.clone()).unwrap();
    println!("{}", text.to_toml().unwrap());
}
