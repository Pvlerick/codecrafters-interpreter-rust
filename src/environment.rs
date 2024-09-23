use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug)]
pub struct Environment<T>
where
    T: Clone,
{
    inner: Rc<RefCell<Inner<T>>>,
}

impl<T> Environment<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner::new())),
        }
    }

    pub fn enclose(&self) -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner::enclose(self.inner.clone()))),
        }
    }

    pub fn define<K: ToString>(&self, key: K, value: T) {
        self.inner.borrow_mut().define(key, value);
    }

    pub fn assign<K: ToString>(&self, key: K, value: T) -> Result<(), ()> {
        self.inner.borrow_mut().assign(key, value)
    }

    pub fn get<K: ToString>(&self, key: K) -> Option<T> {
        self.inner.borrow().get(key)
    }
}

#[derive(Debug)]
struct Inner<T>
where
    T: Clone,
{
    enclosing: Option<Rc<RefCell<Inner<T>>>>,
    values: HashMap<String, T>,
}

impl<T> Inner<T>
where
    T: Clone,
{
    fn new() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    fn enclose(inner: Rc<RefCell<Inner<T>>>) -> Self {
        Self {
            enclosing: Some(inner.clone()),
            values: HashMap::new(),
        }
    }

    fn define<K: ToString>(&mut self, key: K, value: T) {
        self.values.insert(key.to_string(), value);
    }

    pub fn assign<K: ToString>(&mut self, key: K, value: T) -> Result<(), ()> {
        let key_s = key.to_string();
        if self.values.contains_key(&key_s) {
            self.values.insert(key_s, value);
            Ok(())
        } else {
            match &self.enclosing {
                Some(inner) => inner.borrow_mut().assign(key, value),
                None => Err(()),
            }
        }
    }

    pub fn get<K: ToString>(&self, key: K) -> Option<T> {
        let key_s = key.to_string();
        match self.values.get(&key_s) {
            Some(value) => Some(value.clone()),
            None => match &self.enclosing {
                Some(inner) => inner.borrow().get(key).clone(),
                None => None,
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty() {
        let sut = Environment::<u32>::new();
        assert_eq!(None, sut.get("foo"));
    }

    #[test]
    fn define_and_get() {
        let sut = Environment::new();
        sut.define("foo", 42);
        assert_eq!(Some(42), sut.get("foo"));
    }

    #[test]
    fn define_and_get_in_enclosed_1() {
        let sut = Environment::<u32>::new();
        sut.define("foo", 42);
        let enclosing = sut.enclose();
        enclosing.define("foo", 84);
        assert_eq!(Some(84), enclosing.get("foo"));
        assert_eq!(Some(42), sut.get("foo"));
    }

    #[test]
    fn define_and_get_in_enclosed_2() {
        let sut = Environment::<u32>::new();
        sut.define("foo", 42);
        let enclosing = sut.enclose();
        enclosing.define("bar", 84);
        assert_eq!(Some(84), enclosing.get("bar"));
        assert_eq!(None, sut.get("bar"));
    }

    #[test]
    fn define_and_get_in_enclosed_3() {
        let sut = Environment::<u32>::new();
        sut.define("foo", 42);
        let enclosing = sut.enclose();
        assert_eq!(Some(42), enclosing.get("foo"));
        assert_eq!(Some(42), sut.get("foo"));
    }

    #[test]
    fn define_and_get_in_enclosed_4() {
        let sut = Environment::<u32>::new();
        sut.define("foo", 42);
        let enclosing_1 = sut.enclose();
        enclosing_1.define("foo", 84);
        let enclosing_2 = enclosing_1.enclose();
        enclosing_2.define("foo", 168);
        assert_eq!(Some(42), sut.get("foo"));
        assert_eq!(Some(84), enclosing_1.get("foo"));
        assert_eq!(Some(168), enclosing_2.get("foo"));
    }
}
