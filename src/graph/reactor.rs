use crate::graph::{DynSystemFactory, SystemFactory};
use petgraph::algo::toposort;
use petgraph::graph::NodeIndex;
use petgraph::Graph;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::ops::Deref;

use super::systemdeps::{SystemDeps, SystemDepsList};
use super::System;

pub(crate) struct ReactorBuilder {
    map: HashMap<Box<dyn DynSystemFactory>, (NodeIndex, SmallVec<[Box<dyn DynSystemFactory>; 12]>)>,
    graph: Graph<(), ()>,
}

impl ReactorBuilder {
    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
            graph: Graph::new(),
        }
    }

    pub(crate) fn add_system<S: System>(&mut self) {
        let index = self.graph.add_node(());

        let mut deps = SmallVec::new();
        <S::Deps as SystemDeps>::SystemDepsList::for_each(|dep| deps.push(dep));

        self.map
            .insert(Box::new(SystemFactory::<S>::of()), (index, deps));
    }

    pub(crate) fn build(self) {
        let Self { mut graph, map } = self;

        for (system_index, deps) in map.values() {
            for dep in deps {
                let (dep_index, _) = map.get(dep).unwrap();
                graph.add_edge(*dep_index, *system_index, ());
            }
        }
        let indexes: Vec<NodeIndex> = toposort(&graph, None).unwrap();

        let map: HashMap<_, _> = map
            .into_iter()
            .map(|(factory, (index, deps))| (index, (factory, deps)))
            .collect();

        let mut empty_vec = SmallVec::new();
        let mut res = HashMap::new();
        for system_index in &indexes {
            let (factory, deps) = map.get(system_index).unwrap();
            let instance = if !factory.has_deps() {
                factory.build(&mut empty_vec).unwrap()
            } else {
                let mut test = SmallVec::new();
                for dep in deps {
                    let type_id = DynSystemFactory::type_id(dep.deref());
                    let dep = res.remove(&type_id).unwrap();
                    test.push(dep);
                }
                factory.build(&mut test).unwrap()
            };
            let type_id = DynSystemFactory::type_id(factory.deref());
            res.insert(type_id, instance);
        }
    }
}
