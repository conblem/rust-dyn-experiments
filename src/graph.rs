use std::any::TypeId;
use tuple_list::{Tuple, TupleList};

trait SystemDeps: Tuple {
    type SystemDepsList: SystemDepsList;

    fn into_system_deps_list(self) -> Self::SystemDepsList;
}

impl<T> SystemDeps for T
where
    T: Tuple,
    T::TupleList: SystemDepsList,
{
    type SystemDepsList = T::TupleList;

    fn into_system_deps_list(self) -> Self::SystemDepsList {
        self.into_tuple_list()
    }
}

trait SystemDepsList: TupleList + 'static {
    fn for_each<F>(callable: F)
    where
        F: FnMut(TypeId);
}

impl SystemDepsList for () {
    fn for_each<F>(_: F)
    where
        F: FnMut(TypeId),
    {
    }
}

impl<Head, Tail> SystemDepsList for (Head, Tail)
where
    Self: TupleList + 'static,
    Tail: SystemDepsList,
{
    fn for_each<F>(mut callable: F)
    where
        F: FnMut(TypeId),
    {
        callable(TypeId::of::<Head>());
        Tail::for_each(callable);
    }
}

trait System: 'static {
    type Deps: SystemDeps;
}

#[cfg(test)]
mod tests {
    use petgraph::algo::toposort;
    use petgraph::graph::NodeIndex;
    use petgraph::Graph;
    use std::any::TypeId;
    use std::collections::HashMap;

    use super::*;

    struct BookController;

    impl System for BookController {
        type Deps = (BookFacade,);
    }

    struct BookFacade;

    impl System for BookFacade {
        type Deps = ();
    }

    #[test]
    fn test() {
        let mut map = HashMap::new();
        let mut graph = Graph::<(), ()>::new();

        insert_nodes::<BookController>(&mut map, &mut graph);
        insert_nodes::<BookFacade>(&mut map, &mut graph);
        insert_edges::<BookController>(&map, &mut graph);
        insert_edges::<BookFacade>(&map, &mut graph);

        let res = toposort(&graph, None);
        res.unwrap();
    }

    fn insert_nodes<S: System>(map: &mut HashMap<TypeId, NodeIndex>, graph: &mut Graph<(), ()>) {
        let index = graph.add_node(());
        map.insert(TypeId::of::<S>(), index);
    }

    fn insert_edges<S: System>(map: &HashMap<TypeId, NodeIndex>, graph: &mut Graph<(), ()>) {
        let system_index = map.get(&TypeId::of::<S>()).unwrap();
        <S::Deps as SystemDeps>::SystemDepsList::for_each(|type_id| {
            let dep_index = map.get(&type_id).unwrap();
            graph.add_edge(*dep_index, *system_index, ());
        });
    }
}
