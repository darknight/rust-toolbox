
// TODO:
// 1. refer to https://stackoverflow.com/questions/27791532/how-do-i-create-a-global-mutable-singleton
// 2. refer to std::sync::Once;

// FIXME: make it usable outside of the module
#[derive(Debug, PartialEq)]
struct Singleton;

static INSTANCE: &'static Singleton = &Singleton;

fn get_instance() -> &'static Singleton {
    INSTANCE
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test() {
        let inst1 = get_instance();
        let inst2 = get_instance();

        assert_eq!(inst1, inst2);
        assert_eq!(inst2, INSTANCE);
    }
}