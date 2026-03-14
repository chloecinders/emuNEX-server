use std::ops::Deref;
use std::sync::OnceLock;

pub struct AutoOnceLock<T>(OnceLock<T>);

impl<T> AutoOnceLock<T> {
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }

    pub fn set(&self, v: T) -> Result<(), T> {
        self.0.set(v)
    }

    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        self.0.get_or_init(f)
    }
}

impl<T> Deref for AutoOnceLock<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self.0.get() {
            Some(v) => v,
            _ => panic!("not initialized"),
        }
    }
}
