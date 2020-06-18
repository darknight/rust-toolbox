/// Currently implementation will use blocking queue first
///
/// refer to
/// https://www.geeksforgeeks.org/thread-pools-java/
/// https://caffinc.github.io/2016/03/simple-threadpool/
///
/// Caveat:
/// 1. avoid dead lock
/// 2. avoid thread leakage
/// 3. avoid resource thrashing
/// 4. tune pool size based on task type (CPU bound or IO bound)
/// 5. shutdown/terminate explicitly

// TODO: implement concurrent linked queue, then replace blocking queue with it
// TODO: refer other implementation
// 1) https://github.com/rayon-rs/rayon/tree/master/rayon-core/src/thread_pool
// 2) https://github.com/rust-threadpool/rust-threadpool


use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use data_structure::thread_safe_blocking_queue::BlockingQueue;
use concurrency::atomic::atomic_boolean::AtomicBoolean;

///
/// refer to
/// https://stackoverflow.com/questions/30557152/is-it-possible-to-send-closures-via-channels
///
type TaskQueue = BlockingQueue<Box<Fn() + Send + 'static>>;

trait Executor {
    fn execute<F>(&self, f: F) where F: Fn() + Send + 'static;
}

pub struct FixedThreadPool {
    pub name: String,
    pub size: u32,
    running: AtomicBoolean,
    queue: TaskQueue,
    threads: Vec<JoinHandle<()>>
}

struct ManagedThread {
    running: AtomicBoolean,
    queue: TaskQueue,
}

impl ManagedThread {

    fn new(running: AtomicBoolean, queue: TaskQueue) -> ManagedThread {
        ManagedThread {
            running: running,
            queue: queue
        }
    }

    fn run(&self) {
        while self.running.get() || !self.queue.is_empty() {
            println!("{} is running...", thread::current().name().unwrap());
            match self.queue.poll() {
                Some(task) => {
                    println!("{} found task, run it...", thread::current().name().unwrap());
                    // here if use FnOnce, compiler fails
                    // TODO: capture exception to avoid running thread exit
                    (task)();
                }
                None => {
                    println!("{} found nothing, sleep for a while...", thread::current().name().unwrap());
                    thread::sleep(Duration::from_millis(1000))
                }
            }
        }

        println!("{} finished running...", thread::current().name().unwrap());
    }
}

impl FixedThreadPool {

    pub fn new(size: u32) -> FixedThreadPool {
        Self::new_with_name(size, "default-poll".to_string())
    }

    pub fn new_with_name(size: u32, name: String) -> FixedThreadPool {
        let running = AtomicBoolean::new(true);
        let queue = BlockingQueue::new();
        let mut threads = vec![];

        for i in 0..size {
            let _running = running.clone();
            let _queue = queue.clone();

            let handle = thread::Builder::new()
                .name(format!("[T{}]", i).to_string())
                .spawn(move || {
                    let managed_thread = ManagedThread::new(_running, _queue);
                    managed_thread.run();
                })
                .unwrap();
            threads.push(handle);
        }

        FixedThreadPool {
            name: name,
            size: size,
            running: running,
            queue: queue,
            threads: threads
        }
    }

    /// stop thread pool, but not purge the blocking queue
    /// so all the pending tasks will be executed
    pub fn stop(&self) {
        self.running.set(false);
    }

    /// shutdown thread pool, will purge the blocking queue
    /// so it will exit when current running tasks has been completed
    pub fn shutdown(&self) {
        self.stop();
        self.queue.clear();
    }
}

impl Executor for FixedThreadPool {

    /// if there's no `'static` lifetime
    /// the compiler complains
    /// `the parameter type `F` may not live long enough`
    fn execute<F>(&self, f: F) where F: Fn() + Send + 'static {
        if self.running.get() {
            self.queue.add(Box::new(f));
        } else {
            panic!("thread pool has stopped to accept new tasks");
        }
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    fn block_main_thread() {
        for i in 0..5 {
            thread::sleep(Duration::from_secs(1));
        }
    }

    #[test]
    fn test_pool_size_1() {
        let pool = FixedThreadPool::new(1);
        assert_eq!(pool.running.get(), true);
        block_main_thread()
    }

    #[test]
    fn test_pool_size_4() {
        let pool = FixedThreadPool::new_with_name(4, "TestPool".to_string());
        assert_eq!(pool.running.get(), true);
        block_main_thread()
    }

    #[test]
    fn test_pool_with_task() {
        let pool = FixedThreadPool::new(2);

        for i in 0..10 {
            pool.execute(move || {
                println!("TASK-{} is executed", i);
            });
        }

        block_main_thread();
        assert_eq!(pool.queue.is_empty(), true);
    }

    #[test]
    fn test_pool_with_stop() {
        let pool = FixedThreadPool::new(2);

        for i in 0..4 {
            pool.execute(move || {
                println!("TASK-{} is executed & run a long time", i);
                thread::sleep(Duration::from_secs(10));
            });
        }

        // block for a while to let thread pool pick up first 2 tasks
        block_main_thread();

        assert_eq!(pool.queue.size(), 2);
        pool.stop();
        assert_eq!(pool.queue.size(), 2);

        block_main_thread();
        assert_eq!(pool.queue.is_empty(), true);
    }

    #[test]
    fn test_pool_with_shutdown() {
        let pool = FixedThreadPool::new(2);

        for i in 0..4 {
            pool.execute(move || {
                println!("TASK-{} is executed & run a long time", i);
                thread::sleep(Duration::from_secs(10));
            });
        }

        // block for a while to let thread pool pick up first 2 tasks
        block_main_thread();

        assert_eq!(pool.queue.size(), 2);
        pool.shutdown();
        assert_eq!(pool.queue.size(), 0);

        block_main_thread();
        assert_eq!(pool.queue.is_empty(), true);
    }
}
