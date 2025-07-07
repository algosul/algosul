pub mod command;
pub mod lang;
pub mod profile;
pub mod project;
pub mod utils;
/// the app name
pub const NAME: &str = env!("CARGO_PKG_NAME");
/// the app version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
