// see https://github.com/malobre/erased_set for inspiration and API
// see https://lucumr.pocoo.org/2022/1/7/as-any-hack/ for additional impl details

use downcast_rs::{Downcast, impl_downcast};
use std::collections::HashMap;
use std::fmt::{self, Debug};
use crate::common::typekey::TypeKey;

#[derive(Debug, Default)]
pub struct ErasedSet(HashMap<TypeKey, Box<dyn AnyDebug>>);

impl ErasedSet {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    #[must_use]
    pub fn contains<T: 'static>(&self) -> bool {
        let key = TypeKey::of::<T>();
        self.0.contains_key(&key)
    }

    #[must_use]
    pub fn get<T: Debug + Downcast>(&self) -> Option<&T> {
        let key = TypeKey::of::<T>();
        let result = self.0.get(&key)?;
        Some(&result.downcast_ref::<ErasedMember<T>>()?.0)
    }

    #[must_use]
    pub fn get_mut<T: Debug + Downcast>(&mut self) -> Option<&mut T> {
        let key = TypeKey::of::<T>();
        let result = self.0.get_mut(&key)?;
        Some(&mut result.downcast_mut::<ErasedMember<T>>()?.0)
    }

    pub fn get_or_insert<T: Debug + Downcast>(&mut self, value: T) -> &mut T {
        let key = TypeKey::of::<T>();
        let result = self.0.entry(key).or_insert_with(|| Box::new(ErasedMember(value)));
        &mut result.downcast_mut::<ErasedMember<T>>().unwrap().0
    }

    pub fn get_or_insert_with<T: Debug + Downcast>(&mut self, f: impl FnOnce() -> T) -> &mut T {
        let key = TypeKey::of::<T>();
        let result = self.0.entry(key).or_insert_with(|| Box::new(ErasedMember(f())));
        &mut result.downcast_mut::<ErasedMember<T>>().unwrap().0
    }

    pub fn get_or_insert_default<T: Debug + Downcast + Default>(&mut self) -> &mut T {
        let key = TypeKey::of::<T>();
        let result = self.0.entry(key).or_insert_with(|| Box::new(ErasedMember(T::default())));
        &mut result.downcast_mut::<ErasedMember<T>>().unwrap().0
    }

    pub fn insert<T: Debug + Downcast>(&mut self, value: T) -> Option<T> {
        let key = TypeKey::of::<T>();
        let result = self.0.insert(key, Box::new(ErasedMember(value)))?;
        return Some(result.downcast::<ErasedMember<T>>().unwrap().0);
    }

    pub fn remove<T: Debug + Downcast>(&mut self) -> Option<T> {
        let key = TypeKey::of::<T>();
        let result = self.0.remove(&key)?;
        return Some(result.downcast::<ErasedMember<T>>().unwrap().0);
    }
}

trait AnyDebug : Debug + Downcast {}
impl_downcast!(AnyDebug);

impl<T: Debug + Downcast> AnyDebug for T {}

struct ErasedMember<T>(T);

impl<T: Debug + Downcast> Debug for ErasedMember<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::ErasedSet;

    #[test]
    fn empty_set() {
        let set = ErasedSet::new();
        assert_eq!(set.len(), 0);
        assert!(set.is_empty());
    }

    #[test]
    fn insert_elements() {
        let mut set = ErasedSet::new();
        set.insert(42u8);
        assert_eq!(set.len(), 1);
        assert!(!set.is_empty());
        set.insert(42u16);
        assert_eq!(set.len(), 2);
        assert!(!set.is_empty());
    }

    #[test]
    fn contains() {
        let mut set = ErasedSet::new();
        set.insert(42u8);
        assert!(!set.contains::<u16>());
        set.insert(42u16);
        assert!(set.contains::<u16>());
        set.remove::<u16>();
        assert!(!set.contains::<u16>());
    }

    #[test]
    fn get() {
        let mut set = ErasedSet::new();
        set.insert(42u8);
        assert_eq!(set.get::<u16>(), None);
        set.insert(42u16);
        set.insert(123u16);
        assert_eq!(set.get::<u16>(), Some(&123u16));
    }

    #[test]
    fn get_mut() {
        let mut set = ErasedSet::new();
        set.insert(42u16);
        *set.get_mut::<u16>().unwrap() = 123u16;
        assert_eq!(set.get::<u16>(), Some(&123u16));
    }

    #[test]
    fn or_insert() {
        let mut set = ErasedSet::new();
        set.insert(123u8);
        assert_eq!(set.get_or_insert(0u8), &mut 123u8);
        assert_eq!(set.get_or_insert_default::<u8>(), &mut 123u8);
        assert_eq!(set.get_or_insert_with(|| 0u8), &mut 123u8);
        set.clear();
        assert_eq!(set.get_or_insert(10u8), &mut 10u8);
        set.clear();
        assert_eq!(set.get_or_insert_with(|| 20u8), &mut 20u8);
        set.clear();
        assert_eq!(set.get_or_insert_default::<u8>(), &mut 0u8);
    }

    #[test]
    fn different_types_stored_once() {
        let mut set = ErasedSet::new();
        set.insert("foo");
        set.insert("bar");
        set.insert("quux");
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn zero_sized_types() {
        #[derive(Debug, PartialEq, Eq)]
        struct A;
        #[derive(Debug, PartialEq, Eq)]
        struct B;
        let mut set = ErasedSet::new();
        set.insert(A);
        set.insert(B);
        assert_eq!(set.len(), 2);
        assert_eq!(set.get::<A>(), Some(&A));
    }

    #[test]
    fn clear() {
        let mut set = ErasedSet::new();
        set.insert(42u8);
        set.insert(42u16);
        assert_eq!(set.len(), 2);
        set.clear();
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn debug() {
        let mut set = ErasedSet::new();
        set.insert(42);
        set.insert("test");
        let str = format!("{:?}", set);
        // there are no guarantees about field order, so check both
        assert!(str == "ErasedSet({i32: 42, &str: \"test\"})" ||
                str == "ErasedSet({&str: \"test\", i32: 42})");
    }
}
