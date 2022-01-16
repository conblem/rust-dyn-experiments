use std::any::Any;
use std::hash::{Hash, Hasher};

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
