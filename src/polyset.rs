#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Polyset<T> {
    elems: Vec<(T, i64)>,
}

impl<T> Polyset<T> {
    pub fn new() -> Self {
        Self { elems: Vec::new() }
    }

    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }

    pub fn keys(&self) -> impl Iterator<Item = &T> {
        self.iter().map(|(x, _)| x)
    }
}

impl<T: Ord> Polyset<T> {
    pub fn from_vec(data: Vec<T>) -> Self {
        data.into_iter().collect()
    }

    pub fn union(self, that: Self) -> Self {
        let mut result = Vec::new();
        result.extend(self.elems.into_iter());
        result.extend(that.elems.into_iter());
        result.into_iter().collect()
    }

    pub fn join(self, that: Self) -> Self {
        let mut a = self.into_iter();
        let mut b = that.into_iter();
        let (mut x, mut m) = match a.next() {
            None => return Self::new(),
            Some(x) => x,
        };
        let (mut y, mut n) = match b.next() {
            None => return Self::new(),
            Some(y) => y,
        };

        let mut elems = Vec::new();
        loop {
            if x == y {
                elems.push((x, m * n));
                (x, m) = match a.next() {
                    None => return Polyset { elems },
                    Some(x) => x,
                };
                (y, n) = match b.next() {
                    None => return Polyset { elems },
                    Some(y) => y,
                };
            } else if x < y {
                (x, m) = match a.next() {
                    None => return Polyset { elems },
                    Some(x) => x,
                };
            } else {
                (y, n) = match b.next() {
                    None => return Polyset { elems },
                    Some(y) => y,
                };
            }
        }
    }
}

impl<T> IntoIterator for Polyset<T> {
    type Item = (T, i64);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.elems.into_iter()
    }
}

impl<'a, T: 'a> IntoIterator for &'a Polyset<T> {
    type Item = &'a (T, i64);
    type IntoIter = std::slice::Iter<'a, (T, i64)>;

    fn into_iter(self) -> Self::IntoIter {
        self.elems.iter()
    }
}

impl<T: Ord> FromIterator<(T, i64)> for Polyset<T> {
    fn from_iter<I: IntoIterator<Item = (T, i64)>>(iter: I) -> Self {
        let mut data: Vec<_> = iter.into_iter().collect();
        data.sort();
        let mut elems = Vec::new();
        if !data.is_empty() {
            let (mut pivot, mut multiplicity) = data.remove(0);
            for (item, n) in data {
                if item == pivot {
                    multiplicity += n;
                } else {
                    elems.push((pivot, multiplicity));
                    pivot = item;
                    multiplicity = n;
                }
            }
            elems.push((pivot, multiplicity));
        }
        Self { elems }
    }
}

impl<T: Ord> FromIterator<T> for Polyset<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_iter(iter.into_iter().map(|v| (v, 1)))
    }
}
