use std::error::Error;

use super::observer::*;

enum Source<'a, T> {
    Creator(Box<dyn Fn(Box<dyn Observer<T>>) + 'a>),
    Just(Option<T>),
    Defer(Box<dyn Fn() -> Observable<'a, T> + 'a>),
}

pub struct Observable<'a, T> {
    source: Source<'a, T>
}

impl<'a, T> Observable<'a, T> {

    ///
    /// create an Observable from scratch by means of a function
    ///
    /// F: FnOnce(impl Observer)
    /// `impl Trait` not allowed outside of function and inherent method return types
    ///
    pub fn create<F>(f: F) -> Self where F: Fn(Box<dyn Observer<T>>) + 'a {
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
    pub fn defer<F>(f: F) -> Self where F: Fn() -> Observable<'a, T> + 'a {
        Observable { source: Source::Defer(Box::new(f)) }
    }

    ///
    /// operate upon the emissions and notifications from an Observable
    ///
    /// FIXME: change Fn to FnOnce for error & completed
    ///
    pub fn subscribe<N, E, C>(self, next: N, error: E, completed: C) -> ()
        where N: Fn(T),
              E: Fn(Box<dyn Error>),
              C: Fn() {

        match self.source {
            Source::Creator(creator) => {
                let observer = ObserverOnAll::new(
                    next,
                    error,
                    completed
                );
//                (creator)(Box::new(observer));
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
            observer.on_completed();
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
            fn value_observable<'a>(&self) -> Observable<'a, String> {
                Observable::just(self.value.clone())
            }
        }

        let mut t = SomeType::new();
        let value = t.value_observable();
        t.set_value("new".to_string());
        value.subscribe_on_next(|x| assert_eq!(x, "default".to_string()));
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
            fn value_observable<'a>(&self) -> Observable<'a, String> {
                let val = self.value.clone();
                Observable::defer(move || Observable::just(val.clone()))
            }
        }

        let mut t = SomeType::new();
        let value = t.value_observable();
        t.set_value("new".to_string());
        value.subscribe_on_next(|x| assert_eq!(x, "new".to_string()));
    }
}
