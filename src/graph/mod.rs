use smallvec::SmallVec;
use std::any::{Any, TypeId};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use tuple_list::{Tuple, TupleList};

use super::dynhash::DynHash;
use crate::graph::systemdeps::SystemDepsList;
use systemdeps::SystemDeps;

mod reactor;
mod systemdeps;

pub(crate) trait System: 'static {
    type Deps: SystemDeps;

    fn build(deps: Self::Deps) -> Self;
}

pub(crate) trait DynSystemFactory: DynHash {
    fn as_dyn_hash(&self) -> &dyn DynHash;

    fn has_deps(&self) -> bool;
    fn build(&self, deps: &mut SmallVec<[Box<dyn Any>; 12]>) -> Option<Box<dyn Any>>;
    fn type_id(&self) -> TypeId;
}

impl Hash for dyn DynSystemFactory {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // we use the explicit form
        Hash::hash(self.as_dyn_hash(), state)
    }
}

impl Eq for dyn DynSystemFactory {}

impl PartialEq for dyn DynSystemFactory {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(self.as_dyn_hash(), other.as_dyn_hash())
    }
}

struct SystemFactory<S> {
    type_id: TypeId,
    _phantom: PhantomData<S>,
}

impl<S> Hash for SystemFactory<S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.type_id, state)
    }
}

impl<S> PartialEq for SystemFactory<S> {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

impl<S> Eq for SystemFactory<S> {}

impl<S: System> SystemFactory<S> {
    pub(crate) fn of() -> Self {
        Self {
            type_id: TypeId::of::<S>(),
            _phantom: PhantomData,
        }
    }
}

impl<S: System> DynSystemFactory for SystemFactory<S> {
    fn as_dyn_hash(&self) -> &dyn DynHash {
        self
    }

    fn has_deps(&self) -> bool {
        <S::Deps as Tuple>::TupleList::TUPLE_LIST_SIZE != 0
    }

    fn build(&self, deps: &mut SmallVec<[Box<dyn Any>; 12]>) -> Option<Box<dyn Any>> {
        <S::Deps as SystemDeps>::SystemDepsList::build(deps)
            .map(SystemDepsList::into_system_deps)
            .map(|deps| S::build(deps))
            .map(|system| Box::new(system) as Box<dyn Any>)
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<S>()
    }
}

#[cfg(test)]
mod tests {
    use super::reactor::ReactorBuilder;
    use super::*;
    use std::fmt::rt::v1::Count::Param;

    struct BookController {
        facade: BookFacade,
    }

    impl System for BookController {
        type Deps = (BookFacade,);

        fn build(deps: Self::Deps) -> Self {
            let (facade,) = deps;
            Self { facade }
        }
    }

    struct BookFacade;

    impl System for BookFacade {
        type Deps = ();

        fn build(_: Self::Deps) -> Self {
            Self
        }
    }

    #[test]
    fn test() {
        let mut reactor_builder = ReactorBuilder::new();
        reactor_builder.add_system::<BookController>();
        reactor_builder.add_system::<BookFacade>();

        reactor_builder.build();
    }
}
