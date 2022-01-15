use std::any::{Any, TypeId};
use std::cell::Cell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

mod supertuple;
mod dynhash;
mod ordignorehash;

type SuperAny = dyn Any + Send + Sync + 'static;
type BoxAny = Box<SuperAny>;

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


struct Map {
    inner: HashMap<TypeId, BoxAny>,
}

impl Map {
    fn new() -> Self {
        Map {
            inner: HashMap::new(),
        }
    }

    fn remove_any(&mut self, key: &TypeId) -> Option<BoxAny> {
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

    fn get_any(&mut self, key: &TypeId) -> Option<&mut BoxAny> {
        self.inner.get_mut(key)
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
