pub mod std;
pub trait Ref {}
impl<T: ?Sized> Ref for &T {}
