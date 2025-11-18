use crate::geometry::*;
use crate::inline_vec::*;
use crate::vector;

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

/// Shorthand for `Rotation::Ccw`.
pub const L: Rotation = Rotation::Ccw;
/// Shorthand for `Rotation::Cw`.
pub const R: Rotation = Rotation::Cw;

/// Represents the four cardinal directions.
#[derive(Default, Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cardinal {
    /// North
    #[default]
    North,
    /// South
    South,
    /// East
    East,
    /// West
    West,
}

/// Represents the four ordinal/intercardinal directions.
#[derive(Default, Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Diagonal {
    /// Northeast
    #[default]
    Northeast,
    /// Northwest
    Northwest,
    /// Southeast
    Southeast,
    /// Southwest
    Southwest,
}

/// One of the 8 compass directions.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompassDirection {
    /// N/S/E/W
    Cardinal(Cardinal),
    /// NE/NW/SE/SW
    Diagonal(Diagonal),
}

impl Default for CompassDirection {
    fn default() -> Self {
        CompassDirection::Cardinal(Cardinal::North)
    }
}

impl CompassDirection {
    /// Returns the euclidean position offset representing the shortest move in the given
    /// direction.
    pub const fn offset(self) -> Vector {
        match self {
            CompassDirection::Cardinal(c) => c.offset(),
            CompassDirection::Diagonal(d) => d.offset(),
        }
    }

    /// Returns the direction 180 degress from `self`.
    pub fn flip(self) -> Self {
        match self {
            CompassDirection::Cardinal(c) => c.flip().into(),
            CompassDirection::Diagonal(d) => d.flip().into(),
        }
    }
}

impl From<Cardinal> for CompassDirection {
    fn from(dir: Cardinal) -> Self {
        Self::Cardinal(dir)
    }
}

impl From<Diagonal> for CompassDirection {
    fn from(dir: Diagonal) -> Self {
        Self::Diagonal(dir)
    }
}

/// Represents a relative turn from the current orientation.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Rotation {
    /// Counterclockwise / Left
    Ccw,
    /// Clockwise / Right
    Cw,
}

impl Cardinal {
    /// Returns the new direction after turning.
    pub const fn turn(self, t: Rotation) -> Self {
        match t {
            Rotation::Cw => match self {
                Cardinal::North => Cardinal::East,
                Cardinal::East => Cardinal::South,
                Cardinal::South => Cardinal::West,
                Cardinal::West => Cardinal::North,
            },
            Rotation::Ccw => match self {
                Cardinal::North => Cardinal::West,
                Cardinal::West => Cardinal::South,
                Cardinal::South => Cardinal::East,
                Cardinal::East => Cardinal::North,
            },
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

    /// Calculates the `Rotation` needed to get from `self` to `other`. Returns `None` if no turn is
    /// needed (i.e., `self` == `other`) or it is not possible to reach `other` in a single turn
    /// (i.e., `self` == `other.flip()`).
    pub const fn turn_for(self, other: Cardinal) -> Option<Rotation> {
        match self {
            Cardinal::North => match other {
                Cardinal::North | Cardinal::South => None,
                Cardinal::East => Some(Rotation::Cw),
                Cardinal::West => Some(Rotation::Ccw),
            },
            Cardinal::South => match other {
                Cardinal::North | Cardinal::South => None,
                Cardinal::East => Some(Rotation::Ccw),
                Cardinal::West => Some(Rotation::Cw),
            },
            Cardinal::East => match other {
                Cardinal::North => Some(Rotation::Ccw),
                Cardinal::South => Some(Rotation::Cw),
                Cardinal::East | Cardinal::West => None,
            },
            Cardinal::West => match other {
                Cardinal::North => Some(Rotation::Cw),
                Cardinal::South => Some(Rotation::Ccw),
                Cardinal::East | Cardinal::West => None,
            },
        }
    }

    /// Returns the two directions perpendicular to `self`.
    pub const fn perp(self) -> RotationData<Self> {
        match self {
            Cardinal::North => RotationData {
                ccw: Cardinal::West,
                cw: Cardinal::East,
            },
            Cardinal::South => RotationData {
                ccw: Cardinal::East,
                cw: Cardinal::West,
            },
            Cardinal::East => RotationData {
                ccw: Cardinal::North,
                cw: Cardinal::South,
            },
            Cardinal::West => RotationData {
                ccw: Cardinal::South,
                cw: Cardinal::North,
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

    /// Returns the two diagonal directions adjacent to `self`.
    pub const fn diagonal_neighbors(self) -> (Diagonal, Diagonal) {
        match self {
            Cardinal::North => (Diagonal::Northeast, Diagonal::Northwest),
            Cardinal::South => (Diagonal::Southeast, Diagonal::Southwest),
            Cardinal::East => (Diagonal::Northeast, Diagonal::Southeast),
            Cardinal::West => (Diagonal::Northwest, Diagonal::Southwest),
        }
    }
}

use rand::prelude::*;
impl Distribution<Cardinal> for rand::distr::StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Cardinal {
        const CHOICES: [Cardinal; 4] = [
            Cardinal::North,
            Cardinal::South,
            Cardinal::East,
            Cardinal::West,
        ];

        *CHOICES.choose(rng).unwrap()
    }
}

impl Distribution<Diagonal> for rand::distr::StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Diagonal {
        const CHOICES: [Diagonal; 4] = [
            Diagonal::Northeast,
            Diagonal::Northwest,
            Diagonal::Southeast,
            Diagonal::Southwest,
        ];
        *CHOICES.choose(rng).unwrap()
    }
}

impl Distribution<CompassDirection> for rand::distr::StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> CompassDirection {
        const CHOICES: [CompassDirection; 8] = [
            CompassDirection::Cardinal(Cardinal::North),
            CompassDirection::Cardinal(Cardinal::South),
            CompassDirection::Cardinal(Cardinal::East),
            CompassDirection::Cardinal(Cardinal::West),
            CompassDirection::Diagonal(Diagonal::Northeast),
            CompassDirection::Diagonal(Diagonal::Northwest),
            CompassDirection::Diagonal(Diagonal::Southeast),
            CompassDirection::Diagonal(Diagonal::Southwest),
        ];

        *CHOICES.choose(rng).unwrap()
    }
}

impl Diagonal {
    /// Returns the new direction after turning.
    pub const fn turn(self, t: Rotation) -> Self {
        match t {
            Rotation::Cw => match self {
                Diagonal::Northeast => Diagonal::Southeast,
                Diagonal::Southeast => Diagonal::Southwest,
                Diagonal::Southwest => Diagonal::Northwest,
                Diagonal::Northwest => Diagonal::Northeast,
            },
            Rotation::Ccw => match self {
                Diagonal::Northeast => Diagonal::Northwest,
                Diagonal::Northwest => Diagonal::Southwest,
                Diagonal::Southwest => Diagonal::Southeast,
                Diagonal::Southeast => Diagonal::Northeast,
            },
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

    /// Returns the two directions perpendicular to `self`.
    pub const fn perp(self) -> RotationData<Self> {
        match self {
            Diagonal::Northwest => RotationData {
                ccw: Diagonal::Southwest,
                cw: Diagonal::Northeast,
            },
            Diagonal::Northeast => RotationData {
                ccw: Diagonal::Northwest,
                cw: Diagonal::Southeast,
            },
            Diagonal::Southeast => RotationData {
                ccw: Diagonal::Northeast,
                cw: Diagonal::Southwest,
            },
            Diagonal::Southwest => RotationData {
                ccw: Diagonal::Southeast,
                cw: Diagonal::Northwest,
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

    /// Returns the two cardinal directions adjacent to `self`.
    pub const fn cardinal_neighbors(self) -> (Cardinal, Cardinal) {
        match self {
            Diagonal::Northeast => (Cardinal::North, Cardinal::East),
            Diagonal::Northwest => (Cardinal::North, Cardinal::West),
            Diagonal::Southeast => (Cardinal::South, Cardinal::East),
            Diagonal::Southwest => (Cardinal::South, Cardinal::West),
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
    /// Get the data corresponding to the given direction.
    pub fn get(&self, dir: Cardinal) -> &T {
        match dir {
            Cardinal::North => &self.n,
            Cardinal::South => &self.s,
            Cardinal::East => &self.e,
            Cardinal::West => &self.w,
        }
    }

    /// Mutably get the data corresponding to the given direction.
    pub fn get_mut(&mut self, dir: Cardinal) -> &mut T {
        match dir {
            Cardinal::North => &mut self.n,
            Cardinal::South => &mut self.s,
            Cardinal::East => &mut self.e,
            Cardinal::West => &mut self.w,
        }
    }

    /// Get N/S/E/W data as a tuple.
    pub fn as_tuple(&self) -> (&T, &T, &T, &T) {
        (&self.n, &self.s, &self.e, &self.w)
    }

    /// Unpack N/S/E/W data into a tuple (by value).
    pub fn to_tuple(self) -> (T, T, T, T) {
        (self.n, self.s, self.e, self.w)
    }

    /// Returns true iff `predicate` returns true for any direction's data.
    pub fn any<P>(&self, predicate: P) -> bool
    where
        P: Fn(&T) -> bool,
    {
        predicate(&self.n) || predicate(&self.s) || predicate(&self.e) || predicate(&self.w)
    }

    /// Returns true iff `predicate` returns true for all directions' data.
    pub fn all<P>(&self, predicate: P) -> bool
    where
        P: Fn(&T) -> bool,
    {
        predicate(&self.n) && predicate(&self.s) && predicate(&self.e) && predicate(&self.w)
    }
}

impl<T> CardinalData<T>
where
    T: Default + Copy + PartialEq,
{
    /// Count the number of directions that have non-default values.
    pub fn count(self) -> u8 {
        (self.n != T::default()) as u8
            + (self.s != T::default()) as u8
            + (self.e != T::default()) as u8
            + (self.w != T::default()) as u8
    }

    /// Get the list of directions that have a non-default value.
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
    /// Set all directions to the same value.
    pub fn set_all(&mut self, value: T) {
        self.n = value;
        self.s = value;
        self.e = value;
        self.w = value;
    }
}

/// Holds data for the 4 intercardinal/diagonal directions.
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
    /// Returns a reference to the data corresponding to the given direction.
    pub fn get(&self, dir: Diagonal) -> &T {
        match dir {
            Diagonal::Northeast => &self.ne,
            Diagonal::Northwest => &self.nw,
            Diagonal::Southeast => &self.se,
            Diagonal::Southwest => &self.sw,
        }
    }
    /// Returns a mutable reference to the data corresponding to the given direction.
    pub fn get_mut(&mut self, dir: Diagonal) -> &mut T {
        match dir {
            Diagonal::Northeast => &mut self.ne,
            Diagonal::Northwest => &mut self.nw,
            Diagonal::Southeast => &mut self.se,
            Diagonal::Southwest => &mut self.sw,
        }
    }
    /// Returns the data for the two diagonal directions that are adjacent to the given cardinal
    /// direction (e.g., `side(Cardinal::North)` will return the data for Northeast and
    /// Northwest).
    pub fn side(&self, dir: Cardinal) -> RotationData<&T> {
        match dir {
            Cardinal::North => RotationData {
                ccw: &self.nw,
                cw: &self.ne,
            },
            Cardinal::South => RotationData {
                ccw: &self.se,
                cw: &self.sw,
            },
            Cardinal::East => RotationData {
                ccw: &self.ne,
                cw: &self.se,
            },
            Cardinal::West => RotationData {
                ccw: &self.sw,
                cw: &self.nw,
            },
        }
    }

    /// Same as `side()`, but returns mutable references.
    pub fn side_mut(&mut self, dir: Cardinal) -> RotationData<&mut T> {
        match dir {
            Cardinal::North => RotationData {
                ccw: &mut self.nw,
                cw: &mut self.ne,
            },
            Cardinal::South => RotationData {
                ccw: &mut self.se,
                cw: &mut self.sw,
            },
            Cardinal::East => RotationData {
                ccw: &mut self.ne,
                cw: &mut self.se,
            },
            Cardinal::West => RotationData {
                ccw: &mut self.sw,
                cw: &mut self.nw,
            },
        }
    }

    /// Returns true iff `predicate` returns true for any direction's data.
    pub fn any<P>(&self, predicate: P) -> bool
    where
        P: Fn(&T) -> bool,
    {
        predicate(&self.ne) || predicate(&self.se) || predicate(&self.nw) || predicate(&self.sw)
    }

    /// Returns true iff `predicate` returns true for all directions' data.
    pub fn all<P>(&self, predicate: P) -> bool
    where
        P: Fn(&T) -> bool,
    {
        predicate(&self.ne) && predicate(&self.se) && predicate(&self.nw) && predicate(&self.sw)
    }
}

impl<T> DiagonalData<T>
where
    T: Copy,
{
    /// Set all directions to the same data.
    pub fn set_all(&mut self, data: T) {
        self.ne = data;
        self.nw = data;
        self.se = data;
        self.sw = data;
    }
}

/// Holds data for the two handed turn directions
#[derive(Default, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct RotationData<T> {
    /// Data for the clockwise/right side
    pub cw: T,
    /// Data for the counterclockwise/left side
    pub ccw: T,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cardinal_direction_rules() {
        for dir in [N, S, E, W] {
            assert_ne!(dir, dir.flip());
            assert_ne!(dir, dir.turn(Rotation::Ccw));
            assert_ne!(dir, dir.turn(Rotation::Cw));

            assert_eq!(dir, dir.flip().flip());

            assert_eq!(dir, dir.turn(Rotation::Ccw).turn(Rotation::Cw));
            assert_eq!(dir, dir.turn(Rotation::Cw).turn(Rotation::Ccw));
            assert_eq!(
                dir,
                dir.turn(Rotation::Cw)
                    .turn(Rotation::Cw)
                    .turn(Rotation::Cw)
                    .turn(Rotation::Cw)
            );
            assert_eq!(
                dir,
                dir.turn(Rotation::Ccw)
                    .turn(Rotation::Ccw)
                    .turn(Rotation::Ccw)
                    .turn(Rotation::Ccw)
            );

            assert_eq!(dir.turn(Rotation::Cw).flip(), dir.turn(Rotation::Ccw));
            assert_eq!(dir.turn(Rotation::Ccw).flip(), dir.turn(Rotation::Cw));

            assert_eq!(dir.turn(Rotation::Cw).turn(Rotation::Cw), dir.flip());
            assert_eq!(dir.turn(Rotation::Ccw).turn(Rotation::Ccw), dir.flip());

            assert_eq!(dir.perp().cw, dir.turn(Rotation::Cw));
            assert_eq!(dir.perp().ccw, dir.turn(Rotation::Ccw));
        }
    }

    #[test]
    fn test_diagonal_direction_rules() {
        for dir in [NW, NE, SW, SE] {
            assert_ne!(dir, dir.flip());
            assert_ne!(dir, dir.turn(Rotation::Ccw));
            assert_ne!(dir, dir.turn(Rotation::Cw));

            assert_eq!(dir, dir.flip().flip());

            assert_eq!(dir, dir.turn(Rotation::Ccw).turn(Rotation::Cw));
            assert_eq!(dir, dir.turn(Rotation::Cw).turn(Rotation::Ccw));
            assert_eq!(
                dir,
                dir.turn(Rotation::Cw)
                    .turn(Rotation::Cw)
                    .turn(Rotation::Cw)
                    .turn(Rotation::Cw)
            );
            assert_eq!(
                dir,
                dir.turn(Rotation::Ccw)
                    .turn(Rotation::Ccw)
                    .turn(Rotation::Ccw)
                    .turn(Rotation::Ccw)
            );

            assert_eq!(dir.turn(Rotation::Cw).flip(), dir.turn(Rotation::Ccw));
            assert_eq!(dir.turn(Rotation::Ccw).flip(), dir.turn(Rotation::Cw));

            assert_eq!(dir.turn(Rotation::Cw).turn(Rotation::Cw), dir.flip());
            assert_eq!(dir.turn(Rotation::Ccw).turn(Rotation::Ccw), dir.flip());

            assert_eq!(dir.perp().cw, dir.turn(Rotation::Cw));
            assert_eq!(dir.perp().ccw, dir.turn(Rotation::Ccw));
        }
    }
}
