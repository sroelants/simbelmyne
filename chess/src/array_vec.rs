use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
pub struct ArrayVec<T, const SIZE: usize> {
    moves: [T; SIZE],
    len: usize,
}

impl<T, const SIZE: usize> ArrayVec<T, SIZE> where T: Copy + Clone + Default {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, mv: T) {
        self.moves[self.len] = mv;
        self.len += 1;
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl<T, const SIZE: usize> Default for ArrayVec<T, SIZE> where T: Copy + Clone + Default {
    fn default() -> Self {
        Self {
            moves: [T::default(); SIZE],
            len: 0,
        }
    }
}

impl<T, const SIZE: usize> IntoIterator for ArrayVec<T, SIZE> where T: Copy + Clone {
    type Item = T;

    type IntoIter = IntoIter<T, SIZE>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            inner: self,
            idx: 0
        }
    }
}

impl<T, const SIZE: usize> Deref for ArrayVec<T, SIZE> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.moves
    }
}

impl<T, const SIZE: usize> DerefMut for ArrayVec<T, SIZE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.moves
    }
}

pub struct IntoIter<T, const SIZE: usize> {
    inner: ArrayVec<T, SIZE>,
    idx: usize,
}

impl<T, const SIZE: usize> Iterator for IntoIter<T, SIZE> where T: Copy + Clone {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.inner.len {
            let mv = self.inner[self.idx];
            self.idx += 1;
            Some(mv)
        } else {
            None
        }
    }
}
