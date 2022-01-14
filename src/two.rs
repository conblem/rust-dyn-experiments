use std::any::{Any, TypeId};
use tuple_list::{Tuple, TupleList};

struct Res<'a, T> {
    t: &'a T,
    ty: TypeId,
}

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

// Trait to Support Downcasting of BoxAny TupleList to concrete TupleList
// To avoid overflowing the Rust Compiler we pass the concrete TupleList split into Head and Tail
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
        // At this point (H, T) should be ((), ())
        // If this is the case we can cast ((), ()) to (H, T)
        // Otherwise this method was called incorrectly so we return None
        let mut wrapper = Some(((), ()));

        // To get an owned value out of a downcast_mut we use Option<>
        // otherwise we would have to Box it
        <dyn Any>::downcast_mut::<Option<(H, T)>>(&mut wrapper)
            .map(Option::take)
            // wrapper is always Some
            .unwrap()
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
        // Early Return if TupleList have different Sizes
        if T::TUPLE_LIST_SIZE != Tail::TUPLE_LIST_SIZE {
            return None;
        }

        let (head, tail) = self;
        // Get Boxed Head using TypeId
        fun(head)
            // Downcast Boxed Head
            .and_then(|head| head.downcast::<H>().ok())
            // Unbox Concrete Head
            .map(|head| *head)
            .and_then(|head| {
                // Recursively Downcast Tail
                tail.downcast(fun)
                    // Reassemble Tail from (T::Head, T::Tail)
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
            // Reassemble concrete TupleList from (A::TupleList::Head, A::TupleList::Tail)
            .map(<A::SuperTupleList>::from_parts)
            // Turn concrete TupleList into concrete Tuple
            .map(SuperTupleList::into_super_tuple)
            // Pass concrete Tuple to fun
            .map(fun)
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

    // is currently experimental #66359 so we just implement it ourselves
    fn type_name_of_val<T>(_val: &T) {
        println!("{}", std::any::type_name::<T>());
    }
}
