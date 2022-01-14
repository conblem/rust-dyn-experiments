use std::any::{Any, TypeId};
use tuple_list::{Tuple, TupleList};

type BoxAny = Box<dyn Any + Send + Sync>;

trait SuperTuple: Tuple {
    type SuperTupleList: SuperTupleList<SuperTuple = Self>;

    fn into_super_tuple_list(self) -> Self::SuperTupleList;
}

impl<T> SuperTuple for T
where
    T: Tuple,
    T::TupleList: SuperTupleList<SuperTuple = Self>,
{
    type SuperTupleList = Self::TupleList;

    fn into_super_tuple_list(self) -> Self::SuperTupleList {
        self.into_tuple_list()
    }
}

trait SuperTupleList: TupleList + 'static {
    type SuperTuple: SuperTuple<SuperTupleList = Self>;
    type Head: 'static;
    type Tail: SuperTupleList;
    type TypeIds: TypeIds<Self::Head, Self::Tail>;

    fn into_super_tuple(self) -> Self::SuperTuple;
    fn from_parts(head_tail: (Self::Head, Self::Tail)) -> Self;
    fn type_ids() -> Self::TypeIds;
}

impl SuperTupleList for () {
    type SuperTuple = ();
    type Head = ();
    type Tail = ();
    type TypeIds = ();

    fn into_super_tuple(self) -> Self::SuperTuple {
        ()
    }

    fn from_parts(_: (Self::Head, Self::Tail)) -> Self {
        ()
    }

    fn type_ids() -> Self::TypeIds {
        ()
    }
}

impl<H: 'static, T> SuperTupleList for (H, T)
where
    Self: TupleList,
    T: SuperTupleList,
    (TypeId, T::TypeIds): TypeIds<H, T>,
{
    type SuperTuple = Self::Tuple;
    type Head = H;
    type Tail = T;
    type TypeIds = (TypeId, T::TypeIds);

    fn into_super_tuple(self) -> Self::SuperTuple {
        self.into_tuple()
    }

    fn from_parts(head_tail: (Self::Head, Self::Tail)) -> Self {
        head_tail
    }

    fn type_ids() -> Self::TypeIds {
        (TypeId::of::<H>(), T::type_ids())
    }
}

trait TypeIds<H: 'static, T: SuperTupleList>: TupleList {
    fn downcast<F>(self, fun: F) -> Option<(H, T)>
    where
        F: FnMut(TypeId) -> Option<BoxAny>;
}

impl<H: 'static, T: SuperTupleList> TypeIds<H, T> for () {
    fn downcast<F>(self, _fun: F) -> Option<(H, T)>
    where
        F: FnMut(TypeId) -> Option<BoxAny>,
    {
        let mut wrapper = Some(((), ()));

        <dyn Any>::downcast_mut::<Option<(H, T)>>(&mut wrapper)
            .map(Option::take)
            .flatten()
    }
}

impl<H: 'static, T: SuperTupleList, Tail> TypeIds<H, T> for (TypeId, Tail)
where
    Self: TupleList,
    Tail: TypeIds<T::Head, T::Tail>,
{
    fn downcast<F>(self, mut fun: F) -> Option<(H, T)>
    where
        F: FnMut(TypeId) -> Option<BoxAny>,
    {
        if T::TUPLE_LIST_SIZE != Tail::TUPLE_LIST_SIZE {
            return None;
        }

        let (head, tail) = self;

        fun(head)
            .and_then(|head| head.downcast::<H>().ok())
            .map(|head| *head)
            .and_then(move |head| {
                tail.downcast(fun)
                    .map(T::from_parts)
                    .map(|tail| (head, tail))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::super::Map;
    use super::*;

    #[test]
    fn test() {
        let res = hallo(|(num, str, string): (usize, &'static str, String)| {
            format!("{}, {}, {}", num, str, string)
        });
        println!("{}", res.unwrap());
    }

    fn hallo<F, A, R>(fun: F) -> Option<R>
    where
        F: Fn(A) -> R,
        A: SuperTuple,
    {
        let mut map = Map::new();
        map.insert(1 as usize);
        map.insert("Hello");
        map.insert("World".to_string());

        let types = <A::SuperTupleList>::type_ids();
        types
            .downcast(|type_id| map.remove_any(&type_id))
            .map(<A::SuperTupleList>::from_parts)
            .map(SuperTupleList::into_super_tuple)
            .map(|tuple| fun(tuple))
    }

    #[test]
    fn test_is_super_tuple() {
        let tuple = (1 as usize, 2 as usize, 3 as usize);
        is_super_tuple(tuple);
    }

    fn is_super_tuple<T: SuperTuple>(input: T) {
        let list = input.into_super_tuple_list();
        is_super_tuple_list(list);
    }

    fn is_super_tuple_list<T: SuperTupleList>(input: T) {
        let type_ids = T::type_ids().into_tuple();
        type_name_of_val(&type_ids);
        let res = input.into_super_tuple();
        type_name_of_val(&res);
    }

    fn type_name_of_val<T>(_val: &T) {
        println!("{}", std::any::type_name::<T>());
    }
}
