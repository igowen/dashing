// Copyright 2018 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// Trait for implementing bit sets on top of unsigned integer types.
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
    #[inline]
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

// Implement `BitSet` for all the unsigned ints.
bitset_impl!(u8, 8);
bitset_impl!(u16, 16);
bitset_impl!(u32, 32);
bitset_impl!(u64, 64);
bitset_impl!(u128, 128);

#[cfg(test)]
mod tests {
    use crate::ecs::bitset::*;
    #[test]
    fn bitset() {
        let mut x: u32 = 0;
        // Should default to unset (0).
        for i in 0..32 {
            assert!(x.get_bit(i) == false);
        }
        // Setting one bit shouldn't have any effect on the others.
        x.set_bit(12);
        assert!(x.get_bit(12));
        for i in 0..32 {
            if i != 12 {
                assert!(x.get_bit(i) == false);
            }
        }
        // Same for clearing one bit.
        x.clear_bit(12);
        for i in 0..32 {
            assert!(x.get_bit(i) == false);
        }

        x = 0xffffffff;
        for i in 0..32 {
            assert!(x.get_bit(i) == true);
        }

        x.clear_bit(14);
        assert!(x.get_bit(14) == false);
        for i in 0..32 {
            if i != 14 {
                assert!(x.get_bit(i) == true);
            }
        }
    }

    #[test]
    fn bitset_iter() {
        let x: u16 = 0b1010011101101010;
        let idxs = x.iter().collect::<Vec<_>>();
        assert_eq!(idxs, vec![1, 3, 5, 6, 8, 9, 10, 13, 15]);
    }
}
