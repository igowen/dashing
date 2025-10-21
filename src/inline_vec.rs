/// Simple array-backed variable-length vector. The size of the backing array (`N`) is the maximum
/// capacity of the vector.
///
/// **Panics** if push() is called when there is no remaining capacity.
#[derive(Clone, Copy)]
pub struct InlineVec<T, const N: usize> {
    storage: [T; N],
    len: usize,
}

impl<T, const N: usize> Default for InlineVec<T, N>
where
    T: Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> InlineVec<T, N>
where
    T: Default,
{
    /// Construct a new, empty vector with default-initialized storage.
    pub fn new() -> Self {
        Self {
            storage: [(); N].map(|_| T::default()),
            len: 0,
        }
    }
}

impl<T, const N: usize> InlineVec<T, N> {
    /// Construct a new vector with the provided storage array. **Note that the size of
    /// the returned vector is still 0**; the values just serve as placeholders since we
    /// don't know how to construct them implicitly (`T` is not `Default` here).
    pub fn new_with_storage(storage: [T; N]) -> Self {
        Self { storage, len: 0 }
    }

    /// Construct a full vector with the provided backing storage. This the same as
    /// `new_with_storage` but the resulting vector has length N instead of 0.
    pub fn new_with_data(storage: [T; N]) -> Self {
        Self { storage, len: N }
    }
}

impl<T, const N: usize> InlineVec<T, N> {
    /// Pushes a value onto the end of the vector.
    ///
    /// **Panics** if this would exceed the vector's capacity (`N`).
    pub fn push(&mut self, item: T) {
        if self.len >= N {
            panic!("InlineVec exceeded capacity ({})", N);
        }
        self.storage[self.len] = item;
        self.len += 1;
    }

    /// Clears the vector. Note that this merely sets the length to 0; the elements are not dropped
    /// until either the vector is dropped or they are overwritten with other values.
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl<T, const N: usize, I> std::ops::Index<I> for InlineVec<T, N>
where
    I: std::slice::SliceIndex<[T]>,
{
    type Output = <I as std::slice::SliceIndex<[T]>>::Output;
    fn index(&self, index: I) -> &Self::Output {
        &self.storage[0..self.len][index]
    }
}

impl<T, const N: usize, I> std::ops::IndexMut<I> for InlineVec<T, N>
where
    I: std::slice::SliceIndex<[T]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.storage[..self.len][index]
    }
}

impl<T, const N: usize> std::ops::Deref for InlineVec<T, N> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &self.storage[..self.len]
    }
}

impl<T, const N: usize> std::ops::DerefMut for InlineVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.storage[..self.len]
    }
}

impl<T, const N: usize> FromIterator<T> for InlineVec<T, N>
where
    T: Default,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut result = Self::new();
        for item in iter {
            result.push(item)
        }
        result
    }
}

type InlineVecIter<T, const N: usize> = std::iter::Take<<[T; N] as IntoIterator>::IntoIter>;

impl<T, const N: usize> IntoIterator for InlineVec<T, N> {
    type IntoIter = InlineVecIter<T, N>;
    type Item = T;
    fn into_iter(self) -> Self::IntoIter {
        self.storage.into_iter().take(self.len)
    }
}
