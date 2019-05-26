use std::error::Error;

use super::observer::*;

pub struct Observable<T> {
    creator: Box<Fn(Box<dyn Observer<T>>)>
}

// FIXME: get rid of 'static lifetime
impl<T> Observable<T> where T: 'static {

    ///
    /// create an Observable from scratch by means of a function
    ///
    /// F: FnOnce(impl Observer)
    /// `impl Trait` not allowed outside of function and inherent method return types
    ///
    fn create<F>(f: F) -> Self where F: Fn(Box<dyn Observer<T>>) + Sized + 'static {
        Observable {
            creator: Box::new(f),
        }
    }

    ///
    /// operate upon the emissions and notifications from an Observable
    ///
    /// FIXME: change Fn to FnOnce for error & completed
    ///
    fn subscribe<N, E, C>(&self, next: N, error: E, completed: C) -> ()
        where N: Fn(T) + 'static,
              E: Fn(Box<dyn Error>) + 'static,
              C: Fn() + 'static {
        let observer = ObserverOnAll::new(
            Box::new(next),
            Box::new(error),
            Box::new(completed)
        );

        (self.creator)(Box::new(observer));
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_create() {
        let source = Observable::create(|observer| {
            for i in 1..5 {
                observer.on_next(i);
            }
            observer.on_completed();
        });

        source.subscribe(
            |x| { println!("Next => {}", x); },
            |e| { println!("Error!"); },
            || { println!("Completed!") }
        )
    }
}
