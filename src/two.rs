use std::any::TypeId;
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

    fn map<F>(self, f: F)
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

#[cfg(test)]
mod tests {
    use super::super::Map;
    use super::*;
    use tuple_list::Tuple;

    #[test]
    fn test() {
        //let test = (1, 2, 3).into_tuple_list();
        let types = <<(usize, usize, usize) as Tuple>::TupleList as AsTypeId>::as_type_id();

        println!("{:?}", types);

        let res: ((), (), ()) = types
            .map(|type_id| {
                println!("{:?}", type_id);
                ()
            })
            .into_tuple();
        println!("{:?}", res);
    }
}
