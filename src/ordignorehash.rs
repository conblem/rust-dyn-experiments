use std::hash::{BuildHasher, Hash, Hasher};

struct Container<T, S> {
    inner: Vec<T>,
    hasher_factory: S,
}

impl<T, S> Container<T, S> {
    fn new(hasher_factory: S) -> Self {
        Container {
            inner: Vec::new(),
            hasher_factory,
        }
    }
}

impl<T: Hash, S: BuildHasher> Hash for Container<T, S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut res = 0;
        for elem in &self.inner {
            let mut hasher = self.hasher_factory.build_hasher();
            elem.hash(&mut hasher);
            res ^= hasher.finish();
        }
        state.write_u64(res);
    }
}

impl<T: PartialEq, S> PartialEq for Container<T, S> {
    fn eq(&self, other: &Self) -> bool {
        if self.inner.len() != other.inner.len() {
            return false;
        }
        self.inner.iter().all(|elem| other.inner.contains(elem))
    }
}

impl<T: Eq, S> Eq for Container<T, S> {}
