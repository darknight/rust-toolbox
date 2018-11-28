
struct Subject<'a, T: 'a + Observer> {
    observers: Vec<Box<&'a T>>,
}

trait Observer {
    fn update(&self);
}

impl<'a, T> Subject<'a, T> where T: 'a + Observer + Ord {
    fn new() -> Subject<'a, T> {
        Subject {
            observers: Vec::new()
        }
    }

    fn attach(&mut self, o: &'a T) {
        self.observers.push(Box::new(o));
        self.observers.sort();
    }

    fn detach(&mut self, target: &'a T) -> Option<&'a T> {
        let res = self.observers.binary_search(&Box::new(target));
        match res {
            Ok(idx) => {
                Some(*self.observers.remove(idx))
            },
            Err(_) => None
        }
    }

    fn notify(&self) {
        self.observers.iter()
            .for_each(|o| {
                o.update();
            });
    }

    fn change_state(&mut self) {
        println!("change state in subject, then notify all the observers");
        self.notify();
    }
}

///
/// user cargo test `-- --nocapture` test_subject
/// or the stdout of successful test will be hidden
///
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialOrd, PartialEq, Ord, Eq)]
    struct TestObserver { name: String }
    impl Observer for TestObserver {
        fn update(&self) {
            println!("[Observer] {} has been updated", self.name);
        }
    }

    #[test]
    fn test_subject() {
        let o1 = TestObserver { name: "Observer1".to_string() };
        let o2 = TestObserver { name: "Observer2".to_string() };
        let o3 = TestObserver { name: "Observer3".to_string() };

        let mut sub = Subject::new();
        sub.attach(&o1);
        sub.attach(&o2);
        sub.attach(&o3);

        sub.change_state();

        sub.detach(&o1);
        sub.change_state();
    }
}