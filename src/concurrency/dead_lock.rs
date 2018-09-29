use std::thread;
use std::thread::JoinHandle;
use std::sync::{Mutex, Arc};
use std::time::Duration;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_dead_lock() {
        let lock_res1 = Arc::new(Mutex::new(1));
        let lock_res2 = Arc::new(Mutex::new("res2"));

        let wait_res1 = lock_res1.clone();
        let wait_res2 = lock_res2.clone();

        let handle1 = thread::Builder::new()
            .name("thread-1".to_string())
            .spawn(move || -> () {
                let res1 = lock_res1.lock().unwrap();
                println!("{} got res1: {:?}", thread::current().name().unwrap(), *res1);

                thread::sleep(Duration::from_secs(3));

                println!("{} tried to acquire res2", thread::current().name().unwrap());
                let res2 = wait_res2.lock().unwrap();

                unreachable!();
            })
            .unwrap();

        let handle2 = thread::Builder::new()
            .name("thread-2".to_string())
            .spawn(move || -> (){
                let res2 = lock_res2.lock().unwrap();
                println!("{} got res2: {:?}", thread::current().name().unwrap(), *res2);

                thread::sleep(Duration::from_secs(3));

                println!("{} tried to acquire res1", thread::current().name().unwrap());

                let res1 = wait_res1.lock().unwrap();

                unreachable!();
            })
            .unwrap();

        handle1.join();
        handle2.join();
        unreachable!();
    }

}