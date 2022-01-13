use itertools::traits::HomogeneousTuple;
use std::any::{Any, TypeId};
use tuple_list::{Tuple, TupleCons, TupleList};

trait ToTypeId {
    fn to_type_id(&self) -> Vec<TypeId>;
}

impl ToTypeId for () {
    fn to_type_id(&self) -> Vec<TypeId> {
        Vec::new()
    }
}

impl<Head, Tail> ToTypeId for (Head, Tail)
where
    Head: 'static,
    Tail: ToTypeId + TupleList,
{
    fn to_type_id(&self) -> Vec<TypeId> {
        let (head, tail) = self;
        let mut res = tail.to_type_id();
        res.push(TypeId::of::<Head>());
        res
    }
}

type BoxAny = Box<dyn Any + Send + Sync>;

trait SuperTuple: Tuple + HomogeneousTuple<Item = BoxAny> {}

impl<T> SuperTuple for T where T: Tuple + HomogeneousTuple<Item = BoxAny> {}

trait AsBoxAnyType {
    type Result: TupleList;
    type Collect: SuperTuple;
}

impl AsBoxAnyType for () {
    type Result = ();
    type Collect = (BoxAny,);
}

impl<Head, Tail> AsBoxAnyType for (Head, Tail)
where
    Tail: AsBoxAnyType + TupleList,
    (BoxAny, Tail::Result): TupleList,
    <(BoxAny, Tail::Result) as TupleList>::Tuple: SuperTuple,
{
    type Result = (BoxAny, Tail::Result);
    type Collect = <(BoxAny, Tail::Result) as TupleList>::Tuple;
}

trait Split: TupleList {
    type Head: 'static;
    type Tail: Split;

    fn from_parts(head: Self::Head, tail: Self::Tail) -> Self;
}

impl Split for () {
    type Head = ();
    type Tail = ();

    fn from_parts(_: Self::Head, _: Self::Tail) -> Self {
        ()
    }
}

impl<H: 'static, T> Split for (H, T)
where
    (H, T): TupleList,
    T: Split,
{
    type Head = H;
    type Tail = T;

    fn from_parts(head: Self::Head, tail: Self::Tail) -> Self {
        (head, tail)
    }
}

trait Downcast<T: Split> {
    fn downcast(self) -> Option<T>;
}

impl<T: Split> Downcast<T> for () {
    fn downcast(self) -> Option<T> {
        None
    }
}

impl<T: Split, Tail> Downcast<T> for (BoxAny, Tail)
where
    Tail: Downcast<T::Tail>,
{
    fn downcast(self) -> Option<T> {
        let (head, tail) = self;
        let head = match head.downcast::<T::Head>() {
            Ok(head) => *head,
            Err(_) => return None,
        };
        let tail = match tail.downcast() {
            Some(tail) => tail,
            None => return None,
        };
        Some(T::from_parts(head, tail))
        //Some((head, tail))
    }
}

#[cfg(test)]
mod tests {
    use super::super::Map;
    use super::*;
    use itertools::Itertools;
    use tuple_list::Tuple;

    #[test]
    fn test() {
        let test = (1, 2, 3);
        hallo(test);
    }

    fn hallo<T: Tuple>(tuple: T)
    where
        T::TupleList: ToTypeId + AsBoxAnyType + Split,
        <<T::TupleList as AsBoxAnyType>::Collect as Tuple>::TupleList: Downcast<T::TupleList>,
    {
        let mut map = Map::new();

        let tuple = tuple.into_tuple_list();
        let res = tuple
            .to_type_id()
            .into_iter()
            .map(|type_id| map.remove_any(&type_id).unwrap());

        let anies = res
            .collect_tuple::<<T::TupleList as AsBoxAnyType>::Collect>()
            .unwrap()
            .into_tuple_list();

        Downcast::<T::TupleList>::downcast(anies).unwrap();
    }
}
