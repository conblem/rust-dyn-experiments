
#[cfg(test)]
mod tests {
    use daggy::{Dag, Walker};
    use daggy::petgraph::graph::DefaultIx;
    use daggy::petgraph::visit::Topo;
    use std::any::TypeId;
    use std::collections::HashMap;

    #[test]
    fn test() {
        let mut map = HashMap::new();
        let mut dag: Dag<(), (), DefaultIx> = Dag::new();
        let index = dag.add_node(());
        map.insert(index, TypeId::of::<String>());

        let mut topo = Topo::new(&dag);
        let index = topo.walk_next(&dag).unwrap();
        let type_id = map.get(&index).unwrap();
        println!("{:?}", type_id);
    }
}