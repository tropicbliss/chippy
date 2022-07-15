use std::ops::{Index, IndexMut};

/// Do not use this
pub struct UncheckedArray<const N: usize, T> {
    array: [T; N],
}

impl<const N: usize, T> UncheckedArray<N, T> {
    /// Pls no
    pub unsafe fn new(array: [T; N]) -> Self {
        Self { array }
    }
}

impl<const N: usize, T> Index<usize> for UncheckedArray<N, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { self.array.get_unchecked(index) }
    }
}

impl<const N: usize, T> IndexMut<usize> for UncheckedArray<N, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { self.array.get_unchecked_mut(index) }
    }
}

/// D:
pub struct UncheckedVec<T> {
    vec: Vec<T>,
}

impl<T> UncheckedVec<T> {
    /// Naurr
    pub unsafe fn new(vec: Vec<T>) -> Self {
        Self { vec }
    }

    /// Ok
    pub fn clear(&mut self) {
        self.vec.clear();
    }
}

impl<T> Index<usize> for UncheckedVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { self.vec.get_unchecked(index) }
    }
}

impl<T> IndexMut<usize> for UncheckedVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { self.vec.get_unchecked_mut(index) }
    }
}
