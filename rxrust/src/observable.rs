use std::error::Error;

use super::observer::*;

enum Source<'a, T> {
    Creator(Box<dyn FnOnce(Box<dyn Observer<T> + 'a>) + 'a>),
    Just(Option<T>),
    Defer(Box<dyn FnOnce() -> Observable<'a, T> + 'a>),
}

pub struct Observable<'a, T> {
    source: Source<'a, T>
}

impl<'a, T> Observable<'a, T> where T: 'a {

    ///
    /// create an Observable from scratch by means of a function
    ///
    /// F: FnOnce(impl Observer)
    /// `impl Trait` not allowed outside of function and inherent method return types
    ///
    pub fn create<F>(f: F) -> Self where F: FnOnce(Box<dyn Observer<T> + 'a>) + 'a {
        Observable { source: Source::Creator(Box::new(f)) }
    }

    ///
    /// create an Observable that emits a particular item
    ///
    pub fn just(item: T) -> Self {
        Observable { source: Source::Just(Some(item)) }
    }

    ///
    /// do not create the Observable until the observer subscribes,
    /// and create a fresh Observable for each observer
    ///
    pub fn defer<F>(f: F) -> Self where F: FnOnce() -> Observable<'a, T> + 'a {
        Observable { source: Source::Defer(Box::new(f)) }
    }

    ///
    /// operate upon the emissions and notifications from an Observable
    ///
    pub fn subscribe<N, E, C>(self, next: N, error: E, completed: C) -> ()
        where N: Fn(T) + 'a,
              E: FnOnce(Box<dyn Error>) + 'a,
              C: FnOnce() + 'a {

        match self.source {
            Source::Creator(creator) => {
                let observer = ObserverOnAll::new(
                    next,
                    error,
                    completed
                );
                let x = Box::new(observer);
                (creator)(x);
            },
            _ => unimplemented!()
        }
    }

    ///
    /// subscribe with only on_next
    ///
    pub fn subscribe_on_next<F>(self, next: F) -> () where F: Fn(T) {
        match self.source {
            Source::Just(Some(item)) => {
                let observer = ObserverOnNext::new(
                    next
                );
                observer.on_next(item);
            },
            Source::Defer(f) => {
                let observable: Observable<T> = (f)();
                observable.subscribe_on_next(next);
            }
            _ => unimplemented!()
        }
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
            (*observer).on_completed();
        });

        source.subscribe(
            |x| { println!("Next => {}", x); },
            |_e| { println!("Error!"); },
            || { println!("Completed!") }
        )
    }

    ///
    /// https://blog.danlew.net/2015/07/23/deferring-observable-code-until-subscription-in-rxjava/
    ///
    #[test]
    fn test_just() {
        struct SomeType { value: String };
        impl SomeType {
            fn new() -> SomeType { SomeType { value: "default".to_string() } }
            fn set_value(&mut self, value: String) { self.value = value; }
            fn value_observable<'a>(&'a self) -> Observable<'a, &String> {
                Observable::just(&self.value)
            }
        }

        let mut t = SomeType::new();
        let value = t.value_observable();
        value.subscribe_on_next(|x| assert_eq!(x, &"default".to_string()));
    }

    ///
    /// https://blog.danlew.net/2015/07/23/deferring-observable-code-until-subscription-in-rxjava/
    ///
    #[test]
    fn test_defer() {
        struct SomeType { value: String };
        impl SomeType {
            fn new() -> SomeType { SomeType { value: "default".to_string() } }
            fn set_value(&mut self, value: String) { self.value = value; }
            fn value_observable<'a>(&'a self) -> Observable<'a, &String> {
                let f = move || Observable::just(&self.value);
                Observable::defer(f)
            }
        }

        let t = SomeType::new();
        let value = t.value_observable();
        value.subscribe_on_next(|x| assert_eq!(x, &"default".to_string()));
    }
}
