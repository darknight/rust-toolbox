use std::sync::Mutex;

struct RWLock {
    r: Mutex<usize>,
    g: Mutex<()>,
}

impl RWLock {

//    fn lock(&self) {
//        let mut b = self.r.lock();
//        *b += 1;
//        if *b == 1 {
//            self.g.lock();
//        }
//        b.unlock();
//    }
//
//    fn unlock(&self) {
//        let mut b = self.r.lock();
//        *b -= 1;
//        if *b == 0 {
//            self.g.unlock();
//        }
//        b.unlock();
//    }

}
