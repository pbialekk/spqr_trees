#[derive(Debug, Clone)]
pub struct CircularList {
    pub prev: Vec<usize>,
    pub next: Vec<usize>,
    /// Maps internal index to actual value
    pub vals: Vec<usize>,
    pub size: usize,
}

impl CircularList {
    pub fn new(vals: Vec<usize>) -> Self {
        let n = vals.len();
        if n == 0 {
            return Self {
                prev: vec![],
                next: vec![],
                vals,
                size: 0,
            };
        }
        let mut next = Vec::with_capacity(n);
        let mut prev = Vec::with_capacity(n);
        for i in 0..n {
            next.push((i + 1) % n);
            prev.push((i + n - 1) % n);
        }
        Self {
            prev,
            next,
            vals,
            size: n,
        }
    }

    /// Creates a list with the given values, but with no initial connections (prev/next set to usize::MAX)
    pub fn new_disconnected(vals: Vec<usize>) -> Self {
        let n = vals.len();
        Self {
            prev: vec![usize::MAX; n],
            next: vec![usize::MAX; n],
            vals,
            size: n,
        }
    }

    pub fn remove(&mut self, idx: usize) {
        if self.size == 0 {
            return;
        }
        let p = self.prev[idx];
        let n = self.next[idx];
        self.next[p] = n;
        self.prev[n] = p;
        self.size -= 1;
    }

    pub fn link(&mut self, u_idx: usize, v_idx: usize) {
        self.next[u_idx] = v_idx;
        self.prev[v_idx] = u_idx;
    }
}
