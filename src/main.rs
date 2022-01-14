use std::any::{Any, TypeId};
use std::cell::Cell;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash, Hasher};
use tuple_list::Tuple;
use std::ops::{Deref, DerefMut};

mod two;

fn main() {
    let mut map: HashMap<TypeId, Box<dyn Any>> = HashMap::new();
    map.insert(TypeId::of::<usize>(), Box::new(1 as usize));

    let mut ref_map = HashMap::new();
    for (key, val) in map.iter_mut() {
        ref_map.insert(*key, Value::new(val.deref_mut()));
    }

    {
        let mut test = ref_map.get(&TypeId::of::<usize>()).and_then(Value::take).unwrap();
        let test: &mut usize = test.downcast_mut().unwrap();
        *test += 1;
    }
    {
        let mut test = ref_map.get(&TypeId::of::<usize>()).and_then(Value::take).unwrap();
        let test: &mut usize = test.downcast_mut().unwrap();
        println!("{}", test);
    }
}

struct Value<T> {
    inner: Cell<Option<T>>,
}

impl<T> Value<T> {
    fn new(input: T) -> Self {
        Value {
            inner: Cell::new(Some(input)),
        }
    }
    fn take(&self) -> Option<Guard<T>> {
        self.inner.take().map(|inner| Guard {
            owner: &self.inner,
            inner: Some(inner),
        })
    }
}

struct Guard<'a, T: 'a> {
    owner: &'a Cell<Option<T>>,
    inner: Option<T>,
}

impl <'a, T: 'a> Deref for Guard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

impl <'a, T: 'a> DerefMut for Guard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap()
    }
}


impl <'a, T: 'a> Drop for Guard<'a, T> {
    fn drop(&mut self) {
        self.owner.set(self.inner.take())
    }
}

struct Container<T, S> {
    inner: Vec<T>,
    hasher_factory: S,
}

impl<T, S> Container<T, S> {
    fn new(hasher_factory: S) -> Self {
        Container {
            inner: Vec::new(),
            hasher_factory,
        }
    }
}

impl<T: Hash, S: BuildHasher> Hash for Container<T, S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut res = 0;
        for elem in &self.inner {
            let mut hasher = self.hasher_factory.build_hasher();
            elem.hash(&mut hasher);
            res ^= hasher.finish();
        }
        state.write_u64(res);
    }
}

impl<T: PartialEq, S> PartialEq for Container<T, S> {
    fn eq(&self, other: &Self) -> bool {
        if self.inner.len() != other.inner.len() {
            return false;
        }
        self.inner.iter().all(|elem| other.inner.contains(elem))
    }
}

impl<T: Eq, S> Eq for Container<T, S> {}

trait DynHash: DynEq {
    fn clone_box(&self) -> Box<dyn DynHash>;
    fn as_box_any(self: Box<Self>) -> Box<dyn Any>;
    fn as_dyn_eq(&self) -> &dyn DynEq;
    fn hash(&self, state: &mut dyn Hasher);
}

trait DynEq: Any {
    fn as_any(&self) -> &dyn Any;
    fn eq(&self, other: &dyn DynEq) -> bool;
}

impl<T> DynEq for T
where
    T: Eq + Any,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq(&self, other: &dyn DynEq) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<T>() {
            return self == other;
        }
        false
    }
}
impl<T> DynHash for T
where
    T: Hash + Clone + Eq + Any,
{
    fn clone_box(&self) -> Box<dyn DynHash> {
        Box::new(self.clone())
    }

    fn as_box_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn as_dyn_eq(&self) -> &dyn DynEq {
        self
    }

    fn hash(&self, mut state: &mut dyn Hasher) {
        Hash::hash(self, &mut state);
    }
}

impl Hash for dyn DynHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        DynHash::hash(self, state)
    }
}

impl PartialEq for dyn DynHash {
    fn eq(&self, other: &Self) -> bool {
        DynEq::eq(self, other.as_dyn_eq())
    }
}

impl Eq for dyn DynHash {}

impl Clone for Box<dyn DynHash> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

trait System<A: Tuple> {
    type Res;
}

impl<T, A: Tuple> System<A> for T
where
    T: FnMut(A),
{
    type Res = T::Output;
}

struct Map {
    inner: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Map {
    fn new() -> Self {
        Map {
            inner: HashMap::new(),
        }
    }

    fn remove_any(&mut self, key: &TypeId) -> Option<Box<dyn Any + Send + Sync>> {
        self.inner.remove(key)
    }

    fn remove<T: 'static>(&mut self) -> Option<T> {
        let any = match self.inner.remove(&TypeId::of::<T>()) {
            None => return None,
            Some(any) => any,
        };
        let res = *any.downcast().unwrap();
        Some(res)
    }

    fn get<T: 'static>(&mut self) -> Option<&mut T> {
        let any = match self.inner.get_mut(&TypeId::of::<T>()) {
            None => return None,
            Some(any) => any,
        };
        let res = any.downcast_mut().unwrap();
        Some(res)
    }
    fn insert<T: Send + Sync + 'static>(&mut self, elem: T) -> Option<T> {
        let res = self.inner.insert(TypeId::of::<T>(), Box::new(elem));
        let any = match res {
            None => return None,
            Some(any) => any,
        };
        let res = *any.downcast().unwrap();
        Some(res)
    }
}
