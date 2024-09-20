use std::{collections::HashMap, rc::Rc};

pub struct Environment<T> {
    enclosing: Option<Box<Rc<Environment<T>>>>,
    values: HashMap<String, T>,
}

impl<T> Environment<T> {
    pub fn new() -> Self {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn enclosing(env: Rc<Environment<T>>) -> Self {
        Self {
            enclosing: Some(Box::new(env)),
            values: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: T) {
        self.values.insert(key, value);
    }

    pub fn get(&self, key: &String) -> Option<&T> {
        match self.values.get(key) {
            Some(value) => Some(value),
            None => match &self.enclosing {
                Some(env) => env.get(key),
                None => None,
            },
        }
    }
}
