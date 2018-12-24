pub trait BitSet {
    /// Number of bits stored in this bitset.
    const SIZE: usize;
    /// Return `true` iff bit `i` is set.
    fn get_bit(&self, i: usize) -> bool;
    /// Set bit `i` to `true`.
    fn set_bit(&mut self, i: usize);
    /// Set bit `i` to `false`.
    fn clear_bit(&mut self, i: usize);

    /// Type returned by `iter()`.
    type Iter: Iterator<Item = usize>;
    /// Iterate over the bits (by position) that are set.
    fn iter(&self) -> Self::Iter;
}

pub struct BitSetIter<T: BitSet> {
    bits: T,
    curr: usize,
}

impl<T: BitSet> Iterator for BitSetIter<T> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        while self.curr < T::SIZE && !self.bits.get_bit(self.curr) {
            self.curr += 1;
        }
        if self.curr < T::SIZE {
            let i = self.curr;
            self.curr += 1;
            Some(i)
        } else {
            None
        }
    }
}

macro_rules! bitset_impl {
    ($t:ty,$b:tt) => {
        impl BitSet for $t {
            const SIZE: usize = $b;
            type Iter = BitSetIter<$t>;
            #[inline]
            fn get_bit(&self, i: usize) -> bool {
                if i < Self::SIZE {
                    (self & (1 << i)) != 0
                } else {
                    false
                }
            }

            #[inline]
            fn set_bit(&mut self, i: usize) {
                if i < Self::SIZE {
                    *self |= 1 << i;
                }
            }

            #[inline]
            fn clear_bit(&mut self, i: usize) {
                if i < Self::SIZE {
                    *self &= !(1 << i);
                }
            }
            #[inline]
            fn iter(&self) -> Self::Iter {
                BitSetIter {
                    bits: *self,
                    curr: 0,
                }
            }
        }
    };
}

bitset_impl!(u8, 8);
bitset_impl!(u16, 16);
bitset_impl!(u32, 32);
bitset_impl!(u64, 64);
bitset_impl!(u128, 128);
