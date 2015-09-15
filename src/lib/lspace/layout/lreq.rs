/// Computes the maximum of two numbers
///
/// A faster alternative to f64::max
pub fn fast_max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        return a;
    }
    else {
        return b;
    }
}

/// Computes the maximum of two optional numbers
///
/// If both numbers are present, the maximum is returned.
/// If one number is present, it is returned as is.
/// If both are absent (`None`), then `None` is returned.
pub fn fast_maxopt<T: PartialOrd>(a: Option<T>, b: Option<T>) -> Option<T> {
    match (a, b) {
        (Some(x), Some(y)) => Some(fast_max(x, y)),
        (None, None) => None,
        (x, None) => x,
        (None, y) => y,
    }
}

/// Computes the minimum of two numbers
///
/// A faster alternative to f64::min
pub fn fast_min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        return a;
    }
    else {
        return b;
    }
}

/// Computes the minimum of two optional numbers
///
/// If both numbers are present, the minimum is returned.
/// If one number is present, it is returned as is.
/// If both are absent (`None`), then `None` is returned.
pub fn fast_minopt<T: PartialOrd>(a: Option<T>, b: Option<T>) -> Option<T> {
    match (a, b) {
        (Some(x), Some(y)) => Some(fast_min(x, y)),
        (None, None) => None,
        (x, None) => x,
        (None, y) => y,
    }
}


/// A repesentation of the natural size of an element with an optional reference point
///
/// Three variants:
/// `Empty` - nothing
/// `Size` - a single value for the natural size
/// `Ref` - two values, for space before and after the reference point
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum LNatSize {
    Empty,
    Size{size: f64},
    Ref{before: f64, after: f64},
}


impl LNatSize {
    /// Constructs an empty `LNatSize`
    pub fn new_empty() -> LNatSize {
        return LNatSize::Empty;
    }

    /// Constructs an `LNatSize` representing a natural size
    pub fn new_size(size: f64) -> LNatSize {
        return LNatSize::Size{size: size};
    }

    /// Constructs an `LNatSize` representing a natural size for `before` units of space before the
    /// reference point and `after` units of space after it.
    pub fn new_ref(before: f64, after: f64) -> LNatSize {
        return LNatSize::Ref{before: before, after: after};
    }

    /// Get the amount of space requested by an `LNatSize`
    ///
    /// For a `Ref` variant `LNatSize`, the sum of space before and after the reference point is
    /// returned
    pub fn size(&self) -> f64 {
        match self {
            &LNatSize::Empty => 0.0,
            &LNatSize::Size{size: s} => s,
            &LNatSize::Ref{before: b, after: a} => a + b
        }
    }

    /// Get the amount of space requested before the reference point
    ///
    /// For the a `Size` variant of `LNatSize`, half the size is returned
    pub fn before_ref(&self) -> f64 {
        match self {
            &LNatSize::Empty => 0.0,
            &LNatSize::Size{size: s} => s * 0.5,
            &LNatSize::Ref{before: b, ..} => b
        }
    }

    /// Get the amount of space requested after the reference point
    ///
    /// For the a `Size` variant of `LNatSize`, half the size is returned
    pub fn after_ref(&self) -> f64 {
        match self {
            &LNatSize::Empty => 0.0,
            &LNatSize::Size{size: s} => s * 0.5,
            &LNatSize::Ref{after: a, ..} => a
        }
    }

    /// Get the amount of space requested before the reference point
    ///
    /// For the a `Size` variant of `LNatSize`, half the size is returned
    pub fn before_ref_opt(&self) -> Option<f64> {
        match self {
            &LNatSize::Empty => None,
            &LNatSize::Size{..} => None,
            &LNatSize::Ref{before: b, ..} => Some(b)
        }
    }

    /// Get the amount of space requested after the reference point
    ///
    /// For the a `Size` variant of `LNatSize`, half the size is returned
    pub fn after_ref_opt(&self) -> Option<f64> {
        match self {
            &LNatSize::Empty => None,
            &LNatSize::Size{..} => None,
            &LNatSize::Ref{after: a, ..} => Some(a)
        }
    }

    /// Get the amount of space requested before and after the reference point
    ///
    /// For the `Ref` variant, `Some(pair)` is returned where `pair` is
    /// `(b, a)` where `b` and `a` are the space before and after the reference point respectively.
    /// Otherwise, `None`.
    pub fn before_and_after_ref_opt(&self) -> Option<(f64, f64)> {
        match self {
            &LNatSize::Empty => None,
            &LNatSize::Size{..} => None,
            &LNatSize::Ref{before: b, after: a} => Some((b, a))
        }
    }

    /// Get the fractional position of the reference point
    pub fn fractional_ref_point(&self) -> f64 {
        match self {
            &LNatSize::Ref{before, after} => before / (before + after),
            _ => 0.5,
        }
    }

    /// Determines if an `LNatSize` has a reference point
    ///
    /// Returns `true` if the LNatSize is of the `Ref` variant, `false` otherwise
    pub fn has_ref_point(&self) -> bool {
        match self {
            &LNatSize::Ref{..} => true,
            _ => false
        }
    }

    /// Convert to `Size` variant
    pub fn as_size(&self) -> LNatSize {
        match self {
            &LNatSize::Empty => LNatSize::new_size(0.0),
            &LNatSize::Size{..} => *self,
            &LNatSize::Ref{..} => LNatSize::new_size(self.size())
        }
    }

    /// Convert to `Ref` variant
    pub fn as_ref(&self) -> LNatSize {
        match self {
            &LNatSize::Empty => LNatSize::new_ref(0.0, 0.0),
            &LNatSize::Size{size: s} => LNatSize::new_ref(s * 0.5, s * 0.5),
            &LNatSize::Ref{..} => *self
        }
    }

    /// Add the space requests of two `LNatSize` instances together, with the provided amount of
    /// space between them. The result is a `Size` variant of `LNatSize`.
    ///
    /// E.g. if the `LNatSize` instances are representations of vertical space requirements, the
    /// `add` method computes the space required for stacking them vertically, with `other`
    /// positioned below `self`.
    pub fn add(&self, other: &LNatSize, space_between: f64) -> LNatSize {
        match (self, other) {
            // LNatSize::Empty + LNatSize::Empty -> LNatSize::Empty
            (&LNatSize::Empty, &LNatSize::Empty) => *self,

            // LNatSize::Empty + x -> x as LNatSize::Size
            (&LNatSize::Empty, x @ &LNatSize::Size{..}) => *x,
            (&LNatSize::Empty, x) => LNatSize::Size{size: x.size()},

            // x + LNatSize::Empty -> x as LNatSize::Size
            (x @ &LNatSize::Size{..}, &LNatSize::Empty) => *x,
            (x, &LNatSize::Empty) => LNatSize::Size{size: x.size()},

            // y + y
            (_, _) => LNatSize::Size{size: self.size() + space_between + other.size()}
        }
    }

    /// Add the space requests of two `LNatSize` instances together, with the provided amount of
    /// space between them. The result is a `Ref` variant of `LNatSize`. The
    /// `ref_point_from_second` parameter determines if the reference point of the resulting
    /// `LNatSize` is aligned with the reference point of `self`
    /// (`ref_point_from_second == false`) or `other` (`ref_point_from_second == true`).
    ///
    /// E.g. if the `LNatSize` instances are representations of vertical space requirements, the
    /// `add_ref` method computes the space required for stacking them vertically, with `other`
    /// positioned below `self`.
    pub fn add_ref(&self, other: &LNatSize, space_between: f64,
                   ref_point_from_second: bool) -> LNatSize {
        match (self, other, ref_point_from_second) {
            // LNatSize::Empty + LNatSize::Empty -> LNatSize::Empty
            (&LNatSize::Empty, &LNatSize::Empty, _) => *self,

            // LNatSize::Empty + x -> x as LNatSize::Ref
            (&LNatSize::Empty, x @ &LNatSize::Ref{..}, true) => *x,
            (&LNatSize::Empty, &LNatSize::Size{size: s}, true) =>
                LNatSize::Ref{before: s * 0.5, after: s * 0.5},
            (&LNatSize::Empty, x, false) => {
                let s = x.size();
                LNatSize::Ref{before: 0.0, after: s}
            },

            // x + LNatSize::Empty -> x as LNatSize::Ref
            (x @ &LNatSize::Ref{..}, &LNatSize::Empty, false) => *x,
            (x, &LNatSize::Empty, true) => {
                let s = x.size();
                LNatSize::Ref{before: s, after: 0.0}
            },
            (&LNatSize::Size{size: s}, &LNatSize::Empty, false) =>
                LNatSize::Ref{before: s * 0.5, after: s * 0.5},

            // x + y
            (&LNatSize::Size{size: s}, y, false) =>
                LNatSize::Ref{before: s * 0.5, after: s * 0.5 + space_between + y.size()},
            (&LNatSize::Ref{before: b, after: a}, y, false) =>
                LNatSize::Ref{before: b, after: a + space_between + y.size()},
            (x, &LNatSize::Size{size: s}, true) =>
                LNatSize::Ref{before: x.size() + space_between + s * 0.5, after: s * 0.5},
            (x, &LNatSize::Ref{before: b, after: a}, true) =>
                LNatSize::Ref{before: x.size() + space_between + b, after: a},
        }
    }

    /// Compute the maximum of the space requests of two `LNatSize` instances.
    ///
    /// E.g. if the `LNatSize` instances are representations of vertical space requirements, the
    /// `max` method computes the space required for stacking them horizontally, with `other`
    /// positioned to the right `self`.
    ///
    /// If both are of the `Ref` variant, the returned `LNatSize` instance is the result of
    /// aligning the reference points of `self` and `other` and computing the maximum space before
    /// and after.
    ///
    /// If `self` and `other` are a mix of `Size` and `Ref` variants, the result is a `Ref`
    /// variant. If the `Size` variant was bigger (`x.size > (y.before + y.after)`), then
    /// the space before and after the reference point is expanded to match that of the
    /// `Size` variant.
    pub fn max(&self, other: &LNatSize) -> LNatSize {
        match (self, other) {
            // Empty | empty -> empty
            (&LNatSize::Empty, &LNatSize::Empty) => *self,

            // Empty | x -> x
            (&LNatSize::Empty, x) => *x,

            // x | empty -> x
            (x, &LNatSize::Empty) => *x,

            // x | y where x and y are same type
            (&LNatSize::Size{size: s0}, &LNatSize::Size{size: s1}) =>
                LNatSize::Size{size: fast_max(s0, s1)},
            (&LNatSize::Ref{before: b0, after: a0}, &LNatSize::Ref{before: b1, after: a1}) =>
                LNatSize::Ref{before: fast_max(b0, b1), after: fast_max(a0, a1)},

            // x | y where x and y are different types
            (&LNatSize::Size{size: s}, &LNatSize::Ref{before: b, after: a}) |
                (&LNatSize::Ref{before: b, after: a}, &LNatSize::Size{size: s}) => {
                let rsize = a + b;
                if rsize >= s {
                    LNatSize::Ref{before: b, after: a}
                } else {
                    let diff = (s - rsize) * 0.5;
                    LNatSize::Ref{before: b + diff, after: a + diff}
                }
            }
        }
    }

    /// Cumulatively combine the space requirements of a vector of `LNatSize` instances, along
    /// the axis described by the `LNatSize` instances.
    ///
    /// E.g. if each `LNatSize` instance represents a vertical space requirement, they are
    /// stacked vertically.
    ///
    ///`space_between`units of space are placed between each item. It is notionally equivalent to
    /// repeatedly invoking `add` or `add_ref` in a pair-wise fashion.
    /// If `ref_point_index` is `None`, then a `Size` variant is returned. If it is `Some(index)`,
    /// then a `Ref` variant is returned, with the reference point aligned to that of the item
    /// at index `index`.
    pub fn linear_acc<'a, T: Iterator<Item=&'a LNatSize>>(mut boxes: T, space_between: f64,
                      ref_point_index: Option<usize>) -> LNatSize {
        match boxes.next() {
            // no children
            None => LNatSize::new_empty(),
            // >= 1 children
            Some(ref head) => match ref_point_index {
                // no reference point
                None => boxes.fold(head.as_size(), |acc, x| {acc.add(x, space_between)}),
                // reference point at index `i`, enumerate children and s
                Some(i) => {
                    boxes.enumerate().fold(head.as_ref(),
                        |acc, (j, x)| {acc.add_ref(x, space_between, i==(j+1))})
                },
            }
        }
    }

    /// Cumulatively combine the space requirements of a vector of `LNatSize` instances along the
    /// axis that is perpendicular to that described by the `LNatSize` instances.
    ///
    /// E.g. if each `LNatSize` instance represents a vertical space requirement, they are stacked
    /// horizontally.
    ///
    /// Notionally - but not functionally - equivalent to repeatedly invoking the `max` method
    /// in a pairwise fashion.
    pub fn perpendicular_acc<'a, T: Iterator<Item=&'a LNatSize>>(xs: T) -> LNatSize {
        let mut size = 0.0;
        let mut before = 0.0;
        let mut after = 0.0;
        let mut empty = true;
        let mut need_ref = false;

        for x in xs {
            match x {
                &LNatSize::Empty => {},
                &LNatSize::Size{size: s} => {
                    size = fast_max(size, s);
                    empty = false;
                },
                &LNatSize::Ref{before: b, after: a} => {
                    size = fast_max(size, b + a);
                    before = fast_max(before, b);
                    after = fast_max(after, a);
                    empty = false;
                    need_ref = true;
                }
            }
        }

        if empty {
            return LNatSize::new_empty();
        } else if need_ref {
            let rsize = before + after;
            let diff = (size - rsize) * 0.5;
            if diff > 0.0 {
                before += diff;
                after += diff;
            }
            return LNatSize::new_ref(before, after);
        } else {
            return LNatSize::new_size(size);
        }
    }
}


/// A representation of flexibility added to an associated `LNatSize` space requirement.
///
/// Two variants:
/// `Fixed` - no flexibility
/// `Flex` - the `shrink` field specifies an amount that an associated `LNatSize` can shrink by
/// and the `stretch` field is used to determine the share of additional space that will
/// be acquired
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum LFlex {
    Fixed,
    Flex{shrink: f64, stretch: f32}
}


impl LFlex {
    /// Constructs a `Fixed` variant of `LFlex`
    pub fn new_fixed() -> LFlex {
        LFlex::Fixed
    }

    /// Constructs a `Flex` variant of `LFlex`
    pub fn new_flex(shrink: f64, stretch: f32) -> LFlex {
        LFlex::Flex{shrink: shrink, stretch: stretch}
    }

    /// The the amount of space that a box can shrink by
    pub fn shrink(&self) -> f64 {
        match self {
            &LFlex::Fixed => 0.0,
            &LFlex::Flex{shrink, ..} => shrink
        }
    }

    /// Get the stretchiness of a box
    pub fn stretch(&self) -> f32 {
        match self {
            &LFlex::Fixed => 0.0,
            &LFlex::Flex{stretch, ..} => stretch
        }
    }

    /// Scale
    pub fn scale(&self, scale: f32) -> LFlex {
        match self {
            &LFlex::Fixed => *self,
            &LFlex::Flex{shrink, stretch} => LFlex::Flex{shrink: shrink * scale as f64,
                                                         stretch: stretch * scale},
        }
    }

    /// Private helper: get the minimum space requirement of the given `LNatSize`
    fn min_size(&self, sz: &LNatSize) -> Option<f64> {
        match (self, sz) {
            (_, &LNatSize::Empty) => None,
            (&LFlex::Fixed, x) => Some(x.size()),
            (&LFlex::Flex{shrink: k, ..}, x) => Some(x.size() - k),
        }
    }

    /// Private helper: get the stretch if using the `Flex` variant, `None` otherwise
    fn stretch_opt(&self) -> Option<f32> {
        match self {
            &LFlex::Flex{stretch: r, ..} if r > 0.0 => Some(r),
            _ => None
        }
    }

    /// Add the effects of two `LFlex` instances
    pub fn add(&self, other: &LFlex) -> LFlex {
        match (self, other) {
            (&LFlex::Fixed, &LFlex::Fixed) => *self,
            (&LFlex::Fixed, &LFlex::Flex{..}) => *other,
            (&LFlex::Flex{..}, &LFlex::Fixed) => *self,
            (&LFlex::Flex{shrink: sh0, stretch: st0}, &LFlex::Flex{shrink: sh1, stretch: st1}) =>
                LFlex::Flex{shrink: sh0 + sh1, stretch: st0 + st1}
        }
    }

    /// Cumulatively add the effects of two `LFlex` instances
    pub fn linear_acc<'a, T: Iterator<Item=&'a LFlex>>(mut flexes: T) -> LFlex {
        match flexes.next
        () {
            // no children
            None => LFlex::new_fixed(),
            // >= 1 children
            Some(ref head) => flexes.fold(**head, |acc, x| {acc.add(x)}),
        }
    }
}


/// A representation of an optionally flexible space requirement; combines an `LNatSize` with an
/// `LFlex`.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct LReq {
    size: LNatSize,
    flex: LFlex
}


impl LReq {
    /// Construct and empty `LReq`
    pub fn new_empty() -> LReq {
        return LReq{size: LNatSize::new_empty(), flex: LFlex::new_fixed()};
    }

    /// Construct an `LReq` from a given `LNatSize` and `LFlex`
    pub fn new(size: LNatSize, flex: LFlex) -> LReq {
        return LReq{size: size, flex: flex};
    }

    /// Construct an fixed `LReq` of a given size (no flexibility)
    pub fn new_fixed_size(size: f64) -> LReq {
        return LReq{size: LNatSize::new_size(size),
            flex: LFlex::new_fixed()};
    }

    /// Construct a flexible `LReq` of a given natural siz
    pub fn new_flex_size(size: f64, shrink: f64, stretch: f32) -> LReq {
        return LReq{size: LNatSize::new_size(size),
            flex: LFlex::new_flex(shrink, stretch)};
    }

    /// Construct a flexible `LReq` of a given natural size, with shrink specified as a minimum
    /// size
    pub fn new_flex_size_min(size: f64, min_size: f64, stretch: f32) -> LReq {
        return LReq{size: LNatSize::new_size(size),
            flex: LFlex::new_flex(size - min_size, stretch)};
    }

    /// Construct a fixed size `LReq` with a reference point
    pub fn new_fixed_ref(before_ref: f64, after_ref: f64) -> LReq {
        return LReq{size: LNatSize::new_ref(before_ref, after_ref),
            flex: LFlex::new_fixed()};
    }

    /// Construct a flexible `LReq` with a reference point
    pub fn new_flex_ref(before_ref: f64, after_ref: f64, shrink: f64, stretch: f32) -> LReq {
        return LReq{size: LNatSize::new_ref(before_ref, after_ref),
            flex: LFlex::new_flex(shrink, stretch)};
    }

    /// Construct a flexible `LReq` with a reference point, with shrink specified as a minimum size
    pub fn new_flex_ref_min(before_ref: f64, after_ref: f64, min_size: f64, stretch: f32) -> LReq {
        return LReq{size: LNatSize::new_ref(before_ref, after_ref),
            flex: LFlex::new_flex(before_ref + after_ref - min_size, stretch)};
    }

    /// Get size
    pub fn size(&self) -> &LNatSize {
        return &self.size;
    }

    /// Get flex
    pub fn flex(&self) -> &LFlex {
        return &self.flex;
    }

    /// Determine if an `LReq` is flexible (its flex is not the `Fixed` variant)
    pub fn is_flexible(&self) -> bool {
        return self.flex != LFlex::Fixed;
    }

    /// Add two size requests (`LReq` instances) together, resulting in an `LReq` whose
    /// size does NOT have a reference point.
    /// See `LNatSize::add` for more information; this method combines `LNatSize::add` and
    /// `LFlex::add`.
    pub fn add(&self, other: &LReq, space_between: f64) -> LReq {
        return LReq{size: self.size.add(&other.size, space_between),
                    flex: self.flex.add(&other.flex)};
    }

    /// Add two size requests (`LReq` instances) together, resulting in an `LReq` whose
    /// size does have a reference point.
    /// See `LNatSize::add_ref` for more information; this method combines `LNatSize::add_ref` and
    /// `LFlex::add`.
    pub fn add_ref(&self, other: &LReq, space_between: f64, ref_point_from_second: bool) -> LReq {
        return LReq{size: self.size.add_ref(&other.size, space_between, ref_point_from_second),
                    flex: self.flex.add(&other.flex)};
    }

    /// Private helper for making an `LFlex` instance, given a natural size, an optional minimum
    /// size and an optional stretch.
    fn make_flex(size: LNatSize, min_size: Option<f64>, stretch: Option<f32>) -> LFlex {
        let shrink = match min_size {
            Some(m) => {
                let pref = size.size();
                if m < pref {
                    Some(pref - m)
                } else {
                    None
                }
            },
            None => None,
        };
        match (shrink, stretch) {
            (None, None) => LFlex::Fixed,
            (Some(k), None) => LFlex::Flex{shrink: k, stretch: 0.0 as f32},
            (None, Some(r)) => LFlex::Flex{shrink: 0.0, stretch: r},
            (Some(k), Some(r)) => LFlex::Flex{shrink: k, stretch: r},
        }
    }

    /// Compute the maximum of the space requests of two `LReq` instances.
    ///
    /// E.g. if the `LReq` instances are representations of vertical space requirements, the
    /// `max` method computes the space required for stacking them horizontally, with `other`
    /// positioned to the right `self`.
    ///
    /// Effectively performs the same function as `LNatSize::max`, but taking the flexibility
    /// specified by `self.flex()` into account.
    pub fn max(&self, other: &LReq) -> LReq {
        let min_size0 = self.flex.min_size(&self.size);
        let min_size1 = other.flex.min_size(&other.size);
        let stretch = fast_maxopt(self.flex.stretch_opt(), other.flex.stretch_opt());
        let min_size = fast_maxopt(min_size0, min_size1);
        let size = self.size.max(&other.size);
        return LReq{size: size, flex: LReq::make_flex(size, min_size, stretch)};
    }

    /// Cumulatively combine the space requirements of a vector of `LReq` instances, along
    /// the axis described by the `LNatSize` instances.
    ///
    /// E.g. if each `LReq` instance represents a vertical space requirement, they are
    /// stacked vertically.
    ///
    /// See `LNatSize::linear_acc` for more info; this does the same thing but taking
    /// flexibiity into account
    pub fn linear_acc(reqs: &Vec<&LReq>, space_between: f64,
                      ref_point_index: Option<usize>) -> LReq {
        // If there is no reference point index, we can filter out empty items before calling
        // LNatSize::linear_acc
        // If there is a reference point index, we shouldn't filter, as removing items
        // from the list could cause the index to point to the wrong item
        let sz = match ref_point_index {
            None => LNatSize::linear_acc(
                reqs.iter().filter(|x| x.size != LNatSize::Empty).map(|x| &x.size),
                space_between, ref_point_index),
            _ => LNatSize::linear_acc(
                reqs.iter().map(|x| &x.size),
                space_between, ref_point_index)
        };

        //
        let fl = LFlex::linear_acc(reqs.iter().filter(
                |x| x.flex != LFlex::Fixed).map(|x| &x.flex));

        return LReq{size: sz, flex: fl};
    }

    /// Cumulatively combine the space requirements of a vector of `LReq` instances along the
    /// axis that is perpendicular to that described by the `LNatSize` instances.
    ///
    /// E.g. if each `LReq` instance represents a vertical space requirement, they are stacked
    /// horizontally.
    ///
    /// See `LNatSize::perpendicular_acc` for more info; this does the same thing but taking
    /// flexibiity into account.
    pub fn perpendicular_acc(reqs: &Vec<&LReq>) -> LReq {
        // Would prefer to use an iterator based solutions above for the purpose of elegance, but
        // walking the array in a single loop is faster
        let mut max_min_size = 0.0;
        let mut got_min_size = false;
        let mut max_stretch = 0.0;
        let mut got_stretch = false;
        for x in reqs {
            match x.flex.min_size(&x.size()) {
                None => {},
                Some(m) => {
                    max_min_size = fast_max(max_min_size, m);
                    got_min_size = true;
                }
            }

            match x.flex {
                LFlex::Fixed => {},
                LFlex::Flex{stretch: s, ..} => {
                    max_stretch = fast_max(max_stretch, s);
                    got_stretch = true;
                }
            }
        }
        let min_size = if got_min_size {Some(max_min_size)} else {None};
        let stretch = if got_stretch {Some(max_stretch)} else {None};

        let sz = LNatSize::perpendicular_acc(reqs.iter().map(|x| &x.size));

        return LReq{size: sz, flex: LReq::make_flex(sz, min_size, stretch)};
    }
}


//
// TESTS
//

#[cfg(test)]
mod tests {
    extern crate rand;
    extern crate test;

    use std::mem;
    use self::rand::distributions::{Range, IndependentSample};
    use super::*;


    //
    //
    // LNatSize
    //
    //

    #[test]
    fn test_lnatsize_new_and_accessors() {
        let a = LNatSize::new_empty();
        assert_eq!(a.size(), 0.0);
        assert_eq!(a.before_ref(), 0.0);
        assert_eq!(a.after_ref(), 0.0);
        assert_eq!(a.before_ref_opt(), None);
        assert_eq!(a.after_ref_opt(), None);
        assert_eq!(a.before_and_after_ref_opt(), None);

        let b = LNatSize::new_size(1.0);
        assert_eq!(b.size(), 1.0);
        assert_eq!(b.before_ref(), 0.5);
        assert_eq!(b.after_ref(), 0.5);
        assert_eq!(a.before_ref_opt(), None);
        assert_eq!(a.after_ref_opt(), None);
        assert_eq!(b.before_and_after_ref_opt(), None);

        let c = LNatSize::new_ref(3.0, 2.0);
        assert_eq!(c.size(), 5.0);
        assert_eq!(c.before_ref(), 3.0);
        assert_eq!(c.after_ref(), 2.0);
        assert_eq!(c.before_ref_opt(), Some(3.0));
        assert_eq!(c.after_ref_opt(), Some(2.0));
        assert_eq!(c.before_and_after_ref_opt(), Some((3.0, 2.0)));
    }

    #[test]
    fn test_lnatsize_add() {
        assert_eq!(LNatSize::new_empty().add(&LNatSize::new_empty(), 100.0),
            LNatSize::new_empty());

        assert_eq!(LNatSize::new_empty().add(&LNatSize::new_size(5.0), 100.0),
            LNatSize::new_size(5.0));
        assert_eq!(LNatSize::new_empty().add(&LNatSize::new_ref(3.0, 2.0), 100.0),
            LNatSize::new_size(5.0));

        assert_eq!(LNatSize::new_size(5.0).add(&LNatSize::new_empty(), 100.0),
            LNatSize::new_size(5.0));
        assert_eq!(LNatSize::new_ref(3.0, 2.0).add(&LNatSize::new_empty(), 100.0),
            LNatSize::new_size(5.0));

        assert_eq!(LNatSize::new_size(5.0).add(&LNatSize::new_size(5.0), 100.0),
            LNatSize::new_size(110.0));
        assert_eq!(LNatSize::new_size(5.0).add(&LNatSize::new_ref(3.0, 2.0), 100.0),
            LNatSize::new_size(110.0));
        assert_eq!(LNatSize::new_ref(3.0, 2.0).add(&LNatSize::new_size(5.0), 100.0),
            LNatSize::new_size(110.0));
        assert_eq!(LNatSize::new_ref(3.0, 2.0).add(&LNatSize::new_ref(3.0, 2.0), 100.0),
            LNatSize::new_size(110.0));
    }

    #[test]
    fn test_lnatsize_add_ref() {
        assert_eq!(LNatSize::new_empty().add_ref(&LNatSize::new_empty(), 100.0, false),
            LNatSize::new_empty());
        assert_eq!(LNatSize::new_empty().add_ref(&LNatSize::new_empty(), 100.0, true),
            LNatSize::new_empty());

        assert_eq!(LNatSize::new_empty().add_ref(&LNatSize::new_size(5.0), 100.0, false),
            LNatSize::new_ref(0.0, 5.0));
        assert_eq!(LNatSize::new_empty().add_ref(&LNatSize::new_size(5.0), 100.0, true),
            LNatSize::new_ref(2.5, 2.5));
        assert_eq!(LNatSize::new_empty().add_ref(&LNatSize::new_ref(3.0, 2.0), 100.0, false),
            LNatSize::new_ref(0.0, 5.0));
        assert_eq!(LNatSize::new_empty().add_ref(&LNatSize::new_ref(3.0, 2.0), 100.0, true),
            LNatSize::new_ref(3.0, 2.0));

        assert_eq!(LNatSize::new_size(5.0).add_ref(&LNatSize::new_empty(), 100.0, false),
            LNatSize::new_ref(2.5, 2.5));
        assert_eq!(LNatSize::new_size(5.0).add_ref(&LNatSize::new_empty(), 100.0, true),
            LNatSize::new_ref(5.0, 0.0));
        assert_eq!(LNatSize::new_ref(3.0, 2.0).add_ref(&LNatSize::new_empty(), 100.0, false),
            LNatSize::new_ref(3.0, 2.0));
        assert_eq!(LNatSize::new_ref(3.0, 2.0).add_ref(&LNatSize::new_empty(), 100.0, true),
            LNatSize::new_ref(5.0, 0.0));

        assert_eq!(LNatSize::new_size(5.0).add_ref(&LNatSize::new_size(5.0), 100.0, false),
            LNatSize::new_ref(2.5, 107.5));
        assert_eq!(LNatSize::new_size(5.0).add_ref(&LNatSize::new_size(5.0), 100.0, true),
            LNatSize::new_ref(107.5, 2.5));
        assert_eq!(LNatSize::new_size(5.0).add_ref(&LNatSize::new_ref(3.0, 2.0), 100.0, false),
            LNatSize::new_ref(2.5, 107.5));
        assert_eq!(LNatSize::new_size(5.0).add_ref(&LNatSize::new_ref(3.0, 2.0), 100.0, true),
            LNatSize::new_ref(108.0, 2.0));
        assert_eq!(LNatSize::new_ref(3.0, 2.0).add_ref(&LNatSize::new_size(5.0), 100.0, false),
            LNatSize::new_ref(3.0, 107.0));
        assert_eq!(LNatSize::new_ref(3.0, 2.0).add_ref(&LNatSize::new_size(5.0), 100.0, true),
            LNatSize::new_ref(107.5, 2.5));
        assert_eq!(LNatSize::new_ref(3.0, 2.0).add_ref(&LNatSize::new_ref(3.0, 2.0), 100.0, false),
            LNatSize::new_ref(3.0, 107.0));
        assert_eq!(LNatSize::new_ref(3.0, 2.0).add_ref(&LNatSize::new_ref(3.0, 2.0), 100.0, true),
            LNatSize::new_ref(108.0, 2.0));
    }

    #[test]
    fn test_lnatsize_max() {
        assert_eq!(LNatSize::new_empty().max(&LNatSize::new_empty()), LNatSize::new_empty());

        assert_eq!(LNatSize::new_empty().max(&LNatSize::new_size(5.0)), LNatSize::new_size(5.0));
        assert_eq!(LNatSize::new_empty().max(&LNatSize::new_ref(3.0, 2.0)),
            LNatSize::new_ref(3.0, 2.0));

        assert_eq!(LNatSize::new_size(5.0).max(&LNatSize::new_empty()), LNatSize::new_size(5.0));
        assert_eq!(LNatSize::new_ref(3.0, 2.0).max(&LNatSize::new_empty()),
            LNatSize::new_ref(3.0, 2.0));

        assert_eq!(LNatSize::new_size(5.0).max(&LNatSize::new_ref(6.0, 4.0)),
            LNatSize::new_ref(6.0, 4.0));
        assert_eq!(LNatSize::new_size(5.0).max(&LNatSize::new_ref(1.5, 1.0)),
            LNatSize::new_ref(2.75, 2.25));
        assert_eq!(LNatSize::new_ref(6.0, 4.0).max(&LNatSize::new_size(5.0)),
            LNatSize::new_ref(6.0, 4.0));
        assert_eq!(LNatSize::new_ref(1.5, 1.0).max(&LNatSize::new_size(5.0)),
            LNatSize::new_ref(2.75, 2.25));
    }

    #[test]
    fn test_lnatsize_linear_acc() {
        let a = LNatSize::new_size(1.0);
        let b = LNatSize::new_size(5.0);
        let c = LNatSize::new_size(4.0);

        assert_eq!(LNatSize::linear_acc(vec![].into_iter(), 0.0, None),
            LNatSize::new_empty());
        assert_eq!(LNatSize::linear_acc(vec![].into_iter(), 100.0, None),
            LNatSize::new_empty());

        assert_eq!(LNatSize::linear_acc(vec![&a].into_iter(), 0.0, None),
            LNatSize::new_size(1.0));
        assert_eq!(LNatSize::linear_acc(vec![&a].into_iter(), 100.0, None),
            LNatSize::new_size(1.0));

        assert_eq!(LNatSize::linear_acc(vec![&a, &b].into_iter(), 0.0, None),
            LNatSize::new_size(6.0));
        assert_eq!(LNatSize::linear_acc(vec![&a, &b].into_iter(), 100.0, None),
            LNatSize::new_size(106.0));

        assert_eq!(LNatSize::linear_acc(vec![&a, &b, &c].into_iter(), 0.0, None),
            LNatSize::new_size(10.0));
        assert_eq!(LNatSize::linear_acc(vec![&a, &b, &c].into_iter(), 100.0, None),
            LNatSize::new_size(210.0));
    }

    #[test]
    fn test_lnatsize_linear_acc_ref() {
        let a = LNatSize::new_ref(7.0, 3.0);
        let b = LNatSize::new_ref(11.0, 9.0);
        let c = LNatSize::new_ref(5.0, 10.0);
        let e0 = LNatSize::new_empty();

        assert_eq!(LNatSize::linear_acc(vec![].into_iter(), 0.0, None),
            LNatSize::new_empty());
        assert_eq!(LNatSize::linear_acc(vec![].into_iter(), 0.0, Some(0)),
            LNatSize::new_empty());
        assert_eq!(LNatSize::linear_acc(vec![].into_iter(), 100.0, None),
            LNatSize::new_empty());
        assert_eq!(LNatSize::linear_acc(vec![].into_iter(), 100.0, Some(0)),
            LNatSize::new_empty());

        assert_eq!(LNatSize::linear_acc(vec![&a].into_iter(), 0.0, None),
            LNatSize::new_size(10.0));
        assert_eq!(LNatSize::linear_acc(vec![&a].into_iter(), 0.0, Some(0)),
            LNatSize::new_ref(7.0, 3.0));
        assert_eq!(LNatSize::linear_acc(vec![&a].into_iter(), 100.0, None),
            LNatSize::new_size(10.0));
        assert_eq!(LNatSize::linear_acc(vec![&a].into_iter(), 100.0, Some(0)),
            LNatSize::new_ref(7.0, 3.0));

        assert_eq!(LNatSize::linear_acc(vec![&a, &b].into_iter(), 0.0, None),
            LNatSize::new_size(30.0));
        assert_eq!(LNatSize::linear_acc(vec![&a, &b].into_iter(), 0.0, Some(0)),
            LNatSize::new_ref(7.0, 23.0));
        assert_eq!(LNatSize::linear_acc(vec![&a, &b].into_iter(), 0.0, Some(1)),
            LNatSize::new_ref(21.0, 9.0));
        assert_eq!(LNatSize::linear_acc(vec![&a, &b].into_iter(), 100.0, None),
            LNatSize::new_size(130.0));
        assert_eq!(LNatSize::linear_acc(vec![&a, &b].into_iter(), 100.0, Some(0)),
            LNatSize::new_ref(7.0, 123.0));
        assert_eq!(LNatSize::linear_acc(vec![&a, &b].into_iter(), 100.0, Some(1)),
            LNatSize::new_ref(121.0, 9.0));

        assert_eq!(LNatSize::linear_acc(vec![&a, &b, &c].into_iter(), 0.0, None),
            LNatSize::new_size(45.0));
        assert_eq!(LNatSize::linear_acc(vec![&a, &b, &c].into_iter(), 0.0, Some(0)),
            LNatSize::new_ref(7.0, 38.0));
        assert_eq!(LNatSize::linear_acc(vec![&a, &b, &c].into_iter(), 0.0, Some(1)),
            LNatSize::new_ref(21.0, 24.0));
        assert_eq!(LNatSize::linear_acc(vec![&a, &b, &c].into_iter(), 0.0, Some(2)),
            LNatSize::new_ref(35.0, 10.0));
        assert_eq!(LNatSize::linear_acc(vec![&a, &b, &c].into_iter(), 100.0, None),
            LNatSize::new_size(245.0));
        assert_eq!(LNatSize::linear_acc(vec![&a, &b, &c].into_iter(), 100.0, Some(0)),
            LNatSize::new_ref(7.0, 238.0));
        assert_eq!(LNatSize::linear_acc(vec![&a, &b, &c].into_iter(), 100.0, Some(1)),
            LNatSize::new_ref(121.0, 124.0));
        assert_eq!(LNatSize::linear_acc(vec![&a, &b, &c].into_iter(), 100.0, Some(2)),
            LNatSize::new_ref(235.0, 10.0));

        assert_eq!(LNatSize::linear_acc(vec![&a, &e0, &b].into_iter(), 0.0, None),
            LNatSize::new_size(30.0));
        assert_eq!(LNatSize::linear_acc(vec![&a, &e0, &b].into_iter(), 0.0, Some(0)),
            LNatSize::new_ref(7.0, 23.0));
        assert_eq!(LNatSize::linear_acc(vec![&a, &e0, &b].into_iter(), 0.0, Some(1)),
            LNatSize::new_ref(10.0, 20.0));
        assert_eq!(LNatSize::linear_acc(vec![&a, &e0, &b].into_iter(), 0.0, Some(2)),
            LNatSize::new_ref(21.0, 9.0));
    }

    #[test]
    fn test_lnatsize_perpendicular_acc() {
        let et = LNatSize::new_empty();
        let s2 = LNatSize::new_size(2.0);
        let s5 = LNatSize::new_size(5.0);
        let s10 = LNatSize::new_size(10.0);
        let r1_1 = LNatSize::new_ref(1.0, 1.0);
        let r3_2 = LNatSize::new_ref(3.0, 2.0);
        let r2_6 = LNatSize::new_ref(2.0, 6.0);

        assert_eq!(LNatSize::perpendicular_acc(vec![].into_iter()), LNatSize::new_empty());
        assert_eq!(LNatSize::perpendicular_acc(vec![&et].into_iter()), LNatSize::new_empty());
        assert_eq!(LNatSize::perpendicular_acc(vec![&et, &et].into_iter()), LNatSize::new_empty());

        assert_eq!(LNatSize::perpendicular_acc(vec![&s2].into_iter()), LNatSize::new_size(2.0));
        assert_eq!(LNatSize::perpendicular_acc(vec![&s2, &s5].into_iter()),
            LNatSize::new_size(5.0));
        assert_eq!(LNatSize::perpendicular_acc(vec![&s2, &s5, &s10].into_iter()),
            LNatSize::new_size(10.0));

        assert_eq!(LNatSize::perpendicular_acc(vec![&r1_1].into_iter()),
            LNatSize::new_ref(1.0, 1.0));
        assert_eq!(LNatSize::perpendicular_acc(vec![&r1_1, &r3_2].into_iter()),
            LNatSize::new_ref(3.0, 2.0));
        assert_eq!(LNatSize::perpendicular_acc(vec![&r3_2, &r2_6].into_iter()),
            LNatSize::new_ref(3.0, 6.0));
        assert_eq!(LNatSize::perpendicular_acc(vec![&r3_2, &r2_6, &s10].into_iter()),
            LNatSize::new_ref(3.5, 6.5));
        assert_eq!(LNatSize::perpendicular_acc(vec![&r3_2, &s10].into_iter()),
            LNatSize::new_ref(5.5, 4.5));
    }

    #[test]
    fn test_lnatsize_mem() {
        assert_eq!(mem::size_of::<LNatSize>(), 24);
    }


    //
    //
    // LFlex
    //
    //

    #[test]
    fn test_lflex_new_and_accessors() {
        let a = LFlex::new_fixed();
        assert_eq!(a.shrink(), 0.0);
        assert_eq!(a.stretch(), 0.0);

        let b = LFlex::new_flex(1.0, 2.0);
        assert_eq!(b.shrink(), 1.0);
        assert_eq!(b.stretch(), 2.0);
    }

    #[test]
    fn test_lflex_add() {
        assert_eq!(LFlex::new_fixed().add(&LFlex::new_fixed()), LFlex::new_fixed());

        assert_eq!(LFlex::new_fixed().add(&LFlex::new_flex(3.0, 2.0)), LFlex::new_flex(3.0, 2.0));
        assert_eq!(LFlex::new_flex(3.0, 2.0).add(&LFlex::new_fixed()), LFlex::new_flex(3.0, 2.0));

        assert_eq!(LFlex::new_flex(3.0, 2.0).add(&LFlex::new_flex(10.0, 20.0)),
            LFlex::new_flex(13.0, 22.0));
    }

    #[test]
    fn test_lflex_linear_acc() {
        let d = LFlex::new_fixed();
        let l1_2 = LFlex::new_flex(1.0, 2.0);
        let l3_8 = LFlex::new_flex(3.0, 8.0);

        assert_eq!(LFlex::linear_acc(vec![].into_iter()), LFlex::new_fixed());
        assert_eq!(LFlex::linear_acc(vec![&d].into_iter()), LFlex::new_fixed());
        assert_eq!(LFlex::linear_acc(vec![&d, &d].into_iter()), LFlex::new_fixed());

        assert_eq!(LFlex::linear_acc(vec![&l1_2].into_iter()), LFlex::new_flex(1.0, 2.0));
        assert_eq!(LFlex::linear_acc(vec![&d, &l1_2].into_iter()), LFlex::new_flex(1.0, 2.0));
        assert_eq!(LFlex::linear_acc(vec![&l1_2, &d].into_iter()), LFlex::new_flex(1.0, 2.0));

        assert_eq!(LFlex::linear_acc(vec![&l1_2, &l3_8].into_iter()), LFlex::new_flex(4.0, 10.0));
        assert_eq!(LFlex::linear_acc(vec![&d, &l1_2, &l3_8].into_iter()), LFlex::new_flex(4.0, 10.0));
        assert_eq!(LFlex::linear_acc(vec![&l1_2, &l3_8, &d].into_iter()), LFlex::new_flex(4.0, 10.0));
    }

    #[test]
    fn test_lflex_mem() {
        assert_eq!(mem::size_of::<LFlex>(), 24);
    }


    //
    //
    // LFlex
    //
    //

    #[test]
    fn test_lreq_new_and_accessors() {
        let a = LReq::new_empty();
        assert_eq!(a.size(), &LNatSize::new_empty());
        assert_eq!(a.flex(), &LFlex::new_fixed());

        let b = LReq::new(LNatSize::new_ref(1.0, 2.0), LFlex::new_flex(3.0, 4.0));
        assert_eq!(b.size(), &LNatSize::new_ref(1.0, 2.0));
        assert_eq!(b.flex(), &LFlex::new_flex(3.0, 4.0));

        let c = LReq::new_fixed_size(10.0);
        assert_eq!(c.size(), &LNatSize::new_size(10.0));
        assert_eq!(c.flex(), &LFlex::new_fixed());

        let d = LReq::new_flex_size(10.0, 3.0, 4.0);
        assert_eq!(d.size(), &LNatSize::new_size(10.0));
        assert_eq!(d.flex(), &LFlex::new_flex(3.0, 4.0));

        let e = LReq::new_fixed_ref(7.0, 5.0);
        assert_eq!(e.size(), &LNatSize::new_ref(7.0, 5.0));
        assert_eq!(e.flex(), &LFlex::new_fixed());

        let f = LReq::new_flex_ref(7.0, 5.0, 3.0, 4.0);
        assert_eq!(f.size(), &LNatSize::new_ref(7.0, 5.0));
        assert_eq!(f.flex(), &LFlex::new_flex(3.0, 4.0));
    }

    #[test]
    fn test_lreq_add() {
        assert_eq!(LReq::new_empty().add(&LReq::new_empty(), 10.0), LReq::new_empty());

        assert_eq!(LReq::new_empty().add(&LReq::new_fixed_size(20.0), 100.0),
            LReq::new_fixed_size(20.0));
        assert_eq!(LReq::new_empty().add(&LReq::new_fixed_ref(16.0, 4.0), 100.0),
            LReq::new_fixed_size(20.0));
        assert_eq!(LReq::new_empty().add(&LReq::new_flex_size(20.0, 1.0, 2.0), 100.0),
            LReq::new_flex_size(20.0, 1.0, 2.0));
        assert_eq!(LReq::new_empty().add(&LReq::new_flex_ref(16.0, 4.0, 1.0, 2.0), 100.0),
            LReq::new_flex_size(20.0, 1.0, 2.0));

        assert_eq!(LReq::new_fixed_size(10.0).add(&LReq::new_empty(), 100.0),
            LReq::new_fixed_size(10.0));
        assert_eq!(LReq::new_fixed_ref(6.0, 4.0).add(&LReq::new_empty(), 100.0),
            LReq::new_fixed_size(10.0));
        assert_eq!(LReq::new_flex_size(10.0, 1.0, 2.0).add(&LReq::new_empty(), 100.0),
            LReq::new_flex_size(10.0, 1.0, 2.0));
        assert_eq!(LReq::new_flex_ref(6.0, 4.0, 1.0, 2.0).add(&LReq::new_empty(), 100.0),
            LReq::new_flex_size(10.0, 1.0, 2.0));

        assert_eq!(LReq::new_fixed_size(10.0).add(&LReq::new_fixed_size(20.0), 0.0),
            LReq::new_fixed_size(30.0));
        assert_eq!(LReq::new_fixed_size(10.0).add(&LReq::new_fixed_size(20.0), 100.0),
            LReq::new_fixed_size(130.0));
        assert_eq!(LReq::new_flex_size(10.0, 1.0, 2.0).add(&LReq::new_flex_size(20.0, 1.0, 2.0),
            0.0), LReq::new_flex_size(30.0, 2.0, 4.0));
        assert_eq!(LReq::new_flex_size(10.0, 1.0, 2.0).add(&LReq::new_flex_size(20.0, 1.0, 2.0),
            100.0), LReq::new_flex_size(130.0, 2.0, 4.0));

        let flexible_nothing = LReq{size: LNatSize::new_empty(), flex: LFlex::new_flex(1.0, 2.0)};
        assert_eq!(LReq::new_fixed_size(10.0).add(&flexible_nothing, 0.0),
            LReq::new_flex_size(10.0, 1.0, 2.0));
    }

    #[test]
    fn test_lreq_add_ref() {
        assert_eq!(LReq::new_empty().add_ref(&LReq::new_empty(), 10.0, false), LReq::new_empty());

        assert_eq!(LReq::new_empty().add_ref(&LReq::new_fixed_size(20.0), 100.0, true),
            LReq::new_fixed_ref(10.0, 10.0));
        assert_eq!(LReq::new_empty().add_ref(&LReq::new_fixed_ref(16.0, 4.0), 100.0, true),
            LReq::new_fixed_ref(16.0, 4.0));
        assert_eq!(LReq::new_empty().add_ref(&LReq::new_flex_size(20.0, 1.0, 2.0), 100.0, true),
            LReq::new_flex_ref(10.0, 10.0, 1.0, 2.0));
        assert_eq!(LReq::new_empty().add_ref(
            &LReq::new_flex_ref(16.0, 4.0, 1.0, 2.0), 100.0, true),
                LReq::new_flex_ref(16.0, 4.0, 1.0, 2.0));

        assert_eq!(LReq::new_fixed_size(10.0).add_ref(&LReq::new_empty(), 100.0, false),
            LReq::new_fixed_ref(5.0, 5.0));
        assert_eq!(LReq::new_fixed_ref(6.0, 4.0).add_ref(&LReq::new_empty(), 100.0, false),
            LReq::new_fixed_ref(6.0, 4.0));
        assert_eq!(LReq::new_flex_size(10.0, 1.0, 2.0).add_ref(&LReq::new_empty(), 100.0, false),
            LReq::new_flex_ref(5.0, 5.0, 1.0, 2.0));
        assert_eq!(LReq::new_flex_ref(6.0, 4.0, 1.0, 2.0).add_ref(
            &LReq::new_empty(), 100.0, false), LReq::new_flex_ref(6.0, 4.0, 1.0, 2.0));


        assert_eq!(LReq::new_fixed_size(10.0).add_ref(&LReq::new_fixed_size(20.0), 0.0, false),
            LReq::new_fixed_ref(5.0, 25.0));
        assert_eq!(LReq::new_fixed_size(10.0).add_ref(&LReq::new_fixed_size(20.0), 100.0, false),
            LReq::new_fixed_ref(5.0, 125.0));
        assert_eq!(LReq::new_flex_size(10.0, 1.0, 2.0).add_ref(
            &LReq::new_flex_size(20.0, 1.0, 2.0), 0.0, false),
            LReq::new_flex_ref(5.0, 25.0, 2.0, 4.0));
        assert_eq!(LReq::new_flex_size(10.0, 1.0, 2.0).add_ref(
            &LReq::new_flex_size(20.0, 1.0, 2.0), 100.0, false),
                LReq::new_flex_ref(5.0, 125.0, 2.0, 4.0));

        let flexible_nothing = LReq{size: LNatSize::new_empty(), flex: LFlex::new_flex(1.0, 2.0)};
        assert_eq!(LReq::new_fixed_size(10.0).add_ref(&flexible_nothing, 0.0, false),
            LReq::new_flex_ref(5.0, 5.0, 1.0, 2.0));
    }

    #[test]
    fn test_lreq_max() {
        assert_eq!(LReq::new_empty().max(&LReq::new_empty()), LReq::new_empty());

        assert_eq!(LReq::new_empty().max(&LReq::new_fixed_size(20.0)),
            LReq::new_fixed_size(20.0));
        assert_eq!(LReq::new_empty().max(&LReq::new_fixed_ref(16.0, 4.0)),
            LReq::new_fixed_ref(16.0, 4.0));
        assert_eq!(LReq::new_empty().max(&LReq::new_flex_size(20.0, 1.0, 2.0)),
            LReq::new_flex_size(20.0, 1.0, 2.0));
        assert_eq!(LReq::new_empty().max(&LReq::new_flex_ref(16.0, 4.0, 1.0, 2.0)),
            LReq::new_flex_ref(16.0, 4.0, 1.0, 2.0));

        assert_eq!(LReq::new_fixed_size(10.0).max(&LReq::new_empty()),
            LReq::new_fixed_size(10.0));
        assert_eq!(LReq::new_fixed_ref(6.0, 4.0).max(&LReq::new_empty()),
            LReq::new_fixed_ref(6.0, 4.0));
        assert_eq!(LReq::new_flex_size(10.0, 1.0, 2.0).max(&LReq::new_empty()),
            LReq::new_flex_size(10.0, 1.0, 2.0));
        assert_eq!(LReq::new_flex_ref(6.0, 4.0, 1.0, 2.0).max(&LReq::new_empty()),
            LReq::new_flex_ref(6.0, 4.0, 1.0, 2.0));

        assert_eq!(LReq::new_fixed_size(10.0).max(&LReq::new_fixed_size(20.0)),
            LReq::new_fixed_size(20.0));
        assert_eq!(LReq::new_flex_size(10.0, 1.0, 2.0).max(&LReq::new_flex_size(12.0, 5.0, 1.0)),
            LReq::new_flex_size(12.0, 3.0, 2.0));

        let flexible_nothing = LReq{size: LNatSize::new_empty(), flex: LFlex::new_flex(1.0, 2.0)};
        assert_eq!(LReq::new_fixed_size(10.0).max(&flexible_nothing),
            LReq::new_flex_size(10.0, 0.0, 2.0));

        // Check flexible vs Fixed
        assert_eq!(LReq::new_fixed_size(16.0).max(&LReq::new_flex_size(12.0, 4.0, 0.0)),
            LReq::new_fixed_size(16.0));
        assert_eq!(LReq::new_fixed_size(16.0).max(&LReq::new_flex_size(12.0, 4.0, 1.0)),
            LReq::new_flex_size(16.0, 0.0, 1.0));
        assert_eq!(LReq::new_fixed_size(10.0).max(&LReq::new_flex_size(12.0, 4.0, 0.0)),
            LReq::new_flex_size(12.0, 2.0, 0.0));
        assert_eq!(LReq::new_fixed_size(10.0).max(&LReq::new_flex_size(12.0, 4.0, 1.0)),
            LReq::new_flex_size(12.0, 2.0, 1.0));
        assert_eq!(LReq::new_fixed_size(6.0).max(&LReq::new_flex_size(12.0, 4.0, 0.0)),
            LReq::new_flex_size(12.0, 4.0, 0.0));
        assert_eq!(LReq::new_fixed_size(6.0).max(&LReq::new_flex_size(12.0, 4.0, 1.0)),
            LReq::new_flex_size(12.0, 4.0, 1.0));

        assert_eq!(LReq::new_flex_size(12.0, 4.0, 0.0).max(&LReq::new_fixed_size(16.0)),
            LReq::new_fixed_size(16.0));
        assert_eq!(LReq::new_flex_size(12.0, 4.0, 1.0).max(&LReq::new_fixed_size(16.0)),
            LReq::new_flex_size(16.0, 0.0, 1.0));
        assert_eq!(LReq::new_flex_size(12.0, 4.0, 0.0).max(&LReq::new_fixed_size(10.0)),
            LReq::new_flex_size(12.0, 2.0, 0.0));
        assert_eq!(LReq::new_flex_size(12.0, 4.0, 1.0).max(&LReq::new_fixed_size(10.0)),
            LReq::new_flex_size(12.0, 2.0, 1.0));
        assert_eq!(LReq::new_flex_size(12.0, 4.0, 0.0).max(&LReq::new_fixed_size(6.0)),
            LReq::new_flex_size(12.0, 4.0, 0.0));
        assert_eq!(LReq::new_flex_size(12.0, 4.0, 1.0).max(&LReq::new_fixed_size(6.0)),
            LReq::new_flex_size(12.0, 4.0, 1.0));
    }

    #[test]
    fn test_lreq_linear_acc() {
        let ds0 = LReq::new_fixed_size(10.0);
        let ds1 = LReq::new_fixed_size(20.0);
        let dr0 = LReq::new_fixed_ref(7.0, 3.0);
        let dr1 = LReq::new_fixed_ref(14.0, 6.0);
        let gl0 = LReq::new_flex_size(10.0, 1.0, 2.0);

        assert_eq!(LReq::linear_acc(&vec![], 0.0, None), LReq::new_empty());
        assert_eq!(LReq::linear_acc(&vec![&ds0], 0.0, None), ds0);

        assert_eq!(LReq::linear_acc(&vec![&ds0, &gl0, &ds1], 0.0, None),
            LReq::new_flex_size(40.0, 1.0, 2.0));
        assert_eq!(LReq::linear_acc(&vec![&dr0, &gl0, &dr1], 0.0, None),
            LReq::new_flex_size(40.0, 1.0, 2.0));
        assert_eq!(LReq::linear_acc(&vec![&dr0, &gl0, &dr1], 0.0, Some(0)),
            LReq::new_flex_ref(7.0, 33.0, 1.0, 2.0));
        assert_eq!(LReq::linear_acc(&vec![&dr0, &gl0, &dr1], 0.0, Some(1)),
            LReq::new_flex_ref(15.0, 25.0, 1.0, 2.0));
        assert_eq!(LReq::linear_acc(&vec![&dr0, &gl0, &dr1], 0.0, Some(2)),
            LReq::new_flex_ref(34.0, 6.0, 1.0, 2.0));
    }

    #[test]
    fn test_lreq_perpendicular_acc() {
        let ds0 = LReq::new_fixed_size(12.0);
        let ds1 = LReq::new_flex_size(10.0, 2.0, 2.0);
        let ds2 = LReq::new_flex_size(8.0, 1.0, 1.0);
        let ds3 = LReq::new_flex_size(9.0, 0.5, 1.0);

        let dr0 = LReq::new_fixed_ref(8.0, 4.0);
        let dr1 = LReq::new_flex_ref(6.0, 3.0, 2.0, 2.0);
        let dr2 = LReq::new_flex_ref(5.0, 4.0, 1.5, 1.0);
        let dr3 = LReq::new_flex_ref(4.0, 4.0, 1.0, 1.0);
        let dr4 = LReq::new_flex_ref(5.0, 4.0, 0.5, 1.0);

        assert_eq!(LReq::perpendicular_acc(&vec![]), LReq::new_empty());
        assert_eq!(LReq::perpendicular_acc(&vec![&ds0]), ds0);

        assert_eq!(LReq::perpendicular_acc(&vec![&ds0, &ds1, &ds2]),
            LReq::new_flex_size(12.0, 0.0, 2.0));
        assert_eq!(LReq::perpendicular_acc(&vec![&ds1, &ds2, &ds3]),
            LReq::new_flex_size(10.0, 1.5, 2.0));

        assert_eq!(LReq::perpendicular_acc(&vec![&dr0, &dr1, &dr2]),
            LReq::new_flex_ref(8.0, 4.0, 0.0, 2.0));
        assert_eq!(LReq::perpendicular_acc(&vec![&dr1, &dr2, &dr3, &dr4]),
            LReq::new_flex_ref(6.0, 4.0, 1.5, 2.0));
    }

    #[test]
    fn test_lreq_mem() {
        assert_eq!(mem::size_of::<LReq>(), 48);
    }


    #[bench]
    fn bench_lsize_linear_acc(bench: &mut test::Bencher) {
        let num_children = 100;
        let num_parents = 100;

        let kind_range: Range<i32> = Range::new(0, 8);
        let size_range = Range::new(5.0, 25.0);
        let mut rng = rand::thread_rng();

        let mut children = Vec::with_capacity(num_children);
        let mut parents = Vec::with_capacity(num_parents);

        for _ in 0..num_children {
            let x = match kind_range.ind_sample(&mut rng) {
                0 => LNatSize::new_empty(),
                1 ... 4 => LNatSize::new_size(size_range.ind_sample(&mut rng)),
                _ => {LNatSize::new_ref(size_range.ind_sample(&mut rng) * 0.5,
                                     size_range.ind_sample(&mut rng) * 0.5)}
            };
            children.push(x);
        }

        bench.iter(|| {
            parents.clear();
            for _ in 0..num_parents {
                let p = LNatSize::linear_acc(children.iter(), 3.0, None);
                parents.push(p);
            }
        });
    }

    #[bench]
    fn bench_lsize_perpendicular_acc(bench: &mut test::Bencher) {
        let num_children = 100;
        let num_parents = 100;

        let kind_range: Range<i32> = Range::new(0, 8);
        let size_range = Range::new(5.0, 25.0);
        let mut rng = rand::thread_rng();

        let mut children = Vec::with_capacity(num_children);
        let mut parents = Vec::with_capacity(num_parents);

        for _ in 0..num_children {
            let x = match kind_range.ind_sample(&mut rng) {
                0 => LNatSize::new_empty(),
                1 ... 4 => LNatSize::new_size(size_range.ind_sample(&mut rng)),
                _ => {LNatSize::new_ref(size_range.ind_sample(&mut rng) * 0.5,
                                     size_range.ind_sample(&mut rng) * 0.5)}
            };
            children.push(x);
        }

        bench.iter(|| {
            parents.clear();
            for _ in 0..num_parents {
                let p = LNatSize::perpendicular_acc(children.iter());
                parents.push(p);
            }
        });
    }


    #[bench]
    fn bench_lreq_linear_acc(bench: &mut test::Bencher) {
        let num_children = 100;
        let num_parents = 100;

        let natsize_type_range: Range<i32> = Range::new(0, 8);
        let size_range = Range::new(5.0, 25.0);
        let flex_type_range: Range<i32> = Range::new(0, 2);
        let flex_range = Range::new(1.0, 3.0);
        let mut rng = rand::thread_rng();

        let mut children = Vec::with_capacity(num_children);
        let mut parents = Vec::with_capacity(num_parents);

        for _ in 0..num_children
         {
            let size = match natsize_type_range.ind_sample(&mut rng) {
                0 => LNatSize::new_empty(),
                1 ... 4 => LNatSize::new_size(size_range.ind_sample(&mut rng)),
                _ => {LNatSize::new_ref(size_range.ind_sample(&mut rng) * 0.5,
                                     size_range.ind_sample(&mut rng) * 0.5)}
            };
            let flex = match flex_type_range.ind_sample(&mut rng) {
                0 => LFlex::new_fixed(),
                1 => LFlex::new_flex(flex_range.ind_sample(&mut rng),
                                     flex_range.ind_sample(&mut rng) as f32),
                _ => {panic!();},
            };
            children.push(LReq::new(size, flex));
        }

        let child_refs: Vec<&LReq> = children.iter().collect();

        bench.iter(|| {
            parents.clear();
            for _ in 0..num_parents {
                let p = LReq::linear_acc(&child_refs, 3.0, None);
                parents.push(p);
            }
        });
    }

    #[bench]
    fn bench_lreq_perpendicular_acc(bench: &mut test::Bencher) {
        let num_children = 100;
        let num_parents = 100;

        let natsize_type_range: Range<i32> = Range::new(0, 8);
        let size_range = Range::new(5.0, 25.0);
        let flex_type_range: Range<i32> = Range::new(0, 2);
        let flex_range = Range::new(1.0, 3.0);
        let mut rng = rand::thread_rng();

        let mut children = Vec::with_capacity(num_children);
        let mut parents = Vec::with_capacity(num_parents
            );

        for _ in 0..num_children {
            let size = match natsize_type_range.ind_sample(&mut rng) {
                0 => LNatSize::new_empty(),
                1 ... 4 => LNatSize::new_size(size_range.ind_sample(&mut rng)),
                _ => {LNatSize::new_ref(size_range.ind_sample(&mut rng) * 0.5,
                                     size_range.ind_sample(&mut rng) * 0.5)}
            };
            let flex = match flex_type_range.ind_sample(&mut rng) {
                0 => LFlex::new_fixed(),
                1 => LFlex::new_flex(flex_range.ind_sample(&mut rng),
                                     flex_range.ind_sample(&mut rng) as f32),
                _ => {panic!();},
            };
            children.push(LReq::new(size, flex));
        }

        let child_refs: Vec<&LReq> = children.iter().collect();

        bench.iter(|| {
            parents.clear();
            for _ in 0..num_parents {
                let p = LReq::perpendicular_acc(&child_refs);
                parents.push(p);
            }
        });
    }
}
