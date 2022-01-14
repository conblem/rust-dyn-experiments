use std::any::{Any, TypeId};
use tuple_list::TupleList;

trait AsTypeId {
    type Result: TupleList;

    fn as_type_id() -> Self::Result;
}

impl AsTypeId for () {
    type Result = ();

    fn as_type_id() -> Self::Result {
        ()
    }
}

impl<Head: 'static, Tail> AsTypeId for (Head, Tail)
where
    Tail: AsTypeId,
    (TypeId, Tail::Result): TupleList,
{
    type Result = (TypeId, Tail::Result);

    fn as_type_id() -> Self::Result {
        (TypeId::of::<Head>(), Tail::as_type_id())
    }
}

trait Mapper<T, R> {
    type Result: TupleList;
    fn map<F>(self, f: F) -> Self::Result
    where
        F: FnMut(T) -> R;
}

impl<T, R> Mapper<T, R> for () {
    type Result = ();

    fn map<F>(self, _: F)
    where
        F: FnMut(T) -> R,
    {
        ()
    }
}

impl<R, Head, Tail> Mapper<Head, R> for (Head, Tail)
where
    Tail: Mapper<Head, R>,
    (R, Tail::Result): TupleList,
{
    type Result = (R, Tail::Result);

    fn map<F>(self, mut f: F) -> Self::Result
    where
        F: FnMut(Head) -> R,
    {
        let (head, tail) = self;
        let head = f(head);
        (head, tail.map(f))
    }
}

trait Split: TupleList + 'static {
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

type BoxAny = Box<dyn Any + Send + Sync>;

trait Downcast<T: Split>: TupleList {
    fn downcast(self) -> Option<T>;
}

impl<T: Split> Downcast<T> for () {
    fn downcast(self) -> Option<T> {
        let mut wrapper = Some(self);

        <dyn Any>::downcast_mut::<Option<T>>(&mut wrapper)
            .map(Option::take)
            .flatten()
    }
}

impl<T: Split, Tail> Downcast<T> for (BoxAny, Tail)
where
    Self: TupleList,
    Tail: Downcast<T::Tail>,
{
    fn downcast(self) -> Option<T> {
        if T::TUPLE_LIST_SIZE != Self::TUPLE_LIST_SIZE {
            return None;
        }

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
    }
}

#[cfg(test)]
mod tests {
    use super::super::Map;
    use super::*;
    use tuple_list::Tuple;

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
        A: Tuple,
        A::TupleList: AsTypeId + Split,
        <A::TupleList as AsTypeId>::Result: Mapper<TypeId, BoxAny>,
        <<A::TupleList as AsTypeId>::Result as Mapper<TypeId, BoxAny>>::Result:
            Downcast<A::TupleList>,
    {
        let mut map = Map::new();
        map.insert(1 as usize);
        map.insert("Hello");
        map.insert("World".to_string());

        let types = <A::TupleList as AsTypeId>::as_type_id();

        let res = types.map(|type_id| map.remove_any(&type_id).unwrap());

        res.downcast()
            .map(TupleList::into_tuple)
            .map(fun)
    }
}
