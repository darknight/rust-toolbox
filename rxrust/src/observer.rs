use std::error::Error;
use std::marker::PhantomData;

pub trait Observer<T> {

    fn on_next(&self, _item: T) {}

    fn on_error(&self, _e: Box<dyn Error>) {}

    fn on_completed(&self) {}
}

pub struct ObserverOnNext<T, F> where F: Fn(T) {
    next_func: F,
    marker: PhantomData<Fn(T)>,
}

pub struct ObserverOnAll<T, N, E, C> where N: Fn(T), E: Fn(Box<dyn Error>), C: Fn() {
    next_func: N,
    error_func: E,
    completed_func: C,
    marker: PhantomData<Fn(T)>,
}

impl<T, F> ObserverOnNext<T, F> where F: Fn(T)  {
    pub fn new(next_func: F) -> Self {
        ObserverOnNext { next_func, marker: PhantomData }
    }
}

impl<T, N, E, C> ObserverOnAll<T, N, E, C> where N: Fn(T), E: Fn(Box<dyn Error>), C: Fn() {
    pub fn new(next_func: N,
               error_func: E,
               completed_func: C) -> Self {
        ObserverOnAll {
            next_func,
            error_func,
            completed_func,
            marker: PhantomData
        }
    }
}

impl<T, F> Observer<T> for ObserverOnNext<T, F> where F: Fn(T) {
    fn on_next(&self, item: T) {
        (self.next_func)(item)
    }
}

impl<T, N, E, C> Observer<T> for ObserverOnAll<T, N, E, C> where N: Fn(T), E: Fn(Box<dyn Error>), C: Fn() {
    fn on_next(&self, item: T) {
        (self.next_func)(item)
    }

    fn on_error(&self, e: Box<dyn Error>) {
        (self.error_func)(e)
    }

    fn on_completed(&self) {
        (self.completed_func)()
    }
}
