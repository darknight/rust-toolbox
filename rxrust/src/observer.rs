use std::error::Error;

pub trait Observer<T> {

    fn on_next(&self, item: T) {}

    fn on_error(&self, e: Box<dyn Error>) {}

    fn on_completed(&self) {}
}

pub struct ObserverOnNext<T> {
    next_func: Box<Fn(T)>
}

pub struct ObserverOnAll<T> {
    next_func: Box<Fn(T)>,
    error_func: Box<Fn(Box<dyn Error>)>,
    completed_func: Box<Fn()>
}

impl<T> ObserverOnNext<T> {
    pub fn new(next_func: Box<Fn(T)>) -> Self {
        ObserverOnNext { next_func }
    }
}

impl<T> ObserverOnAll<T> {
    pub fn new(next_func: Box<Fn(T)>,
               error_func: Box<Fn(Box<dyn Error>)>,
               completed_func: Box<Fn()>) -> Self {
        ObserverOnAll {
            next_func,
            error_func,
            completed_func
        }
    }
}

impl<T> Observer<T> for ObserverOnNext<T> {
    fn on_next(&self, item: T) {
        (self.next_func)(item)
    }
}

impl<T> Observer<T> for ObserverOnAll<T> {
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
