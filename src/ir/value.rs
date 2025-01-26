use std::hash::{Hash, Hasher};

pub trait Value {
    fn name(&self) -> &str;

    fn is_lvalue(&self) -> bool {
        false
    }

    fn addr(&self) -> *const () {
        self as *const Self as *const ()
    }
}

impl PartialEq for &dyn Value {
    fn eq(&self, other: &Self) -> bool {
        // NOTE DO NOT USE `std::ptr::eq`: https://doc.rust-lang.org/std/ptr/fn.eq.html
        std::ptr::addr_eq(*self, *other)
    }
}

impl Eq for &dyn Value {}

impl Hash for &dyn Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.addr(), state)
    }
}
