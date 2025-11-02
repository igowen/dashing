use crate::geometry::*;
use crate::inline_vec::*;
use crate::{point, vector};

/// Shorthand for `Cardinal::North`.
pub const N: Cardinal = Cardinal::North;
/// Shorthand for `Cardinal::South`.
pub const S: Cardinal = Cardinal::South;
/// Shorthand for `Cardinal::East`.
pub const E: Cardinal = Cardinal::East;
/// Shorthand for `Cardinal::West`.
pub const W: Cardinal = Cardinal::West;

/// Shorthand for `Diagonal::Northwest`.
pub const NW: Diagonal = Diagonal::Northwest;
/// Shorthand for `Diagonal::Northeast`.
pub const NE: Diagonal = Diagonal::Northeast;
/// Shorthand for `Diagonal::Southwest`.
pub const SW: Diagonal = Diagonal::Southwest;
/// Shorthand for `Diagonal::Southeast`.
pub const SE: Diagonal = Diagonal::Southeast;

/// Shorthand for `Turn::Left`.
pub const L: Turn = Turn::Left;
/// Shorthand for `Turn::Right`.
pub const R: Turn = Turn::Right;

/// Represents the four cardinal directions.
#[derive(Default, Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cardinal {
    #[default]
    North,
    South,
    East,
    West,
}

/// Represents the four ordinal/intercardinal directions.
#[derive(Default, Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Diagonal {
    #[default]
    Northeast,
    Northwest,
    Southeast,
    Southwest,
}

/// Represents a 90 or 180 degree turn relative to the current orientation.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Turn {
    Left,
    Right,
    Reverse,
}

impl Cardinal {
    /// Returns the new direction after turning.
    pub const fn turn(self, t: Turn) -> Self {
        match t {
            Turn::Right => match self {
                Cardinal::North => Cardinal::East,
                Cardinal::East => Cardinal::South,
                Cardinal::South => Cardinal::West,
                Cardinal::West => Cardinal::North,
            },
            Turn::Left => match self {
                Cardinal::North => Cardinal::West,
                Cardinal::West => Cardinal::South,
                Cardinal::South => Cardinal::East,
                Cardinal::East => Cardinal::North,
            },
            Turn::Reverse => self.flip(),
        }
    }

    /// Returns the opposite direction.
    pub const fn flip(self) -> Self {
        match self {
            Cardinal::North => Cardinal::South,
            Cardinal::East => Cardinal::West,
            Cardinal::South => Cardinal::North,
            Cardinal::West => Cardinal::East,
        }
    }

    /// Returns the `(dx, dy)` coordinate offset for this direction.
    pub const fn offset(self) -> Vector {
        match self {
            Cardinal::North => vector![0, -1],
            Cardinal::South => vector![0, 1],
            Cardinal::East => vector![1, 0],
            Cardinal::West => vector![-1, 0],
        }
    }

    /// Calculates the `Turn` needed to get from `self` to `other`. Returns `None` if no turn is
    /// needed (i.e., `self` == `other`).
    pub const fn turn_for(self, other: Cardinal) -> Option<Turn> {
        match self {
            Cardinal::North => match other {
                Cardinal::North => None,
                Cardinal::South => Some(Turn::Reverse),
                Cardinal::East => Some(Turn::Right),
                Cardinal::West => Some(Turn::Left),
            },
            Cardinal::South => match other {
                Cardinal::North => Some(Turn::Reverse),
                Cardinal::South => None,
                Cardinal::East => Some(Turn::Left),
                Cardinal::West => Some(Turn::Right),
            },
            Cardinal::East => match other {
                Cardinal::North => Some(Turn::Left),
                Cardinal::South => Some(Turn::Right),
                Cardinal::East => None,
                Cardinal::West => Some(Turn::Reverse),
            },
            Cardinal::West => match other {
                Cardinal::North => Some(Turn::Right),
                Cardinal::South => Some(Turn::Left),
                Cardinal::East => Some(Turn::Reverse),
                Cardinal::West => None,
            },
        }
    }

    /// Returns the two directions perpendicular to `self`.
    pub const fn perp(self) -> TurnData<Self> {
        match self {
            Cardinal::North => TurnData {
                left: Cardinal::West,
                right: Cardinal::East,
            },
            Cardinal::South => TurnData {
                left: Cardinal::East,
                right: Cardinal::West,
            },
            Cardinal::East => TurnData {
                left: Cardinal::North,
                right: Cardinal::South,
            },
            Cardinal::West => TurnData {
                left: Cardinal::South,
                right: Cardinal::North,
            },
        }
    }

    /// Returns the three directions that are not `self`.
    pub const fn complement(self) -> [Cardinal; 3] {
        match self {
            Cardinal::North => [Cardinal::South, Cardinal::East, Cardinal::West],
            Cardinal::South => [Cardinal::North, Cardinal::East, Cardinal::West],
            Cardinal::East => [Cardinal::North, Cardinal::South, Cardinal::West],
            Cardinal::West => [Cardinal::North, Cardinal::South, Cardinal::East],
        }
    }
}

impl Diagonal {
    /// Returns the new direction after turning.
    pub const fn turn(self, t: Turn) -> Self {
        match t {
            Turn::Right => match self {
                Diagonal::Northeast => Diagonal::Southeast,
                Diagonal::Southeast => Diagonal::Southwest,
                Diagonal::Southwest => Diagonal::Northwest,
                Diagonal::Northwest => Diagonal::Northeast,
            },
            Turn::Left => match self {
                Diagonal::Northeast => Diagonal::Northwest,
                Diagonal::Northwest => Diagonal::Southwest,
                Diagonal::Southwest => Diagonal::Southeast,
                Diagonal::Southeast => Diagonal::Northeast,
            },
            Turn::Reverse => self.flip(),
        }
    }

    /// Returns the opposite direction.
    pub const fn flip(self) -> Self {
        match self {
            Diagonal::Northeast => Diagonal::Southwest,
            Diagonal::Northwest => Diagonal::Southeast,
            Diagonal::Southeast => Diagonal::Northwest,
            Diagonal::Southwest => Diagonal::Northeast,
        }
    }

    /// Returns the `(dx, dy)` coordinate offset for this direction.
    pub const fn offset(self) -> Vector {
        match self {
            Diagonal::Northeast => vector![1, -1],
            Diagonal::Northwest => vector![-1, -1],
            Diagonal::Southeast => vector![1, 1],
            Diagonal::Southwest => vector![-1, 1],
        }
    }

    /*
        /// Calculates the `Turn` needed to get from `self` to `other`. Returns `None` if no turn is
        /// needed (i.e., `self` == `other`).
        pub const fn turn_for(self, other: Diagonal) -> Option<Turn> {
            match self {
                Diagonal::North => match other {
                    Diagonal::North => None,
                    Diagonal::South => Some(Turn::Reverse),
                    Diagonal::East => Some(Turn::Right),
                    Diagonal::West => Some(Turn::Left),
                },
                Diagonal::South => match other {
                    Diagonal::North => Some(Turn::Reverse),
                    Diagonal::South => None,
                    Diagonal::East => Some(Turn::Left),
                    Diagonal::West => Some(Turn::Right),
                },
                Diagonal::East => match other {
                    Diagonal::North => Some(Turn::Left),
                    Diagonal::South => Some(Turn::Right),
                    Diagonal::East => None,
                    Diagonal::West => Some(Turn::Reverse),
                },
                Diagonal::West => match other {
                    Diagonal::North => Some(Turn::Right),
                    Diagonal::South => Some(Turn::Left),
                    Diagonal::East => Some(Turn::Reverse),
                    Diagonal::West => None,
                },
            }
        }

    */
    /// Returns the two directions perpendicular to `self`.
    pub const fn perp(self) -> TurnData<Self> {
        match self {
            Diagonal::Northwest => TurnData {
                left: Diagonal::Southwest,
                right: Diagonal::Northeast,
            },
            Diagonal::Northeast => TurnData {
                left: Diagonal::Northwest,
                right: Diagonal::Southeast,
            },
            Diagonal::Southeast => TurnData {
                left: Diagonal::Northeast,
                right: Diagonal::Southwest,
            },
            Diagonal::Southwest => TurnData {
                left: Diagonal::Southeast,
                right: Diagonal::Northwest,
            },
        }
    }

    /// Returns the three directions that are not `self`.
    pub const fn complement(self) -> [Diagonal; 3] {
        match self {
            Diagonal::Northeast => [
                Diagonal::Northwest,
                Diagonal::Southeast,
                Diagonal::Southwest,
            ],
            Diagonal::Northwest => [
                Diagonal::Northeast,
                Diagonal::Southeast,
                Diagonal::Southwest,
            ],
            Diagonal::Southeast => [
                Diagonal::Northeast,
                Diagonal::Northwest,
                Diagonal::Southwest,
            ],
            Diagonal::Southwest => [
                Diagonal::Northeast,
                Diagonal::Northwest,
                Diagonal::Southeast,
            ],
        }
    }
}

/// Holds data for the 4 compass directions.
#[derive(Default, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct CardinalData<T> {
    /// North
    pub n: T,
    /// South
    pub s: T,
    /// East
    pub e: T,
    /// West
    pub w: T,
}

impl<T> CardinalData<T> {
    pub fn get(&self, dir: Cardinal) -> &T {
        match dir {
            Cardinal::North => &self.n,
            Cardinal::South => &self.s,
            Cardinal::East => &self.e,
            Cardinal::West => &self.w,
        }
    }

    pub fn get_mut(&mut self, dir: Cardinal) -> &mut T {
        match dir {
            Cardinal::North => &mut self.n,
            Cardinal::South => &mut self.s,
            Cardinal::East => &mut self.e,
            Cardinal::West => &mut self.w,
        }
    }
    /// Get N/S/E/W data as a tuple
    pub fn as_tuple(&self) -> (&T, &T, &T, &T) {
        (&self.n, &self.s, &self.e, &self.w)
    }

    pub fn to_tuple(self) -> (T, T, T, T) {
        (self.n, self.s, self.e, self.w)
    }
}

impl<T> CardinalData<T>
where
    T: Default + Copy + PartialEq,
{
    pub fn count(self) -> u8 {
        (self.n != T::default()) as u8
            + (self.s != T::default()) as u8
            + (self.e != T::default()) as u8
            + (self.w != T::default()) as u8
    }

    pub fn get_set_dirs(self) -> InlineVec<Cardinal, 4> {
        let mut result = InlineVec::new();
        if self.n != T::default() {
            result.push(N)
        }
        if self.s != T::default() {
            result.push(S)
        }
        if self.e != T::default() {
            result.push(E)
        }
        if self.w != T::default() {
            result.push(W)
        }
        result
    }
}

impl<T> CardinalData<T>
where
    T: Copy,
{
    pub fn set_all(&mut self, value: T) {
        self.n = value;
        self.s = value;
        self.e = value;
        self.w = value;
    }
}

/// Holds data for the 4 intercardinal directions.
#[derive(Default, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct DiagonalData<T> {
    /// Northeast
    pub ne: T,
    /// Northwest
    pub nw: T,
    /// Southeast
    pub se: T,
    /// Southwest
    pub sw: T,
}

impl<T> DiagonalData<T> {
    pub fn side(&self, dir: Cardinal) -> TurnData<&T> {
        match dir {
            Cardinal::North => TurnData {
                left: &self.nw,
                right: &self.ne,
            },
            Cardinal::South => TurnData {
                left: &self.se,
                right: &self.sw,
            },
            Cardinal::East => TurnData {
                left: &self.ne,
                right: &self.se,
            },
            Cardinal::West => TurnData {
                left: &self.sw,
                right: &self.nw,
            },
        }
    }

    pub fn side_mut(&mut self, dir: Cardinal) -> TurnData<&mut T> {
        match dir {
            Cardinal::North => TurnData {
                left: &mut self.nw,
                right: &mut self.ne,
            },
            Cardinal::South => TurnData {
                left: &mut self.se,
                right: &mut self.sw,
            },
            Cardinal::East => TurnData {
                left: &mut self.ne,
                right: &mut self.se,
            },
            Cardinal::West => TurnData {
                left: &mut self.sw,
                right: &mut self.nw,
            },
        }
    }
}

impl<T> DiagonalData<T>
where
    T: Copy,
{
    pub fn set_all(&mut self, data: T) {
        self.ne = data;
        self.nw = data;
        self.se = data;
        self.sw = data;
    }
}

pub struct OctantData<T> {
    cardinal: CardinalData<T>,
    ordinal: OrthogonalData<T>,
}

/// Holds data for the 2 orthogonal directions (N/S and E/W).
#[derive(Default, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct OrthogonalData<T> {
    pub ns: T,
    pub ew: T,
}

#[derive(Default, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct TurnData<T> {
    pub left: T,
    pub right: T,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cardinal_direction_rules() {
        for dir in [N, S, E, W] {
            assert_ne!(dir, dir.flip());
            assert_ne!(dir, dir.turn(Turn::Left));
            assert_ne!(dir, dir.turn(Turn::Right));
            assert_ne!(dir, dir.turn(Turn::Reverse));

            assert_eq!(dir, dir.flip().flip());

            assert_eq!(dir, dir.turn(Turn::Reverse).turn(Turn::Reverse));

            assert_eq!(dir, dir.turn(Turn::Left).turn(Turn::Right));
            assert_eq!(dir, dir.turn(Turn::Right).turn(Turn::Left));
            assert_eq!(
                dir,
                dir.turn(Turn::Right)
                    .turn(Turn::Right)
                    .turn(Turn::Right)
                    .turn(Turn::Right)
            );
            assert_eq!(
                dir,
                dir.turn(Turn::Left)
                    .turn(Turn::Left)
                    .turn(Turn::Left)
                    .turn(Turn::Left)
            );

            assert_eq!(dir.turn(Turn::Right).flip(), dir.turn(Turn::Left));
            assert_eq!(dir.turn(Turn::Left).flip(), dir.turn(Turn::Right));

            assert_eq!(dir.turn(Turn::Right).turn(Turn::Right), dir.flip());
            assert_eq!(dir.turn(Turn::Left).turn(Turn::Left), dir.flip());

            assert_eq!(dir.perp().right, dir.turn(Turn::Right));
            assert_eq!(dir.perp().left, dir.turn(Turn::Left));
        }
    }

    #[test]
    fn test_diagonal_direction_rules() {
        for dir in [NW, NE, SW, SE] {
            assert_ne!(dir, dir.flip());
            assert_ne!(dir, dir.turn(Turn::Left));
            assert_ne!(dir, dir.turn(Turn::Right));
            assert_ne!(dir, dir.turn(Turn::Reverse));

            assert_eq!(dir, dir.flip().flip());

            assert_eq!(dir, dir.turn(Turn::Reverse).turn(Turn::Reverse));

            assert_eq!(dir, dir.turn(Turn::Left).turn(Turn::Right));
            assert_eq!(dir, dir.turn(Turn::Right).turn(Turn::Left));
            assert_eq!(
                dir,
                dir.turn(Turn::Right)
                    .turn(Turn::Right)
                    .turn(Turn::Right)
                    .turn(Turn::Right)
            );
            assert_eq!(
                dir,
                dir.turn(Turn::Left)
                    .turn(Turn::Left)
                    .turn(Turn::Left)
                    .turn(Turn::Left)
            );

            assert_eq!(dir.turn(Turn::Right).flip(), dir.turn(Turn::Left));
            assert_eq!(dir.turn(Turn::Left).flip(), dir.turn(Turn::Right));

            assert_eq!(dir.turn(Turn::Right).turn(Turn::Right), dir.flip());
            assert_eq!(dir.turn(Turn::Left).turn(Turn::Left), dir.flip());

            assert_eq!(dir.perp().right, dir.turn(Turn::Right));
            assert_eq!(dir.perp().left, dir.turn(Turn::Left));
        }
    }
}
