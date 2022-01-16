use std::any::{Any, TypeId};
use tuple_list::{Tuple, TupleList};

use super::{BoxAny, SuperAny};

trait SuperTuple<'a>: Tuple {
    type SuperTupleList: SuperTupleList<'a, SuperTuple = Self>;

    fn into_super_tuple_list(self) -> Self::SuperTupleList;
}

impl<'a, T> SuperTuple<'a> for T
where
    T: Tuple,
    T::TupleList: SuperTupleList<'a, SuperTuple = Self>,
{
    type SuperTupleList = Self::TupleList;

    fn into_super_tuple_list(self) -> Self::SuperTupleList {
        self.into_tuple_list()
    }
}

trait SuperTupleList<'a>: TupleList {
    type SuperTuple: SuperTuple<'a, SuperTupleList = Self>;
    type Head: 'static;
    type Tail: SuperTupleList<'a>;
    type TypeIds: TypeIds<'a, Self::Head, Self::Tail>;

    fn into_super_tuple(self) -> Self::SuperTuple;
    fn from_parts(head_tail: (&'a mut Self::Head, Self::Tail)) -> Self;
    fn from_head(head: &Self::Head) -> Option<Self>;
    fn type_ids() -> Self::TypeIds;
}

impl<'a> SuperTupleList<'a> for () {
    type SuperTuple = ();
    type Head = ();
    type Tail = ();
    type TypeIds = ();

    fn into_super_tuple(self) -> Self::SuperTuple {
        ()
    }

    fn from_parts(_: (&'a mut Self::Head, Self::Tail)) -> Self {
        ()
    }

    fn from_head(head: &Self::Head) -> Option<Self> {
        Some(())
    }

    fn type_ids() -> Self::TypeIds {
        ()
    }
}

impl<'a, H: 'static, T> SuperTupleList<'a> for (&'a mut H, T)
where
    Self: TupleList,
    T: SuperTupleList<'a>,
    (TypeId, T::TypeIds): TypeIds<'a, H, T>,
{
    type SuperTuple = Self::Tuple;
    type Head = H;
    type Tail = T;
    type TypeIds = (TypeId, T::TypeIds);

    fn into_super_tuple(self) -> Self::SuperTuple {
        self.into_tuple()
    }

    fn from_parts(head_tail: (&'a mut Self::Head, Self::Tail)) -> Self {
        head_tail
    }

    fn from_head(head: &Self::Head) -> Option<Self> {
        None
    }

    fn type_ids() -> Self::TypeIds {
        (TypeId::of::<H>(), T::type_ids())
    }
}

// Trait to Support Downcasting of BoxAny TupleList to concrete TupleList
// To avoid overflowing the Rust Compiler we pass the concrete TupleList split into Head and Tail
trait TypeIds<'a, H: 'static, T: SuperTupleList<'a>>: TupleList {
    fn downcast<F>(self, fun: F) -> Option<(&'a mut H, T)>
    where
        F: FnMut(TypeId) -> Option<&'a mut SuperAny>;
}

impl<'a, H: 'static, T: SuperTupleList<'a>> TypeIds<'a, H, T> for () {
    fn downcast<F>(self, _fun: F) -> Option<(&'a mut H, T)>
    where
        F: FnMut(TypeId) -> Option<&'a mut SuperAny>,
    {
        None
    }
}

impl<'a, H: 'static, T: SuperTupleList<'a>, Tail> TypeIds<'a, H, T> for (TypeId, Tail)
where
    Self: TupleList,
    Tail: TypeIds<'a, T::Head, T::Tail>,
{
    fn downcast<F>(self, mut fun: F) -> Option<(&'a mut H, T)>
    where
        F: FnMut(TypeId) -> Option<&'a mut SuperAny>,
    {
        // Early Return if TupleList have different Sizes
        if T::TUPLE_LIST_SIZE != Tail::TUPLE_LIST_SIZE {
            return None;
        }

        let (head, tail) = self;
        // Get Boxed Head using TypeId
        let head = fun(head)
            // Downcast Boxed Head
            .and_then(|head| head.downcast_mut::<H>());

        let head = match head {
            Some(head) => head,
            None => return None,
        };

        // Stop condition is Met if Tail is ()
        // Specialization if we are at the end of the TupleList
        let head_of_tail = &mut ();
        let head_of_tail = <dyn Any>::downcast_mut::<T::Head>(head_of_tail);
        if let Some(head_of_tail) = head_of_tail {
            return T::from_head(head_of_tail).map(|tail| (head, tail));
        }

        // Recursively Downcast Tail
        tail.downcast(fun)
            // Reassemble Tail from (T::Head, T::Tail)
            .map(T::from_parts)
            .map(|tail| (head, tail))
    }
}

#[cfg(test)]
mod tests {
    use super::super::Map;
    use super::*;

    use std::collections::HashMap;
    use std::ops::DerefMut;

    #[test]
    fn test() {
        let mut map = HashMap::new();
        map.insert(TypeId::of::<usize>(), convert(1 as usize));
        map.insert(TypeId::of::<String>(), convert("Hallo".to_string()));
        let mut map: HashMap<_, _> = map
            .iter_mut()
            .map(|(key, val)| (*key, val.deref_mut()))
            .collect();

        let res = hallo(&mut map, |(num, string): (&mut usize, &mut String)| {
            format!("{}, {}", num, string)
        });
        println!("{}", res.unwrap());
    }

    fn convert<T: Send + Sync + 'static>(input: T) -> BoxAny {
        Box::new(input)
    }

    fn hallo<'a, F, A, R>(map: &'a mut HashMap<TypeId, &mut SuperAny>, fun: F) -> Option<R>
    where
        F: Fn(A) -> R,
        A: SuperTuple<'a>,
    {
        let string_type = TypeId::of::<String>();
        let types = <A::SuperTupleList>::type_ids();

        types
            .downcast(|type_id| map.remove(&type_id))
            // Reassemble concrete TupleList from (A::TupleList::Head, A::TupleList::Tail)
            .map(<A::SuperTupleList>::from_parts)
            // Turn concrete TupleList into concrete Tuple
            .map(SuperTupleList::into_super_tuple)
            // Pass concrete Tuple to fun
            .map(fun)
    }

    /*#[test]
    fn test_is_super_tuple() {
        let tuple = (1 as usize, 2 as usize, 3 as usize);
        is_super_tuple(tuple);
    }

    fn is_super_tuple<'a, T: SuperTuple<'a>>(input: T) {
        let list = input.into_super_tuple_list();
        is_super_tuple_list(list);
    }

    fn is_super_tuple_list<'a, T: SuperTupleList<'a>>(input: T) {
        let type_ids = T::type_ids().into_tuple();
        type_name_of_val(&type_ids);
        let res = input.into_super_tuple();
        type_name_of_val(&res);
    }

    // is currently experimental #66359 so we just implement it ourselves
    fn type_name_of_val<T>(_val: &T) {
        println!("{}", std::any::type_name::<T>());
    }*/
}
