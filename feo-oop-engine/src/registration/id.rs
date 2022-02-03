//! Identification systems
//! 
//! TODO
//! 
use std::sync::{Arc, Mutex};

#[derive(Debug, PartialEq, Eq, Hash)]
#[derive(Clone)] // For now you can clone an ID but not create two objects with the same IDs 
// Bug: TOFIX returning a cloned ID may lead to two objects with the same ID being created
pub struct ID(usize, IDSystem);

impl PartialOrd for ID {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        assert_eq!(self.1, other.1);
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for ID {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        assert_eq!(self.1, other.1);
        self.0.cmp(&other.0)
    }
}

impl std::fmt::Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ID{
    pub fn get_value(&self) -> usize{ self.0 }
    pub fn get_system(&self) -> IDSystem { self.1.clone() }
}

#[derive(Debug, Default, Clone)]
pub struct IDSystem {
    inner: Arc<Mutex<IDSystemInner>>
}

impl Eq for IDSystem {}

impl PartialEq for IDSystem {
    fn eq(&self, other: &Self) -> bool {
        Arc::as_ptr(&self.inner) == Arc::as_ptr(&other.inner)
    }
}

impl std::hash::Hash for IDSystem {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.inner).hash(state);
    }
}

#[derive(Debug, Default)]
struct IDSystemInner {
    free: Vec<usize>,
    current: usize,
}

impl IDSystem {

    pub fn take(&self) -> ID {
        let inner = self.inner.clone();
        let inner = &mut inner.lock().unwrap();
        match inner.free.pop() {
            Some(id) => ID(id, self.clone()),
            None => ID({inner.current += 1; inner.current}, self.clone())
        }
    }

    pub fn free(&mut self, id: ID) {
        self.inner.clone().lock().unwrap().free.push(id.0)
    }
}