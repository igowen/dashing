use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// Creates a `Point` from x, y coordinates.
///
/// # Example
/// `let p = point![10, 20];`
#[macro_export]
macro_rules! point {
    ($x:expr, $y:expr) => {
        $crate::Point { x: $x, y: $y }
    };
}

/// Creates a `Vector` from dx, dy components.
///
/// # Example
/// `let v = vector![1, -1];`
#[macro_export]
macro_rules! vector {
    ($dx:expr, $dy:expr) => {
        $crate::Vector { dx: $dx, dy: $dy }
    };
}

/// Creates a `Size` from width, height components.
///
/// # Example
/// `let s = size![12, 12];`
#[macro_export]
macro_rules! size {
    ($w:expr, $h:expr) => {
        $crate::Size { w: $w, h: $h }
    };
}

/// Creates a `Rect` from components.
///
/// # Examples
/// `let r1 = rect![10, 20, 30, 40];`
/// `let r2 = rect![point![10, 20], size![30, 40]];`
#[macro_export]
macro_rules! rect {
    ($x:expr, $y:expr, $w:expr, $h:expr) => {
        $crate::Rect::new($x, $y, $w, $h)
    };
    ($origin:expr, $size:expr) => {
        $crate::Rect {
            origin: $origin,
            size: $size,
        }
    };
}

/// A 2D point in a discrete grid.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash)]
pub struct Point {
    /// The x-coordinate.
    pub x: i32,
    /// The y-coordinate.
    pub y: i32,
}

impl Point {
    /// Creates a new point.
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub const fn zero() -> Self {
        Self { x: 0, y: 0 }
    }

    /// Returns this point iterpreted as a Vector (displacement from the origin).
    pub const fn as_displacement(self) -> Vector {
        vector![self.x, self.y]
    }

    /// const impl of add since the Add trait isn't const
    const fn add(self, rhs: Vector) -> Point {
        point![self.x + rhs.dx, self.y + rhs.dy]
    }

    /// const impl of sub since the Sub trait isn't const
    const fn sub(self, rhs: Vector) -> Point {
        point![self.x - rhs.dx, self.y - rhs.dy]
    }

    /// Returns two Points `px` and `py` that split the x and y components of `self`.
    pub const fn decompose(self) -> (Self, Self) {
        (Self { x: self.x, y: 0 }, Self { x: 0, y: self.y })
    }
}

/// A 2D vector representing displacement or direction in a discrete grid.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash)]
pub struct Vector {
    /// The displacement along the x-axis.
    pub dx: i32,
    /// The displacement along the y-axis.
    pub dy: i32,
}

impl Vector {
    /// Creates a new vector.
    pub const fn new(dx: i32, dy: i32) -> Self {
        Self { dx, dy }
    }

    pub const fn zero() -> Self {
        Self { dx: 0, dy: 0 }
    }

    /// Returns two vectors `vx` and `vy` that split the x and y components of `self` (and
    /// therefore `self == vx + vy`)
    pub const fn decompose(self) -> (Self, Self) {
        (Self { dx: self.dx, dy: 0 }, Self { dx: 0, dy: self.dy })
    }
}

/// A 2D size with a width and height. Can be negative.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash)]
pub struct Size {
    /// The width component.
    pub w: i32,
    /// The height component.
    pub h: i32,
}

impl Size {
    /// The largest possible size.
    pub const MAX: Self = Self {
        w: i32::MAX,
        h: i32::MAX,
    };

    /// Creates a new size.
    pub const fn new(w: i32, h: i32) -> Self {
        Self { w, h }
    }

    /// Returns a Size with width and height set to 0.
    pub const fn zero() -> Self {
        Self { w: 0, h: 0 }
    }

    /// Calculates the area.
    pub const fn area(self) -> u64 {
        self.w.unsigned_abs() as u64 * self.h.unsigned_abs() as u64
    }

    pub const fn as_displacement(self) -> Vector {
        Vector {
            dx: self.w - 1,
            dy: self.h - 1,
        }
    }

    pub const fn is_empty(self) -> bool {
        self.w == 0 || self.h == 0
    }
}

/// A rectangle defined by an origin point and a size.
///
/// Geometric calculations are inclusive and robust to negative sizes.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash)]
pub struct Rect {
    /// The origin point of the rectangle (usually top-left).
    pub origin: Point,
    /// The size of the rectangle.
    pub size: Size,
}

// TODO: remove min/max after std::cmp::Ord is const.
const fn min(a: i32, b: i32) -> i32 {
    if a < b { a } else { b }
}

const fn max(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

impl Rect {
    /// Maximum size Rect, with the origin at (0, 0).
    pub const MAX: Self = Self {
        origin: point![0, 0],
        size: Size::MAX,
    };

    /// Creates a new rectangle from position and size.
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {
            origin: point![x, y],
            size: size![w, h],
        }
    }

    /// Creates the minimum bounding rectangle containing two points.
    /// The resulting rectangle will always have a non-negative size.
    pub const fn from_points(p1: Point, p2: Point) -> Self {
        let x1 = min(p1.x, p2.x);
        let y1 = min(p1.y, p2.y);
        let x2 = max(p1.x, p2.x);
        let y2 = max(p1.y, p2.y);
        Self::new(x1, y1, x2 - x1 + 1, y2 - y1 + 1)
    }

    pub const fn as_points(self) -> (Point, Point) {
        (
            Point {
                x: self.left(),
                y: self.top(),
            },
            Point {
                x: self.right(),
                y: self.bottom(),
            },
        )
    }

    /// Returns a normalized version of the rectangle, with a non-negative size.
    pub const fn normalize(self) -> Self {
        let mut x = self.origin.x;
        let mut y = self.origin.y;
        let mut w = self.size.w;
        let mut h = self.size.h;

        if w < 0 {
            x += w + 1;
            w = -w;
        }
        if h < 0 {
            y += h + 1;
            h = -h;
        }
        Self::new(x, y, w, h)
    }

    /// The minimum x-coordinate (inclusive).
    pub const fn left(self) -> i32 {
        if self.size.w >= 0 {
            self.origin.x
        } else {
            self.origin.x + self.size.w + 1
        }
    }

    /// The maximum x-coordinate (inclusive).
    pub const fn right(self) -> i32 {
        if self.size.w > 0 {
            self.origin.x + self.size.w - 1
        } else {
            self.origin.x
        }
    }

    /// The minimum y-coordinate (inclusive).
    pub const fn top(self) -> i32 {
        if self.size.h >= 0 {
            self.origin.y
        } else {
            self.origin.y + self.size.h + 1
        }
    }

    /// The maximum y-coordinate (inclusive).
    pub const fn bottom(self) -> i32 {
        if self.size.h > 0 {
            self.origin.y + self.size.h - 1
        } else {
            self.origin.y
        }
    }

    /// The center point of the rectangle, truncating division.
    pub const fn center(self) -> Point {
        Point::new(
            self.left() + (self.right() - self.left()) / 2,
            self.top() + (self.bottom() - self.top()) / 2,
        )
    }

    /// Returns the area covered by this rectangle.
    pub const fn area(self) -> u64 {
        self.size.area()
    }

    /// Returns true iff the rectangle's size is 0 in either dimension.
    pub const fn is_empty(self) -> bool {
        self.size.is_empty()
    }

    /// Checks if a point is contained within the rectangle (inclusive).
    pub const fn contains(self, point: Point) -> bool {
        // Empty rects contain nothing.
        if self.is_empty() {
            return false;
        }
        point.x >= self.left()
            && point.x <= self.right()
            && point.y >= self.top()
            && point.y <= self.bottom()
    }

    /// Checks if another rectangle is fully contained within this one (inclusive).
    pub const fn contains_rect(self, other: Rect) -> bool {
        // Empty rects contain nothing.
        if self.is_empty() {
            return false;
        }
        // Empty rects are vacuously contained in non-empty rects.
        if other.is_empty() {
            return true;
        }
        self.left() <= other.left()
            && self.right() >= other.right()
            && self.top() <= other.top()
            && self.bottom() >= other.bottom()
    }

    /// Checks if this rectangle overlaps with another (inclusive of edges).
    pub const fn intersects(self, other: Rect) -> bool {
        self.left() <= other.right()
            && self.right() >= other.left()
            && self.top() <= other.bottom()
            && self.bottom() >= other.top()
    }

    /// Returns the rect capturing the overlap between `self` and `other` (if it exists).
    pub const fn intersection(self, other: Rect) -> Option<Rect> {
        let min_x = max(self.left(), other.left());
        let min_y = max(self.top(), other.top());
        let max_x = min(self.right(), other.right());
        let max_y = min(self.bottom(), other.bottom());
        if max_x >= min_x && max_y >= min_y {
            Some(Rect::from_points(
                point![min_x, min_y],
                point![max_x, max_y],
            ))
        } else {
            None
        }
    }
}

// --- From/Into Conversions ---

impl From<(i32, i32)> for Point {
    fn from((x, y): (i32, i32)) -> Self {
        Self { x, y }
    }
}

impl From<(i32, i32)> for Vector {
    fn from((dx, dy): (i32, i32)) -> Self {
        Self { dx, dy }
    }
}

impl From<(i32, i32)> for Size {
    fn from((w, h): (i32, i32)) -> Self {
        Self { w, h }
    }
}

impl From<Point> for (i32, i32) {
    fn from(val: Point) -> Self {
        (val.x, val.y)
    }
}

impl From<Size> for (i32, i32) {
    fn from(val: Size) -> Self {
        (val.w, val.h)
    }
}

impl From<Vector> for (i32, i32) {
    fn from(val: Vector) -> Self {
        (val.dx, val.dy)
    }
}

// --- Operator Overloads ---

// Point + Vector = Point
impl Add<Vector> for Point {
    type Output = Point;
    fn add(self, rhs: Vector) -> Self::Output {
        self.add(rhs)
    }
}

impl AddAssign<Vector> for Point {
    fn add_assign(&mut self, rhs: Vector) {
        self.x += rhs.dx;
        self.y += rhs.dy;
    }
}

// Point - Vector = Point
impl Sub<Vector> for Point {
    type Output = Point;
    fn sub(self, rhs: Vector) -> Self::Output {
        self.sub(rhs)
    }
}

impl SubAssign<Vector> for Point {
    fn sub_assign(&mut self, rhs: Vector) {
        self.x -= rhs.dx;
        self.y -= rhs.dy;
    }
}

// Point - Point = Vector
impl Sub<Point> for Point {
    type Output = Vector;
    fn sub(self, rhs: Point) -> Self::Output {
        vector![self.x - rhs.x, self.y - rhs.y]
    }
}

// Vector + Vector = Vector
impl Add for Vector {
    type Output = Vector;
    fn add(self, rhs: Self) -> Self::Output {
        vector![self.dx + rhs.dx, self.dy + rhs.dy]
    }
}

impl AddAssign for Vector {
    fn add_assign(&mut self, rhs: Self) {
        self.dx += rhs.dx;
        self.dy += rhs.dy;
    }
}

// Vector - Vector = Vector
impl Sub for Vector {
    type Output = Vector;
    fn sub(self, rhs: Self) -> Self::Output {
        vector![self.dx - rhs.dx, self.dy - rhs.dy]
    }
}

// -Vector = Vector
impl Neg for Vector {
    type Output = Vector;
    fn neg(self) -> Self::Output {
        vector![-self.dx, -self.dy]
    }
}

// Vector * scalar
impl Mul<i32> for Vector {
    type Output = Vector;
    fn mul(self, rhs: i32) -> Self::Output {
        vector![self.dx * rhs, self.dy * rhs]
    }
}

impl MulAssign<i32> for Vector {
    fn mul_assign(&mut self, rhs: i32) {
        self.dx *= rhs;
        self.dy *= rhs;
    }
}

// Vector / scalar
impl Div<i32> for Vector {
    type Output = Vector;
    fn div(self, rhs: i32) -> Self::Output {
        vector![self.dx / rhs, self.dy / rhs]
    }
}

impl DivAssign<i32> for Vector {
    fn div_assign(&mut self, rhs: i32) {
        self.dx /= rhs;
        self.dy /= rhs;
    }
}

impl Add<Vector> for Rect {
    type Output = Rect;
    fn add(mut self, rhs: Vector) -> Self::Output {
        self.origin += rhs;
        self
    }
}

impl Sub<Vector> for Rect {
    type Output = Rect;
    fn sub(mut self, rhs: Vector) -> Self::Output {
        self.origin -= rhs;
        self
    }
}

// Size +/- Size
impl Add<Size> for Size {
    type Output = Size;
    fn add(self, rhs: Size) -> Self::Output {
        size![self.w + rhs.w, self.h + rhs.h]
    }
}

impl Sub<Size> for Size {
    type Output = Size;
    fn sub(self, rhs: Size) -> Self::Output {
        size![self.w - rhs.w, self.h - rhs.h]
    }
}

impl AddAssign for Size {
    fn add_assign(&mut self, rhs: Self) {
        self.w += rhs.w;
        self.h += rhs.h;
    }
}

impl SubAssign for Size {
    fn sub_assign(&mut self, rhs: Self) {
        self.w -= rhs.w;
        self.h -= rhs.h;
    }
}

// Size * scalar
impl Mul<i32> for Size {
    type Output = Size;
    fn mul(self, rhs: i32) -> Self::Output {
        size![self.w * rhs, self.h * rhs]
    }
}

impl MulAssign<i32> for Size {
    fn mul_assign(&mut self, rhs: i32) {
        self.w *= rhs;
        self.h *= rhs;
    }
}

impl Div<i32> for Size {
    type Output = Size;
    fn div(self, rhs: i32) -> Self::Output {
        size![self.w / rhs, self.h / rhs]
    }
}

impl DivAssign<i32> for Size {
    fn div_assign(&mut self, rhs: i32) {
        self.w /= rhs;
        self.h /= rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_ops() {
        let p1 = point![10, 20];
        let p2 = point![12, 25];
        let v = vector![2, 5];
        assert_eq!(p1 + v, p2);
        assert_eq!(p2 - v, p1);
        assert_eq!(p2 - p1, v);
    }

    #[test]
    fn rect_basics() {
        let r = rect![1, 1, 2, 3];
        assert_eq!(r.left(), 1);
        assert_eq!(r.top(), 1);
        assert_eq!(r.right(), 2);
        assert_eq!(r.bottom(), 3);

        let r = rect![2, 3, -2, -3];
        assert_eq!(r.left(), 1);
        assert_eq!(r.top(), 1);
        assert_eq!(r.right(), 2);
        assert_eq!(r.bottom(), 3);
    }

    #[test]
    fn rect_inclusive_geometry() {
        let r = rect![10, 20, 5, 10]; // x: 10-14, y: 20-29
        assert_eq!(r.left(), 10);
        assert_eq!(r.right(), 14);
        assert_eq!(r.top(), 20);
        assert_eq!(r.bottom(), 29);
        assert_eq!(r.center(), point![12, 24]);
    }

    #[test]
    fn rect_inclusive_contains() {
        let r = rect![0, 0, 10, 10]; // x in [0, 9], y in [0, 9]
        assert!(r.contains(point![0, 0]));
        assert!(r.contains(point![9, 9]));
        assert!(r.contains(point![5, 5]));
        assert!(!r.contains(point![10, 9]));
        assert!(!r.contains(point![9, 10]));
        assert!(!r.contains(point![-1, 5]));
    }

    #[test]
    fn rect_inclusive_intersects() {
        let r1 = rect![0, 0, 10, 10];
        let r2 = rect![5, 5, 10, 10]; // Overlapping
        let r3 = rect![9, 0, 5, 5]; // Touching edge
        let r4 = rect![10, 0, 5, 5]; // Separate

        assert!(r1.intersects(r2));
        assert!(r2.intersects(r1));
        assert!(r1.intersects(r3)); // Touching is intersecting
        assert!(r3.intersects(r1));
        assert!(!r1.intersects(r4));
        assert!(!r4.intersects(r1));
    }

    #[test]
    fn rect_contains_rect() {
        let r1 = rect![0, 0, 20, 20];
        let r2 = rect![5, 5, 10, 10]; // Fully inside
        let r3 = rect![15, 5, 10, 10]; // Partially outside
        let r4 = rect![0, 0, 20, 20]; // Equal
        let r5 = rect![-5, -5, 30, 30]; // Superset

        assert!(r1.contains_rect(r2));
        assert!(!r1.contains_rect(r3));
        assert!(r1.contains_rect(r4));
        assert!(!r1.contains_rect(r5));
        assert!(r5.contains_rect(r1));
    }

    #[test]
    fn rect_negative_size_geometry() {
        // x from 5 to 9 (inclusive), y from 10 to 19 (inclusive)
        let r = rect![9, 19, -5, -10];
        assert_eq!(r.left(), 5);
        assert_eq!(r.right(), 9);
        assert_eq!(r.top(), 10);
        assert_eq!(r.bottom(), 19);
        assert_eq!(r.center(), point![7, 14]);
    }

    #[test]
    fn rect_normalize() {
        let r1 = rect![9, 19, -5, -10];
        let r_norm1 = rect![5, 10, 5, 10];
        assert_eq!(r1.normalize(), r_norm1);

        let r2 = rect![0, 0, 10, 10];
        assert_eq!(r2.normalize(), r2);
    }

    #[test]
    fn rect_from_points() {
        let p1 = point![10, 20];
        let p2 = point![0, 5];
        let r = Rect::from_points(p1, p2);
        // x: 0-10 (11 wide), y: 5-20 (16 high)
        assert_eq!(r, rect![0, 5, 11, 16]);
    }

    #[test]
    fn macros_work() {
        assert_eq!(point![1, 2], point![1, 2]);
        assert_eq!(vector![3, 4], vector![3, 4]);
        assert_eq!(size![5, 6], size![5, 6]);

        let r1 = rect![10, 20, 30, 40];
        let r2 = rect![point![10, 20], size![30, 40]];

        assert_eq!(r1, rect![10, 20, 30, 40]);
        assert_eq!(r2, rect![10, 20, 30, 40]);
        assert_eq!(r1, r2);
    }

    use proptest::prelude::*;

    // Rules for creating arbitrary instances of the above geometry types. We limit the range from
    // -1^16 to 1^16 just so we don't hit any overflow cases -- this is already way larger than we
    // would expect to see for geometry on the grid.
    const RANGE: std::ops::Range<i32> = (-(1 << 16))..(1 << 16);
    prop_compose! {
        fn arb_point()(x in RANGE, y in RANGE) -> Point {
            Point { x, y }
        }
    }

    prop_compose! {
        fn arb_vector()(dx in RANGE, dy in RANGE) -> Vector {
            Vector { dx, dy }
        }
    }

    prop_compose! {
        fn arb_size()(w in RANGE, h in RANGE) -> Size {
            Size { w, h }
        }
    }

    prop_compose! {
        fn arb_nonempty_size()(
            w in RANGE.prop_filter("Width must be nonzero", |v| *v != 0),
            h in RANGE.prop_filter("Height must be nonzero", |v| *v != 0)) -> Size {
            Size { w, h }
        }
    }

    prop_compose! {
        fn arb_empty_size()(
            dim in RANGE.prop_filter("Dimension must be nonzero", |v| *v != 0),
            empty_width in proptest::bool::ANY) -> Size {
            if empty_width {
                Size { w: 0, h: dim }
            } else {
                Size { w: dim, h: 0 }
            }
        }
    }

    prop_compose! {
        fn arb_rect()(origin in arb_point(), size in arb_size()) -> Rect {
            Rect { origin, size }
        }
    }

    prop_compose! {
        fn arb_nonempty_rect()(origin in arb_point(), size in arb_nonempty_size()) -> Rect {
            Rect { origin, size }
        }
    }

    prop_compose! {
        fn arb_empty_rect()(origin in arb_point(), size in arb_empty_size()) -> Rect {
            Rect { origin, size }
        }
    }

    prop_compose! {
        fn arb_overlapping_rects()(
            (r0, r1) in (arb_nonempty_rect(), arb_nonempty_rect()).prop_filter(
                "Rects must overlap",
                |(a,b)| a.intersects(*b))
        ) -> (Rect, Rect) {
            (r0, r1)
        }
    }

    proptest! {
        #[test]
        fn test_vector_round_trip(p in arb_point(), v in arb_vector()) {
            prop_assert_eq!((p + v) - v, p);
        }

        #[test]
        fn test_point_displacement_identity(p0 in arb_point(), p1 in arb_point()) {
            prop_assert_eq!(p0 + (p1 - p0), p1);
        }

        #[test]
        fn test_vector_additive_inverse(v in arb_vector()) {
            prop_assert_eq!(v + (-v), Vector::zero());
            prop_assert_eq!(v - v, Vector::zero());
        }

        #[test]
        fn test_vector_commutativity(v0 in arb_vector(), v1 in arb_vector()) {
            prop_assert_eq!(v0 + v1, v1 + v0);
        }

        #[test]
        fn test_normalize_rect_idempotency(r in arb_rect()) {
            prop_assert_eq!(r.normalize().normalize(), r.normalize());
        }

        #[test]
        fn test_normalize_rect_yields_positive_size(r in arb_rect()) {
            let norm = r.normalize();
            prop_assert!(norm.size.w >= 0);
            prop_assert!(norm.size.h >= 0);
        }

        #[test]
        fn test_normalize_rect_geometric_equivalence(r in arb_rect()) {
            let norm = r.normalize();
            prop_assert_eq!(norm.left(), r.left());
            prop_assert_eq!(norm.right(), r.right());
            prop_assert_eq!(norm.top(), r.top());
            prop_assert_eq!(norm.bottom(), r.bottom());
        }

        #[test]
        fn test_normalized_rect_contains_same_points(r in arb_rect(), p in arb_point()) {
            let norm = r.normalize();
            prop_assert_eq!(r.contains(p), norm.contains(p));
        }

        #[test]
        fn test_rect_from_points(p0 in arb_point(), p1 in arb_point()) {
            let r = Rect::from_points(p0, p1);
            // Resulting rect should contain both points.
            prop_assert!(r.contains(p0));
            prop_assert!(r.contains(p1));
            // Also, the order of the points shouldn't matter -- we should get the same rect either
            // way.
            prop_assert_eq!(r, Rect::from_points(p1, p0));
        }

        #[test]
        fn test_intersection_is_commutative(r0 in arb_rect(), r1 in arb_rect()) {
            prop_assert_eq!(r0.intersects(r1), r1.intersects(r0));
        }

        #[test]
        fn test_rect_intersection_is_reflexive(r in arb_rect()) {
            prop_assert!(r.intersects(r));
        }

        #[test]
        fn test_rect_containment_implies_intersection(r0 in arb_rect(), r1 in arb_rect()) {
            // r0.contains_rect(r1) -> r0.intersects(r1)
            prop_assert!(!r0.contains_rect(r1) || r0.intersects(r1));
        }

        #[test]
        fn test_rect_containment_is_transitive(
            r0 in arb_nonempty_rect(),
            r1 in arb_nonempty_rect(),
            r2 in arb_nonempty_rect()
        ) {
            // r0.contains_rect(r1) && r1.contains_rect(r2) -> r0.contains_rect(r2)
            prop_assert!(!(r0.contains_rect(r1) && r1.contains_rect(r2)) || r0.contains_rect(r2));
        }

        #[test]
        fn test_rect_contains_translation_invariance_point_vector(
            r in arb_rect(),
            p in arb_point(),
            v in arb_vector()
        ) {
            //  For any Rect r, Point p, and Vector v, the result of r.contains(p) should be
            //  identical to rect![r.origin + v, r.size].contains(p + v). In other words, if you
            //  move the rect and the point together, the containment relationship should be
            //  preserved.
            prop_assert_eq!(r.contains(p), (r + v).contains(p + v));
        }

        #[test]
        fn test_rect_contains_translation_invariance_other_rect(
            r0 in arb_rect(),
            r1 in arb_rect(),
            v in arb_vector()
        ) {
            prop_assert_eq!(r0.intersects(r1), (r0 + v).intersects(r1 + v));
        }

        #[test]
        fn test_empty_rect_contains_no_points(
            r in arb_empty_rect(),
            p in arb_point()
        ) {
            // R has a zero dimension -> R contains no points
            prop_assert!(!r.contains(p));
        }

        #[test]
        fn test_rect_contains_empty_rect(
            r in arb_nonempty_rect(),
            p in arb_point(),
            s in arb_empty_size()
        ) {
            // Any non-empty rect contains all empty rects.
            prop_assert!(r.contains_rect(rect![p, s]));
        }

        #[test]
        fn test_empty_rect_contains_no_rect(
            r in arb_rect(),
            p in arb_point(),
            s in arb_empty_size()
        ) {
            // All empty rects contain no other rects.
            prop_assert!(!rect![p, s].contains_rect(r));
        }

        #[test]
        fn test_intersection_intersects_consistency(
            r0 in arb_rect(),
            r1 in arb_rect(),
        ) {
            prop_assert_eq!(r0.intersection(r1).is_some(), r0.intersects(r1));
        }

        // Sanity check so I feel comfortable prop_assume!ing the intersection in the following
        // tests.
        #[test]
        fn test_arb_overlapping(
            (r0, r1) in arb_overlapping_rects()
        ) {
            prop_assert!(r0.intersection(r1).is_some());
        }

        #[test]
        fn test_intersection_containment(
            (r0, r1) in arb_overlapping_rects()
        ) {
            let intersection = r0.intersection(r1);
            prop_assume!(intersection.is_some());
            let intersection = intersection.unwrap();
            prop_assert!(r0.contains_rect(intersection));
            prop_assert!(r1.contains_rect(intersection));
        }

        #[test]
        fn test_intersection_completeness(
            (r0, r1) in arb_overlapping_rects(),
            p in arb_point(),
        ) {
            let intersection = r0.intersection(r1);
            prop_assume!(intersection.is_some());
            let intersection = intersection.unwrap();

            prop_assert_eq!(intersection.contains(p), r0.contains(p) && r1.contains(p));
        }

        #[test]
        fn test_from_points_symmetry(
            p0 in arb_point(),
            p1 in arb_point(),
        ) {
            let r0 = Rect::from_points(p0, p1);
            let r1 = Rect::from_points(p1, p0);
            prop_assert_eq!(r0.top(), r1.top());
            prop_assert_eq!(r0.bottom(), r1.bottom());
            prop_assert_eq!(r0.left(), r1.left());
            prop_assert_eq!(r0.right(), r1.right());
        }

        #[test]
        fn test_normalize_preserves_area(r in arb_rect()) {
            prop_assert_eq!(r.normalize().size.area(), r.size.area());
        }
    }
}
