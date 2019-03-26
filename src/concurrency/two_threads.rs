///
/// Two threads A & B, A prints 1, 3, 5..., B prints 2, 4, 6...
/// Console output: 1, 2, 3, 4...
///
use std::thread;
use std::sync::Mutex;
use std::sync::Arc;
use std::time::Duration;
use std::sync::mpsc;

// use shared state
fn thread_print_by_mutex() {

    let mutex = Arc::new(Mutex::new(1u32));

    let odd_mutex = Arc::clone(&mutex);
    let odd_thread = thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(100));
            let mut counter = odd_mutex.lock().unwrap();
            if *counter & 0x00000001 == 1 {
                println!("{}", *counter);
                *counter += 1;
            }
        }
    });

    let even_mutex = Arc::clone(&mutex);
    let even_thread = thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(100));
            let mut counter = even_mutex.lock().unwrap();
            if *counter & 0x00000001 == 0 {
                println!("{}", *counter);
                *counter += 1;
            }
        }
    });

    odd_thread.join();
    even_thread.join();
}

fn thread_print_by_channel() {

    let (odd_tx, odd_rx) = mpsc::channel();
    let (even_tx, even_rx) = mpsc::channel();

    let main_tx = mpsc::Sender::clone(&odd_tx);

    let odd_thread = thread::spawn(move || {
        for mut data in odd_rx {
            data += 1;
            println!("{}", data);
            thread::sleep(Duration::from_millis(100));
            even_tx.send(data).unwrap();
        }
    });

    let even_thread = thread::spawn(move || {
        for mut data in even_rx {
            data += 1;
            println!("{}", data);
            thread::sleep(Duration::from_millis(100));
            odd_tx.send(data).unwrap();
        }
    });

    main_tx.send(0);
    odd_thread.join();
    even_thread.join();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        thread_print_by_mutex()
    }

    #[test]
    fn test2() {
        thread_print_by_channel()
    }
}