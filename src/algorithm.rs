use std::collections::HashMap;
use std::hash::Hash;

pub struct UnionFind<T> {
    parent: HashMap<T, T>,
}

impl<T: Hash + Eq + Clone> UnionFind<T> {
    pub fn new() -> Self {
        Self {
            parent: HashMap::new(),
        }
    }

    pub fn find(&mut self, e: &T) -> T {
        match self.parent.get(e).cloned() {
            None => e.clone(),
            Some(p) => {
                let head = self.find(&p);
                self.parent.insert(e.clone(), head.clone());
                head
            }
        }
    }

    /// a -> b
    pub fn union(&mut self, a: &T, b: &T) -> T {
        let a = self.find(a);
        let b = self.find(b);
        if a != b {
            self.parent.insert(a, b.clone());
        }
        b
    }

    pub fn eq(&mut self, a: &T, b: &T) -> bool {
        let a = self.find(a);
        let b = self.find(b);
        a == b
    }
}

pub struct BiHashMap<L, R> {
    lr: HashMap<L, R>,
    rl: HashMap<R, L>,
}

impl<L: Hash + Eq + Clone, R: Hash + Eq + Clone> BiHashMap<L, R> {
    pub fn new() -> Self {
        Self {
            lr: HashMap::new(),
            rl: HashMap::new(),
        }
    }

    pub fn bind(&mut self, l: L, r: R) {
        self.lr.insert(l.clone(), r.clone());
        self.rl.insert(r, l);
    }

    pub fn get(&self, l: &L) -> Option<&R> {
        self.lr.get(l)
    }
    pub fn find(&self, r: &R) -> Option<&L> {
        self.rl.get(r)
    }

    pub fn mv(&mut self, l: &L, to: R) -> Option<R> {
        let r = self.lr.insert(l.clone(), to.clone());
        self.rl.insert(to, l.clone());

        r.inspect(|r| {
            self.rl.remove(r);
        })
    }

    pub fn rm(&mut self, l: &L) -> Option<R> {
        self.lr.remove(l).inspect(|r| {
            self.rl.remove(r);
        })
    }
}
