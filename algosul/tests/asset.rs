use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
struct Asset {}
#[test]
fn main() { (toml::Serializer::new("Hi")); }
