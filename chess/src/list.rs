use core::slice::Iter;

/// An heapless Vec-like struct, with fixed capacity.
#[derive(Clone, Debug)]
pub struct List<T: Clone, const N: usize> {
    len: usize,
    elements: [T; N],
}

impl<T: Clone, const N: usize> List<T, N> {
    /// Constructs a new empty List.
    #[inline]
    pub fn new() -> List<T, N> {
        List {
            len: 0,
            elements: unsafe {core::mem::MaybeUninit::uninit().assume_init()},
        }
    }

    /// Returns the length of the list.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Pushes a new element at the end of the list. 
    /// Make sur list.len() <= N before attempting to push.
    #[inline]
    pub fn push(&mut self, element: T) {
        self.elements[self.len] = element;
        self.len += 1;
    }

    /// Returns the last element in the list, and removes it from the list.
    /// Make sure list.len() > 0 before attempting to pop.
    #[inline]
    pub fn pop(&mut self) -> T {
        self.len -= 1;
        self.elements[self.len].clone()
    }

    /// Clears the list and empties it.
    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Returns an iterator to the elements of the list.
    #[inline]
    pub fn iter(&self) -> Iter<T> {
        self.elements.iter()
    }
}

impl<T: Clone, const N: usize> core::ops::Index<usize> for List<T, N> {
    type Output = T;

    /// Indexes an element of the list in an immutable context.
    #[inline]
    fn index(&self, index: usize) -> &T {
        &self.elements[index]
    }
}

impl<T: Clone, const N: usize> core::ops::IndexMut<usize> for List<T, N> {
    /// Indexes an element of the list in a mutable context.
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self.elements[index]
    }
}