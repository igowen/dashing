use std::mem::MaybeUninit;

/// Simple array-backed variable-length vector. The size of the backing array (`N`) is the maximum
/// capacity of the vector.
///
/// **Panics** if push() is called when there is no remaining capacity.
///
/// This type uses `MaybeUninit` and `unsafe`; the key invariant that is maintained is that
/// the contents of `storage[..len]` are initialized memory.
pub struct InlineVec<T, const N: usize> {
    storage: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> InlineVec<T, N> {
    /// Construct a new, empty vector.
    pub fn new() -> Self {
        Self {
            storage: [(); N].map(|_| MaybeUninit::uninit()),
            len: 0,
        }
    }

    pub fn try_push(&mut self, item: T) -> Result<(), CapacityExceededError> {
        if self.len >= N {
            Err(CapacityExceededError)
        } else {
            self.storage[self.len].write(item);
            self.len += 1;
            Ok(())
        }
    }

    /// Pushes a value onto the end of the vector.
    ///
    /// **Panics** if this would exceed the vector's capacity (`N`).
    pub fn push(&mut self, item: T) {
        if self.try_push(item).is_err() {
            panic!("InlineVec exceeded capacity ({})", N);
        }
    }

    /// Removes the last element from the vector and returns it, or `None` if the vector is empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;

        // SAFETY: We've just decremented `len`, so `self.len` is a valid,
        // initialized index. `ptr::read` moves the value out without dropping
        // the source, which is correct since the slot is now considered uninitialized.
        let value = unsafe { std::ptr::read(self.storage[self.len].as_ptr()) };
        Some(value)
    }

    /// Clears the vector, dropping all of its elements.
    pub fn clear(&mut self) {
        self.truncate(0);
    }

    /// Truncates the vector to a new, shorter length.
    ///
    /// **Panics** if `new_len` > `len()`.
    pub fn truncate(&mut self, new_len: usize) {
        assert!(new_len <= self.len, "Cannot truncate to a larger size");
        if new_len < self.len {
            // SAFETY: The slice from `new_len..self.len` is guaranteed to be initialized.
            // `drop_in_place` will correctly drop all elements in that range.
            unsafe {
                let s = &mut self.storage[new_len..self.len];
                std::ptr::drop_in_place(s as *mut [MaybeUninit<T>] as *mut [T]);
            }
        }
        self.len = new_len;
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    /// Removes an element freom the vector and returns it.
    ///
    /// The removed element is replaced with the last element, so this does not preserve ordering
    /// of the remaining elements.
    ///
    /// **Panics** if `i` is out of bounds.
    pub fn swap_remove(&mut self, i: usize) -> T {
        let len = self.len();
        if i >= len {
            panic!(
                "swap_remove index out of bounds: the len is {} but the index is {}",
                self.len(),
                i
            );
        }
        self.swap(i, len - 1);
        self.pop().unwrap()
    }

    /// Retains only the elements specified by the given predicate.
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        let n = self.len();
        let mut j = 0;
        for i in 0..n {
            if f(&self[i]) {
                if i != j {
                    self.swap(i, j);
                }
                j += 1;
            }
        }
        self.truncate(j);
    }
}

impl<T, const N: usize> Drop for InlineVec<T, N> {
    fn drop(&mut self) {
        // Ensure all our elements are dropped.
        self.clear();
    }
}

impl<T, const N: usize> std::fmt::Debug for InlineVec<T, N>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self[..].fmt(f)
    }
}

impl<T, const N: usize> Default for InlineVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Clone for InlineVec<T, N>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut new = Self::new();
        for item in &self[..] {
            new.push(item.clone());
        }
        new
    }
}

impl<T, const N: usize> PartialEq for InlineVec<T, N>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self[..] == other[..]
    }
}

impl<T, const N: usize> Eq for InlineVec<T, N> where T: Eq {}

impl<T, const N: usize> PartialOrd for InlineVec<T, N>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self[..].partial_cmp(&other[..])
    }
}

impl<T, const N: usize> Ord for InlineVec<T, N>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self[..].cmp(&other[..])
    }
}

impl<T, const N: usize> std::hash::Hash for InlineVec<T, N>
where
    T: std::hash::Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self[..].hash(state);
    }
}

impl<T, const N: usize> std::ops::Deref for InlineVec<T, N> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        // SAFETY: The first `len` elements of our storage are guaranteed to be initialized, and
        // `MaybeUninit<T>` has the same memory layout as `T`, so the transmutation from
        // `&[MaybeUninit<T>] to `&[T]` is safe.
        unsafe {
            let slice = &self.storage[..self.len];
            &*(slice as *const [MaybeUninit<T>] as *const [T])
        }
    }
}

impl<T, const N: usize> std::ops::DerefMut for InlineVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: See the comment in the `Deref` impl above.
        unsafe {
            let slice = &mut self.storage[..self.len];
            &mut *(slice as *mut [MaybeUninit<T>] as *mut [T])
        }
    }
}
impl<T, const N: usize> AsRef<[T]> for InlineVec<T, N> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, const N: usize> AsMut<[T]> for InlineVec<T, N> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T, const N: usize> std::borrow::Borrow<[T]> for InlineVec<T, N> {
    fn borrow(&self) -> &[T] {
        self
    }
}

impl<T, const N: usize> std::borrow::BorrowMut<[T]> for InlineVec<T, N> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self
    }
}

/// Error type for `InlineVec::try_push()` and the `TryFrom` impl.
#[derive(Debug, PartialEq, Eq)]
pub struct CapacityExceededError;

impl<T, const N: usize> TryFrom<&[T]> for InlineVec<T, N>
where
    T: Clone,
{
    type Error = CapacityExceededError;
    /// Constructs an `InlineVec` by cloning the elements of `slice`. Returns
    /// `Err(CapacityExceededError)` if the length of the slice exceeds the capacity of the vector.
    fn try_from(slice: &[T]) -> Result<Self, Self::Error> {
        if slice.len() > N {
            Err(CapacityExceededError)
        } else {
            let mut vec = Self::new();
            for item in slice {
                vec.push(item.clone());
            }
            Ok(vec)
        }
    }
}

impl<T, const N: usize> From<[T; N]> for InlineVec<T, N> {
    /// Constructs an `InlineVec` by taking ownership of an array.
    fn from(array: [T; N]) -> Self {
        Self {
            storage: {
                let ptr = &array as *const [T; N] as *const [MaybeUninit<T>; N];
                // SAFETY: We have exclusive ownership of `array`, so this is fine as long as we
                // `forget` the original value so it doesn't get double-freed.
                let owned_arr = unsafe { ptr.read() };
                std::mem::forget(array);
                owned_arr
            },
            len: N,
        }
    }
}

/// **Panics** if the iterator yields more than `N` elements.
impl<T, const N: usize> FromIterator<T> for InlineVec<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut result = Self::new();
        for item in iter {
            result.push(item)
        }
        result
    }
}

/// **Panics** if the iterator yields more elements than the available capacity.
impl<T, const N: usize> Extend<T> for InlineVec<T, N> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }
}

pub struct IntoIter<T, const N: usize> {
    vec: std::mem::ManuallyDrop<InlineVec<T, N>>,
    pos: usize,
}

impl<T, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.vec.len {
            None
        } else {
            // SAFETY: `self.pos` is within the initialized range (0..`len`). `std::ptr::read`
            // safely moves the value out, leaving the slot logically uninitialized. In general,
            // this violates our primary invariant, but in this instance we have exclusive
            // ownership of the vector, and we never decrement `self.pos`, so this slot will never
            // be read from again (note that we are careful in our implementation of `Drop` to only
            // read elements from `pos..len`).
            let value = unsafe { std::ptr::read(self.vec.storage[self.pos].as_ptr()) };
            self.pos += 1;
            Some(value)
        }
    }
}

impl<T, const N: usize> Drop for IntoIter<T, N> {
    fn drop(&mut self) {
        if self.pos < self.vec.len {
            // If the iterator is dropped before being exhausted, we must drop any remaining
            // (initialized) elements.
            // SAFETY: The slice from `pos..len` is guaranteed to be initialized (see the comment
            // in the `Iterator` impl above), and therefore its elements must be dropped when we go
            // out of scope (the vector dies with us since we have exclusive ownership of it).
            unsafe {
                let range_to_drop = self.pos..self.vec.len;
                let slice = &mut self.vec.storage[range_to_drop];
                std::ptr::drop_in_place(slice as *mut [MaybeUninit<T>] as *mut [T]);
            }
        }
    }
}

impl<T, const N: usize> IntoIterator for InlineVec<T, N> {
    type IntoIter = IntoIter<T, N>;
    type Item = T;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            vec: std::mem::ManuallyDrop::new(self),
            pos: 0,
        }
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a InlineVec<T, N> {
    type IntoIter = std::slice::Iter<'a, T>;
    type Item = &'a T;
    fn into_iter(self) -> Self::IntoIter {
        (*self).iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut InlineVec<T, N> {
    type IntoIter = std::slice::IterMut<'a, T>;
    type Item = &'a mut T;
    fn into_iter(self) -> Self::IntoIter {
        (*self).iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::{CapacityExceededError, InlineVec};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    const CAP: usize = 4;

    #[test]
    fn test_new_is_empty() {
        let v: InlineVec<u32, CAP> = InlineVec::new();
        assert_eq!(v.len(), 0);
        assert!(v.is_empty());
        assert_eq!(v.capacity(), CAP);
    }

    #[test]
    fn test_try_push() {
        let mut v: InlineVec<u32, CAP> = InlineVec::new();
        assert_eq!(v.try_push(1), Ok(()));
        assert_eq!(v.try_push(2), Ok(()));
        assert_eq!(v.try_push(3), Ok(()));
        assert_eq!(v.try_push(4), Ok(()));
        assert_eq!(v.try_push(5), Err(CapacityExceededError));
        assert_eq!(*v, [1, 2, 3, 4]);
    }

    #[test]
    fn test_push_pop_and_index() {
        let mut v = InlineVec::<String, CAP>::new();
        v.push("hello".to_string());
        v.push("world".to_string());

        assert_eq!(v.len(), 2);
        assert_eq!(v[0], "hello");
        assert_eq!(v[1], "world");

        assert_eq!(v.pop(), Some("world".to_string()));
        assert_eq!(v.len(), 1);

        v[0] = "hella whirled".to_string();
        assert_eq!(v[0], "hella whirled");

        assert_eq!(v.pop(), Some("hella whirled".to_string()));
        assert_eq!(v.len(), 0);
        assert_eq!(v.pop(), None);
    }

    #[test]
    #[should_panic(expected = "InlineVec exceeded capacity (4)")]
    fn test_push_panics_on_full() {
        let mut v = InlineVec::<_, CAP>::new();
        for i in 0..CAP {
            v.push(i);
        }
        // This push should panic.
        v.push(CAP);
    }

    #[test]
    #[should_panic]
    fn test_index_out_of_bounds() {
        let mut v = InlineVec::<_, CAP>::new();
        v.push(1);
        let _ = v[1]; // len is 1, so index 1 is out of bounds
    }

    #[test]
    #[should_panic(expected = "Cannot truncate to a larger size")]
    fn test_truncate_panic() {
        let mut v = InlineVec::<u32, CAP>::new();
        v.push(1);
        v.truncate(2);
    }

    #[test]
    fn test_deref_mut_enables_slice_methods() {
        let mut v = InlineVec::<_, CAP>::new();
        v.push(4);
        v.push(2);
        v.push(3);
        v.push(1);

        v.sort();

        assert_eq!(*v, [1, 2, 3, 4]);
    }

    #[test]
    fn test_try_from_slice() {
        let _: InlineVec<u32, CAP> = InlineVec::try_from(&[1, 2, 3, 4][..])
            .expect("4 elements should fit in a vector with capacity 4");
        // ...but 5 elements will not.
        assert_eq!(
            InlineVec::<u32, CAP>::try_from(&[1, 2, 3, 4, 5][..]),
            Err(CapacityExceededError)
        );
    }

    #[test]
    fn test_from_iterator() {
        let data = vec![10, 20, 30];
        let v: InlineVec<_, CAP> = data.into_iter().collect();
        assert_eq!(v.len(), 3);
        assert_eq!(&v[..], &[10, 20, 30]);
    }

    // A helper struct that increments an atomic counter when it is dropped.
    // This lets us verify that our container drops its elements correctly.
    #[derive(Debug, Clone)]
    struct DropSpy {
        id: usize,
        counter: Arc<AtomicUsize>,
    }

    impl DropSpy {
        fn new(id: usize, counter: &Arc<AtomicUsize>) -> Self {
            Self {
                id,
                counter: Arc::clone(counter),
            }
        }
    }

    impl Drop for DropSpy {
        fn drop(&mut self) {
            self.counter.fetch_add(1, Ordering::SeqCst);
        }
    }

    // We only care about the id for equality checks in tests.
    impl PartialEq for DropSpy {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }

    #[test]
    fn test_from_array() {
        let drop_counter = Arc::new(AtomicUsize::new(0));
        let array = [
            DropSpy::new(1, &drop_counter),
            DropSpy::new(2, &drop_counter),
            DropSpy::new(3, &drop_counter),
            DropSpy::new(4, &drop_counter),
        ];

        let vec: InlineVec<DropSpy, _> = array.into();

        // Ensure nothing was dropped in the conversion.
        assert_eq!(drop_counter.load(Ordering::SeqCst), 0);
        assert_eq!(vec.len(), 4);

        // Now, drop the collected items and check the counter.
        drop(vec);
        assert_eq!(drop_counter.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn test_clear_and_truncate() {
        let drop_counter = Arc::new(AtomicUsize::new(0));
        let mut v = InlineVec::<_, CAP>::new();
        v.push(DropSpy::new(1, &drop_counter));
        v.push(DropSpy::new(2, &drop_counter));
        v.push(DropSpy::new(3, &drop_counter));
        v.push(DropSpy::new(4, &drop_counter));

        assert_eq!(v.len(), 4);

        v.truncate(2);
        assert_eq!(v.len(), 2);
        assert_eq!(drop_counter.load(Ordering::SeqCst), 2); // Dropped spies 3 and 4
        assert_eq!(v[0].id, 1);
        assert_eq!(v[1].id, 2);

        v.clear();
        assert!(v.is_empty());
        assert_eq!(drop_counter.load(Ordering::SeqCst), 4); // Dropped spies 1 and 2
    }

    #[test]
    fn test_drop_behavior_of_vec() {
        let drop_counter = Arc::new(AtomicUsize::new(0));
        {
            let mut v = InlineVec::<_, CAP>::new();
            v.push(DropSpy::new(1, &drop_counter));
            v.push(DropSpy::new(2, &drop_counter));
            assert_eq!(drop_counter.load(Ordering::SeqCst), 0);
        } // v is dropped here
        assert_eq!(drop_counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_into_iterator_partial_consumption() {
        let drop_counter = Arc::new(AtomicUsize::new(0));
        let mut v = InlineVec::<_, CAP>::new();
        v.push(DropSpy::new(1, &drop_counter));
        v.push(DropSpy::new(2, &drop_counter));
        v.push(DropSpy::new(3, &drop_counter));
        v.push(DropSpy::new(4, &drop_counter));

        // Take only two items. The iterator will be dropped after the loop.
        let taken: Vec<_> = v.into_iter().take(2).collect();

        // Spies 3 and 4 should have been dropped by the iterator's drop handler.
        assert_eq!(drop_counter.load(Ordering::SeqCst), 2);
        assert_eq!(taken.len(), 2);

        // Now drop the two we took.
        drop(taken);
        assert_eq!(drop_counter.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn test_into_iterator_full_consumption() {
        let drop_counter = Arc::new(AtomicUsize::new(0));
        let mut v = InlineVec::<_, CAP>::new();
        v.push(DropSpy::new(1, &drop_counter));
        v.push(DropSpy::new(2, &drop_counter));
        v.push(DropSpy::new(3, &drop_counter));
        v.push(DropSpy::new(4, &drop_counter));

        let mut items = vec![];
        for item in v {
            items.push(item);
        }

        // The items were moved into the `items` vec.
        // The InlineVec was consumed, but it was empty by the end of iteration,
        // so its drop does nothing. No spies should be dropped.
        assert_eq!(drop_counter.load(Ordering::SeqCst), 0);
        assert_eq!(items.len(), 4);

        // Now, drop the collected items and check the counter.
        drop(items);
        assert_eq!(drop_counter.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn test_retain() {
        let mut v = InlineVec::<_, CAP>::new();
        v.push(1);
        v.push(2);
        v.push(3);
        v.push(4);

        v.retain(|i| i % 2 == 0);

        assert_eq!(&v[..], &[2, 4]);
    }

    #[test]
    fn test_swap_remove() {
        // Edge case: one element
        {
            let mut v = InlineVec::<i32, CAP>::new();
            v.push(1);

            assert_eq!(v.swap_remove(0), 1);
            assert!(v.is_empty());
        }

        // Edge case: two elements, swap-removing the first element.
        {
            let mut v = InlineVec::<i32, CAP>::new();
            v.push(1);
            v.push(2);

            assert_eq!(v.swap_remove(0), 1);
            assert_eq!(&v[..], &[2]);
        }

        // Edge case: two elements, swap-removing the second.
        {
            let mut v = InlineVec::<i32, CAP>::new();
            v.push(1);
            v.push(2);

            assert_eq!(v.swap_remove(1), 2);
            assert_eq!(&v[..], &[1]);
        }

        // General case.
        {
            let mut v = InlineVec::<i32, CAP>::new();
            v.push(1);
            v.push(2);
            v.push(3);
            v.push(4);

            assert_eq!(v.swap_remove(1), 2);
            assert_eq!(&v[..], &[1, 4, 3]);
        }
    }
}
