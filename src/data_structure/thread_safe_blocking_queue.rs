use std::sync::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct BlockingQueue<T> {
    queue: Arc<Mutex<VecDeque<T>>>
}

impl<T> BlockingQueue<T> {

    pub fn new() -> BlockingQueue<T> {
        BlockingQueue {
            queue: Arc::new(Mutex::new(VecDeque::new()))
        }
    }

    pub fn add(&self, item: T) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(item);
    }

    pub fn poll(&self) -> Option<T> {
        let mut queue = self.queue.lock().unwrap();
        queue.pop_front()
    }

    pub fn clear(&self) {
        let mut queue = self.queue.lock().unwrap();
        queue.clear();
    }

    pub fn is_empty(&self) -> bool {
        let queue = self.queue.lock().unwrap();
        queue.is_empty()
    }
}

impl<T> Clone for BlockingQueue<T> {
    fn clone(&self) -> Self {
        BlockingQueue {
            queue: Arc::clone( &self.queue)
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_single_thread() {
        let queue = BlockingQueue::new();
        let res1 = queue.poll();

        assert_eq!(res1, None);
        assert_eq!(queue.is_empty(), true);

        queue.add(3);
        queue.add(2);
        queue.add(1);

        assert_eq!(queue.is_empty(), false);

        let res2 = queue.poll().unwrap();

        assert_eq!(res2, 3);
        assert_eq!(queue.is_empty(), false);

        queue.clear();
        let res3 = queue.poll();

        assert_eq!(res3, None);
        assert_eq!(queue.is_empty(), true);
    }

    #[test]
    fn test_multi_thread() {
        let queue = BlockingQueue::new();
        let queue1 = queue.clone();
        let queue2 = queue.clone();

        let handle1 = thread::Builder::new()
            .name("1".to_string())
            .spawn(move || {
                let mut sum = 0;
                loop {
                    if let Some(i) = queue1.poll() {
                        if i > 100 { break; }

                        sum += i;
                        if i % 2 == 0 {
                            thread::sleep(Duration::from_millis(200));
                        } else {
                            thread::sleep(Duration::from_millis(100));
                        }
                    } else {
                        thread::sleep(Duration::from_millis(300));
                    }
                }
                sum
            }).unwrap();

        let handle2 = thread::Builder::new()
            .name("2".to_string())
            .spawn(move || {
                let mut sum = 0;
                loop {
                    if let Some(i) = queue2.poll() {
                        if i > 100 { break; }
                        sum += i;
                        if i % 2 == 0 {
                            thread::sleep(Duration::from_millis(200));
                        } else {
                            thread::sleep(Duration::from_millis(100));
                        }
                    } else {
                        thread::sleep(Duration::from_millis(300));
                    }
                }
                sum
            }).unwrap();

        for i in 1..106 {
            queue.add(i);
            thread::sleep(Duration::from_millis(150));
        }

        let sum1 = handle1.join().unwrap_or_default();
        let sum2 = handle2.join().unwrap_or_default();

        assert_eq!(sum1 + sum2, 5050);
    }

    #[test]
    fn test_fn() {
        let queue = BlockingQueue::new();
        let f = move || false;
        queue.add(f);
        let res = queue.poll().unwrap();
        assert_eq!(res(), false);
    }
}
