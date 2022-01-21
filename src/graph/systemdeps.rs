use crate::graph::{DynSystemFactory, System, SystemFactory};
use smallvec::SmallVec;
use std::any::{type_name, Any, TypeId};
use std::ops::Deref;
use tuple_list::{Tuple, TupleList};

pub(crate) trait SystemDeps: Tuple {
    type SystemDepsList: SystemDepsList<SystemDeps = Self>;

    fn into_system_deps_list(self) -> Self::SystemDepsList;
}

impl<T> SystemDeps for T
where
    T: Tuple,
    T::TupleList: SystemDepsList<SystemDeps = T>,
{
    type SystemDepsList = T::TupleList;

    fn into_system_deps_list(self) -> Self::SystemDepsList {
        self.into_tuple_list()
    }
}

pub(crate) trait SystemDepsList: TupleList + 'static {
    type SystemDeps: SystemDeps<SystemDepsList = Self>;
    type Head: 'static;
    type Tail: SystemDepsList;

    fn into_system_deps(self) -> Self::SystemDeps;

    fn for_each<F>(callable: F)
    where
        F: FnMut(Box<dyn DynSystemFactory>);

    fn build(deps: &mut SmallVec<[Box<dyn Any>; 12]>) -> Option<Self>;
}

impl SystemDepsList for () {
    type SystemDeps = ();
    type Head = ();
    type Tail = ();

    fn into_system_deps(self) -> Self::SystemDeps {}

    fn for_each<F>(_callable: F)
    where
        F: FnMut(Box<dyn DynSystemFactory>),
    {
    }

    fn build(deps: &mut SmallVec<[Box<dyn Any>; 12]>) -> Option<Self> {
        if deps.len() == 0 {
            return Some(());
        }
        None
    }
}

impl<Head, Tail> SystemDepsList for (Head, Tail)
where
    Self: TupleList,
    Head: System,
    Tail: SystemDepsList,
{
    type SystemDeps = Self::Tuple;
    type Head = Head;
    type Tail = Tail;

    fn into_system_deps(self) -> Self::SystemDeps {
        self.into_tuple()
    }

    fn for_each<F>(mut callable: F)
    where
        F: FnMut(Box<dyn DynSystemFactory>),
    {
        callable(Box::new(SystemFactory::<Head>::of()));
        Tail::for_each(callable);
    }

    fn build(deps: &mut SmallVec<[Box<dyn Any>; 12]>) -> Option<Self> {
        deps.pop()
            .and_then(|head| head.downcast::<Head>().ok())
            .map(|head| *head)
            .and_then(|head| Tail::build(deps).map(|tail| (head, tail)))
    }
}
