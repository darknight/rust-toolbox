/// idea from java.util.concurrent.atomic.AtomicBoolean
/// but java version is lock-free thread-safe
// TODO: make implementation same as java

use std::sync::Mutex;
use std::sync::Arc;

#[derive(Debug)]
pub struct AtomicBoolean {
    value: Arc<Mutex<bool>>
}

impl AtomicBoolean {

    pub fn new(val: bool) -> AtomicBoolean {
        AtomicBoolean {
            value: Arc::new(Mutex::new(val))
        }
    }

    pub fn get(&self) -> bool {
        let b = self.value.lock().unwrap();
        *b
    }

    pub fn set(&self, val: bool) {
        let mut b = self.value.lock().unwrap();
        *b = val;
    }
}

impl Clone for AtomicBoolean {
    fn clone(&self) -> Self {
        AtomicBoolean {
            value: Arc::clone(&self.value)
        }
    }
}

mod tests {

    use super::*;

    #[test]
    fn basic_test() {
        let ab = AtomicBoolean::new(false);
        assert_eq!(ab.get(), false);

        ab.set(true);
        assert_eq!(ab.get(), true);
    }

    // TODO: cooperate with AtomicInteger
    /// refer to
    /// https://stackoverflow.com/questions/17414924/how-to-test-atomicboolean-atomicity
    #[test]
    fn test_concurrent_access() {
        assert!(true)
    }
}
