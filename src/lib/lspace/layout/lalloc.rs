use geom::fastminmax::{fast_min, fast_max};
use layout::lreq::{LNatSize, LFlex, LReq};

pub const EPSILON : f64 = 1.0e-6;
pub const ONE_MINUS_EPSILON : f64 = 1.0 - EPSILON;
pub const ONE_PLUS_EPSILON : f64 = 1.0 + EPSILON;


#[derive(Debug, PartialEq, Copy, Clone)]
pub struct LAlloc {
    pos_in_parent: f64,
    alloc_size: f64,
    actual_size: f64,
    ref_point: Option<f64>
}


/// Linear allocation helper function
/// Allocate child elements their requested size, no flexibility
///
/// Parameters:
/// `n` : the number of child elements
/// `child_reqs` : child space requisitions
/// `child_allocs` : child allocations
/// `start_pos` : the position of the first child
/// `space_between` : optional amount of space between each child
///
/// Returns:
/// The position of a subsequent child
fn alloc_children_linear_fixed(n: usize,
                               child_reqs_and_allocs: &mut [(&LReq, &mut LAlloc)],
                               start_pos: f64, space_between: f64) -> f64 {
    let mut pos = start_pos;
    for i in 0..n {
        let creq = child_reqs_and_allocs[i].0;
        let mut calloc: &mut LAlloc = child_reqs_and_allocs[i].1;
        let csz = creq.size().size();
        calloc.alloc_from_region(creq, pos, csz, creq.size().before_ref_opt());
        pos = pos + csz + space_between;
    }
    return pos;
}

/// Allocation helper function
/// Allocate child elements using flexibility
///
/// Parameters:
/// `n` : the number of child elements
/// `child_reqs` : child space requisitions
/// `child_allocs` : child allocations
/// `start_pos` : the position of the first child
/// `space_between` : optional amount of space between each child
/// `shrink_frac` : multiply a child's shrink space by this fraction to get the amount to shrink
/// the child element by
/// `stretch_factor` : multiply a child's stretchiness by this factor to get the amount to stretch
/// the child element by
///
/// Returns:
/// The position of a subsequent child
fn alloc_children_linear_flex(n: usize,
                              child_reqs_and_allocs: &mut [(&LReq, &mut LAlloc)],
                              start_pos: f64, space_between: f64,
                              shrink_frac: f64, stretch_factor: f64) -> f64 {
    let mut pos = start_pos;
    for i in 0..n {
        let creq = child_reqs_and_allocs[i].0;
        let mut calloc: &mut LAlloc = child_reqs_and_allocs[i].1;
        let cshrink = creq.flex().shrink() * shrink_frac;
        let cstretch = (creq.flex().stretch() as f64) * stretch_factor;

        let sz = creq.size().size();
        let csz = creq.size().size() - cshrink + cstretch;
        let cref = match creq.size().before_ref_opt() {
            None => None,
            Some(before_ref) => Some(before_ref + (cstretch - cshrink) * before_ref / sz),
        };

        calloc.alloc_from_region(creq, pos, csz, cref);
        pos = pos + csz + space_between;
    }
    return pos;
}

/// Allocation helper function
/// Compute flexibility factors for the region. Given the region allocated size, its minimum and
/// natural size and its accumulated stretchiness (sum of stretchiness of children) this function
/// will determine if child elements should shrink or stretch and by how much.
///
/// Parameters:
/// `region_size` : the amount of space allocated to the region in which child elements are being
/// placed
/// `minimum_size` : the minimum size of the region
/// `natural_size` : the natural size of the region
/// `region_stretch` : sum of stretchiness of children
///
/// Returns:
/// None : No flexibility required
/// Some((shrink_fraction, stretch_factor)) : a tuple of the shrink fraction (fraction of shrink
/// space to apply for each child) and the stretch factor (multiply a child's stretchiness by
/// this factor to compute amount by which it should be stretched).
fn compute_flex_factors(region_size: f64, minimum_size: f64, natural_size: f64,
                        region_stretch: f32) -> Option<(f64, f64)> {
    if region_size < minimum_size * ONE_MINUS_EPSILON {
        // Insufficient space; allocate minimum space; shrink_frac=1.0
        return Some((1.0, 0.0));
    } else if region_size < natural_size * ONE_MINUS_EPSILON {
        // Region_size is between minimum and natural size
        let shrink_frac = (region_size - natural_size) /
                (minimum_size - natural_size);
        return Some((shrink_frac, 0.0));
    } else if region_size > natural_size * ONE_PLUS_EPSILON {
        let stretch_space = region_size - natural_size;

        if region_stretch > 0.0 {
            // Region size is greater than the natural size and stretch > 0
            let stretch_factor = stretch_space / region_stretch as f64;
            return Some((0.0, stretch_factor));
        } else {
            // This element won't stretch
            return None;
        }
    } else {
        // Region size matches natural size
        return None;
    }
}

/// Allocation helper function
/// Allocate child elements in a linear fashion
fn alloc_children_linear(n: usize, child_reqs_and_allocs: &mut [(&LReq, &mut LAlloc)],
                  natural_size: f64, minimum_size: f64, region_pos: f64, region_size: f64,
                  region_stretch: f32, space_between: f64, pad_start: bool) -> f64 {
    match compute_flex_factors(region_size, minimum_size, natural_size, region_stretch) {
        Some((shrink_frac, stretch_frac)) => {
            let start = if pad_start && region_stretch == (0.0 as f32) {
                let shrink = natural_size - minimum_size;
                let shrunk_size = natural_size - shrink * shrink_frac;
                let start_padding = region_size - shrunk_size;
                region_pos + start_padding
            } else {
                region_pos
            };
            alloc_children_linear_flex(n, child_reqs_and_allocs, start, space_between,
                                       shrink_frac, stretch_frac)
        },

        None => {
            let start = if pad_start {
                let start_padding = region_size - natural_size;
                region_pos + start_padding
            } else {
                region_pos
            };
            alloc_children_linear_fixed(n, child_reqs_and_allocs, start, space_between)
        }
    }
}

/// Allocation helper function
/// Allocate child elements in a linear fashion, region has NO reference point
fn alloc_children_linear_no_ref(n: usize, child_reqs_and_allocs: &mut [(&LReq, &mut LAlloc)],
                                region_req: &LReq, region_pos: f64, region_size: f64,
                                space_between: f64) {
    match region_req.flex() {
        &LFlex::Fixed => {
            alloc_children_linear_fixed(n, child_reqs_and_allocs, region_pos,
                                        space_between);
        },
        &LFlex::Flex{..} => {
            let natural_size = region_req.size().size();
            let minimum_size = natural_size - region_req.flex().shrink();

            alloc_children_linear(n, child_reqs_and_allocs,
                                  natural_size, minimum_size,
                                  region_pos, region_size,
                                  region_req.flex().stretch(),
                                  space_between, false);
        },
    };
}

/// Allocation helper function
/// Allocate child elements in a linear fashion, region has reference point
fn alloc_children_linear_with_ref(n: usize, child_reqs_and_allocs: &mut [(&LReq, &mut LAlloc)],
                                  region_req: &LReq, region_pos: f64, region_size: f64,
                                  region_before_ref: f64, space_between: f64,
                                  ref_point_n: usize) {
    match region_req.flex() {
        &LFlex::Fixed => {
            // Compute the offset required to position the array of children so that
            // the required reference point aligns with the allocated reference point.
            // Assuming that the region requisition has its reference point
            // aligned with that of the correct child, this should be fine...
            let before_offset = fast_max(region_before_ref - region_req.size().before_ref(), 0.0);
            alloc_children_linear_fixed(n, child_reqs_and_allocs,
                                        before_offset + region_pos, space_between);
        },
        &LFlex::Flex{..} => {
            let req_at_ref_n = child_reqs_and_allocs[ref_point_n].0;

            let ref_child_shrink = req_at_ref_n.flex().shrink();
            let ref_child_stretch = req_at_ref_n.flex().stretch();
            let ref_child_before_ref_frac = req_at_ref_n.size().fractional_ref_point();
            let ref_child_after_ref_frac = 1.0 - ref_child_before_ref_frac;

            // We have to allocate space in two distinct pieces; the piece before the reference
            // point and the piece after

            // Before reference point index
            let (stretch_before_ref, nat_size_before_ref,
                        min_size_before_ref, region_size_before_ref,
                        spill_before_ref) = {
                let (shrink, stretch) = if ref_point_n == 0 {
                    (ref_child_shrink * ref_child_before_ref_frac,
                     ref_child_stretch * ref_child_before_ref_frac as f32)
                } else {
                    let reqs_and_allocs = &mut child_reqs_and_allocs[..ref_point_n];

                    let shrink = reqs_and_allocs.iter().fold(0.0,
                            |acc, x| acc + x.0.flex().shrink()) +
                                ref_child_shrink * ref_child_before_ref_frac;
                    let stretch = reqs_and_allocs.iter().fold(0.0,
                            |acc, x| acc + x.0.flex().stretch()) +
                                ref_child_stretch * ref_child_before_ref_frac as f32;

                    (shrink, stretch)
                };

                let natural_size = region_req.size().before_ref();
                let minimum_size = natural_size - shrink;
                let avail_size = region_before_ref;
                // The spill is the additional space required to reach the minimum
                // required size.
                let spill = fast_max(minimum_size - avail_size, 0.0);

                (stretch, natural_size, minimum_size, avail_size, spill)
            };

            // After reference point index
            let (stretch_after_ref, nat_size_after_ref,
                        min_size_after_ref, region_size_after_ref) = {
                let (shrink, stretch) = if ref_point_n >= n-1 {
                    (ref_child_shrink * ref_child_after_ref_frac,
                     ref_child_stretch * ref_child_after_ref_frac as f32)
                } else {
                    let reqs_and_allocs = &child_reqs_and_allocs[ref_point_n+1..];

                    let shrink = reqs_and_allocs.iter().fold(0.0,
                            |acc, x| acc + x.0.flex().shrink()) +
                                ref_child_shrink * ref_child_after_ref_frac;
                    let stretch = reqs_and_allocs.iter().fold(0.0,
                            |acc, x| acc + x.0.flex().stretch()) +
                                ref_child_stretch * ref_child_after_ref_frac as f32;

                    (shrink, stretch)
                };

                let natural_size = region_req.size().after_ref();
                let minimum_size = natural_size - shrink;
                // take into account any spill over from before the ref point
                let avail_size = region_size - region_before_ref - spill_before_ref;

                (stretch, natural_size, minimum_size, avail_size)
            };

            // Start position at `region_pos`
            let mut pos = region_pos;

            // Allocate children before ref point
            if ref_point_n > 0 {
                pos = alloc_children_linear(ref_point_n,
                                            &mut child_reqs_and_allocs[..ref_point_n],
                                            nat_size_before_ref, min_size_before_ref,
                                            pos, region_size_before_ref,
                                            stretch_before_ref,
                                            space_between, true);
            }

            // Allocate child aligned with ref point
            // Before reference point: compute flex factors and apply
            let mut ref_child_before_ref = req_at_ref_n.size().before_ref();
            match compute_flex_factors(region_size_before_ref, min_size_before_ref,
                                       nat_size_before_ref, stretch_before_ref) {
                Some((shrink_frac, stretch_factor)) => {
                    ref_child_before_ref = ref_child_before_ref -
                        req_at_ref_n.flex().shrink() * shrink_frac * ref_child_before_ref_frac +
                        req_at_ref_n.flex().stretch() as f64 * stretch_factor * ref_child_before_ref_frac;
                },
                None => {},
            };

            // After reference point: compute flex factors and apply
            let mut ref_child_after_ref = req_at_ref_n.size().after_ref();
            match compute_flex_factors(region_size_after_ref, min_size_after_ref,
                                       nat_size_after_ref, stretch_after_ref) {
                Some((shrink_frac, stretch_factor)) => {
                    ref_child_after_ref = ref_child_after_ref -
                        req_at_ref_n.flex().shrink() * shrink_frac * ref_child_after_ref_frac +
                        req_at_ref_n.flex().stretch() as f64 * stretch_factor * ref_child_after_ref_frac;
                },
                None => {},
            };

            // Get the total size
            let ref_child_sz = ref_child_before_ref + ref_child_after_ref;
            // If the child aligned with the ref point is the first child, then the position
            // won't have been incremented by calling `alloc_children_linear`, since it is
            // not called if `ref_point_n == 0`. Reset the position so that the child 0's
            // ref point aligns with the region reference point.
            if ref_point_n == 0 {
                pos = region_pos + region_before_ref - ref_child_before_ref;
            }
            // Allocate the child aligned with ref point
            child_reqs_and_allocs[ref_point_n].1.alloc_from_region(req_at_ref_n,
                pos, ref_child_sz, Some(ref_child_before_ref));
            // Advance position
            pos = pos + ref_child_sz + space_between;

            // Allocate children after reference point
            alloc_children_linear(n - (ref_point_n + 1),
                                  &mut child_reqs_and_allocs[ref_point_n+1..],
                                  nat_size_after_ref, min_size_after_ref,
                                  pos, region_size_after_ref,
                                  stretch_after_ref,
                                  space_between, false);
        },
    }
}


/// A representation of a space allocation; space that has been allocated to an element.
impl LAlloc {
    /// Construct an empty `LAlloc`.
    pub fn new_empty() -> LAlloc {
        return LAlloc{pos_in_parent: 0.0, alloc_size: 0.0, actual_size: 0.0, ref_point: None};
    }

    /// Consruct a `LAlloc`
    pub fn new(pos_in_parent: f64, alloc_size: f64, actual_size: f64) -> LAlloc {
        return LAlloc{pos_in_parent: pos_in_parent, alloc_size: alloc_size,
                      actual_size: actual_size, ref_point: None};
    }

    /// Consruct a `LAlloc`
    pub fn new_ref(pos_in_parent: f64, alloc_size: f64, actual_size: f64,
                   ref_point: f64) -> LAlloc {
        return LAlloc{pos_in_parent: pos_in_parent, alloc_size: alloc_size,
                      actual_size: actual_size, ref_point: Some(ref_point)};
    }

    /// Construct a `LAlloc` that has enough space for the given `LReq` requisition
    pub fn new_from_req(req: &LReq, pos_in_parent: f64) -> LAlloc {
        let size = req.size().size();
        return LAlloc{pos_in_parent: pos_in_parent, alloc_size: size, actual_size: size,
                      ref_point: req.size().before_ref_opt()};
    }

    /// Construct a `LAlloc` that has enough space for the given `LReq` requisition
    pub fn new_from_req_in_avail_size(req: &LReq, pos_in_parent: f64, avail_size: f64) -> LAlloc {
        let req_size = req.size().size();
        let alloc_size = fast_min(req_size, avail_size);
        return LAlloc{pos_in_parent: pos_in_parent, alloc_size: alloc_size, actual_size: req_size,
                      ref_point: req.size().before_ref_opt()};
    }


    /// Get position of this element relative to parent space.
    pub fn pos_in_parent(&self) -> f64 {
        return self.pos_in_parent;
    }

    /// The the amount of space allocated to this element.
    pub fn alloc_size(&self) -> f64 {
        return self.alloc_size;
    }

    /// The the element's actual size; in cases where not enough space was available and this
    /// element was allocated less than it requires, the actual size is the space required to
    /// display it; use this for building bounding boxes for the purpose of event processing
    /// and culling during rendering.
    pub fn actual_size(&self) -> f64 {
        return self.actual_size;
    }

    /// Get the optional reference point position, as an offset from the position relative
    /// to parent space
    pub fn ref_point(&self) -> Option<f64> {
        return self.ref_point;
    }


    /// Private helper: update all fields, setting the reference point to `None`
    fn update(&mut self, pos_in_parent: f64, alloc_size: f64, actual_size: f64) {
        self.pos_in_parent = pos_in_parent;
        self.alloc_size = alloc_size;
        self.actual_size = actual_size;
        self.ref_point = None;
    }

    /// Private helper: update all fields, setting the reference point to `Some(ref_point)`
    fn update_ref(&mut self, pos_in_parent: f64, alloc_size: f64, actual_size: f64,
                  ref_point: f64) {
        self.pos_in_parent = pos_in_parent;
        self.alloc_size = alloc_size;
        self.actual_size = actual_size;
        self.ref_point = Some(ref_point);
    }

    /// Indent - consume the requested amount of space from the start.
    /// Note that it effectively does the opposite to the `LReq` method of the same name
    pub fn indent(&self, indent: f64) -> LAlloc {
        return LAlloc{pos_in_parent: self.pos_in_parent + indent,
                      alloc_size: self.alloc_size - indent,
                      actual_size: self.actual_size - indent,
                      ref_point: match self.ref_point {
                            Some(r) => Some(r - indent),
                            None => None
                        }};
    }

    /// Copy without position
    pub fn without_position(&self) -> LAlloc {
        return LAlloc{pos_in_parent: 0.0, alloc_size: self.alloc_size,
                      actual_size: self.actual_size, ref_point: self.ref_point};
    }

    /// Allocate this element - a child element - space in the specified region that would be
    /// contained within the space of a parent element.
    ///
    /// Allocates the element represented by `self` space from the given region.
    ///
    /// Parameters:
    /// `child_req` : the space requirements of the element represented by `self`; an instance of `LReq`
    /// `region_pos` : the position of the region in the parent space; most elements that have
    /// multiple children will want to give them different positions
    /// `region_size` : the size of the region
    /// `region_ref` : the reference point of the region (optional)
    pub fn alloc_from_region(&mut self, child_req: &LReq, region_pos: f64, region_size: f64,
                             region_ref: Option<f64>) {
        match child_req.size().before_and_after_ref_opt() {
            // Both the allocation region and the requisition have a reference point
            Some((req_before, _)) => {
                // If the region's reference point is not aligned with the required
                // reference point, compute the offset
                let (ref_offset, alloc_before) = match region_ref {
                    Some(avail_ref) => {
                        if child_req.flex().stretch() == 0.0 {
                            (fast_max(avail_ref - req_before, 0.0), avail_ref)
                        } else {
                            (0.0, avail_ref)
                        }
                    },
                    None => (0.0, req_before),
                };

                // Compute the natural and minimum sizes and end points
                // End points are the sizes that include the offset that aligns the required
                // reference point with the region reference point
                let natural_size = child_req.size().size();
                let minimum_size = natural_size - child_req.flex().shrink();
                let natural_end = natural_size + ref_offset;
                let minimum_end = minimum_size + ref_offset;

                if region_size < minimum_end * ONE_MINUS_EPSILON {
                    // Insufficient space
                    self.update_ref(ref_offset + region_pos, region_size - ref_offset,
                                    minimum_size, alloc_before - ref_offset);
                } else if region_size > natural_end * ONE_PLUS_EPSILON {
                    // More space than natural size
                    if child_req.flex().stretch() > 0.0 {
                        // Stretch to take up additional space
                        self.update_ref(region_pos, region_size, region_size, alloc_before);
                    } else {
                        // No stretch; don't take up additional space
                        self.update_ref(ref_offset + region_pos, natural_size, natural_size,
                                        alloc_before - ref_offset);
                    }
                } else {
                    // region_size is between minimum and natural end
                    let alloc_size = region_size - ref_offset;
                    self.update_ref(ref_offset + region_pos, alloc_size, alloc_size,
                                    alloc_before - ref_offset);
                }
            },

            // No reference point required
            None => {
                let natural_size = child_req.size().size();
                let minimum_size = natural_size - child_req.flex().shrink();

                if region_size < minimum_size * ONE_MINUS_EPSILON {
                    // Insufficient space; allocate the available space, while giving
                    // `minimum_size` as the actual size for bounding box and rendering purposes
                    self.update(region_pos, region_size, minimum_size);
                } else if region_size < natural_size * ONE_PLUS_EPSILON ||
                          child_req.flex().stretch() > 0.0 {
                    // Either:
                    // - region_size is between minimum and natural size
                    // or
                    // - region_size is > natural size and the box can stretch to use the
                    // additional space
                    //
                    // Allocate all the available space
                    self.update(region_pos, region_size, region_size);
                } else {
                    // region_size is > natural and no stretch; don't take up additional space
                    self.update(region_pos, natural_size, natural_size);
                }
            }
        }
    }


    /// Allocate space to an array of child elements that are aligned along the axis represented
    /// by the requisitions and allocations.
    ///
    /// Parameters:
    /// 'child_reqs' : a vector containing an `LReq` for each child element
    /// 'child_allocs' : a vector containing an `LAlloc` for each child element
    /// 'region_req' : the `LReq` for the region within the parent element
    /// `region_pos`, `region_size`, `region_ref` : the position, size and optional reference
    /// point that describe the region in which the child elements are to be positioned
    /// `space_between` : the amount of space to insert between child elements
    /// `ref_point_index` : an optional index that identifies a child element that is to have its
    /// reference point aligned with that of the region
    pub fn alloc_linear(child_reqs_and_allocs: &mut [(&LReq, &mut LAlloc)],
                        region_req: &LReq, region_pos: f64, region_size: f64,
                        region_ref: Option<f64>, space_between: f64,
                        ref_point_index: Option<usize>) {
        let n = child_reqs_and_allocs.len();

        // There are a few valid, consistent configurations that are identified and handled
        match (ref_point_index, region_req.size(), region_ref) {
            (None, &LNatSize::Size{..}, None) => {
                // NO REFERENCE POINTS
                alloc_children_linear_no_ref(n, child_reqs_and_allocs, region_req, region_pos,
                                             region_size, space_between);
            },

            (Some(ref_point_n), &LNatSize::Ref{..}, Some(region_before_ref)) => {
                debug_assert!(ref_point_n < n);
                // REFERENCE POINT INDEX SPECIFIED
                // REQUISITION HAS REF POINT
                // REGION HAS REF POINT

                alloc_children_linear_with_ref(n, child_reqs_and_allocs, region_req,
                                               region_pos, region_size, region_before_ref,
                                               space_between, ref_point_n);
            },

            // EMPTY PARENT REQUISITIONS
            (_, &LNatSize::Empty, None) => {
                // Ensure children are all empty, otherwise we have a bug. Nothing else to do.
                for (i, ch) in child_reqs_and_allocs.iter().enumerate() {
                    if ch.0.size() != &LNatSize::Empty {
                        panic!("Empty parent requisition, non-empty child requisiton at \
                               index {}", i);
                    }
                }
            },

            (_, &LNatSize::Empty, _) => {
                panic!("Empty parent requisition but region allocation has reference point");
            }

            // HANDLE INCONSISTENT CONFIGURATIONS BY IDENTIFYING THEM AND PANICKING
            (_, _, _) => {
                let a = match ref_point_index {
                    None => "NOT PRESENT",
                    Some(_) => "PRESENT",
                };
                let b = match region_req.size() {
                    &LNatSize::Empty => "NOT PRESENT",
                    &LNatSize::Size{..} => "NOT PRESENT",
                    &LNatSize::Ref{..} => "PRESENT",
                };
                let c = match region_ref {
                    None => "NOT PRESENT",
                    Some(_) => "PRESENT",
                };
                panic!("INCONSISTENT CONFIGURATION: ref point index {}, region requisition ref \
                        point {}, region allocation ref point {}", a, b, c);
            },
        };
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
    use geom::fastminmax::{fast_min, fast_max};
    use layout::lreq::{LNatSize, LFlex, LReq};

    const ALMOST_EQ_EPSILON: f64 = 1.0e-6;

    macro_rules! assert_almost_eq {
        ($x:expr, $y:expr) => (
            if ($x).abs_sub($y) > ALMOST_EQ_EPSILON {
                panic!("assert_almost_eq failed: {} !=~ {}", $x, $y);
            }
        );

    }

    // Test helper functions

    fn alloc_linear(region_pos: f64, region_size: f64, region_ref: Option<f64>,
                    child_reqs: &Vec<&LReq>, space_between: f64,
                    ref_point_index: Option<usize>) -> (LReq, Vec<LAlloc>){
        // Compute the region requisition
        let region_req = LReq::linear_acc(child_reqs, space_between, ref_point_index);

        // Create allocs for the children
        let mut child_allocs : Vec<LAlloc> = child_reqs.iter().map(|_| LAlloc::new_empty()).collect();

        {

            let mut child_reqs_and_allocs: Vec<(&LReq, &mut LAlloc)> = Vec::with_capacity(child_reqs.len());
            for (r, mut a) in child_reqs.iter().zip(child_allocs.iter_mut()) {
                child_reqs_and_allocs.push((r, a));
            }
            LAlloc::alloc_linear(&mut child_reqs_and_allocs, &region_req,
                                 region_pos, region_size, region_ref, space_between,
                                 ref_point_index);
        }

        return (region_req, child_allocs);
    }

    fn stretch_prop(stretch: f64, total_stretch: f64) -> f64 {
        if total_stretch == 0.0 {
            return 0.0;
        } else {
            return stretch / total_stretch;
        }
    }

    fn alloc_linear_ref_test(child_reqs: &Vec<&LReq>, shrink_frac_before: f64,
                             shrink_frac_after: f64, additional_before: f64,
                             additional_after: f64) {
        if shrink_frac_before > 0.0 && additional_before > 0.0 {
            panic!("For alloc_linear_ref_test to be valid, the space before the ref point must \
                   either shrink (shrink_frac_before > 0) or stretch (additional_before > 0), \
                   not both.");
        }
        if shrink_frac_after > 0.0 && additional_after > 0.0 {
            panic!("For alloc_linear_ref_test to be valid, the space before the ref point must \
                   either shrink (shrink_frac_after > 0) or stretch (additional_after > 0), \
                   not both.");
        }
        let n = child_reqs.len();
        for region_pos in vec![0.0, 100.0] {
            for space_between in vec![0.0, 10.0] {
                for ref_point_index in 0..n {
                    // Assume that `LReq::linear_acc` is thoroughly tested and working
                    // correctly.
                    // Use it to compute the amount of space required by the child elements.
                    let region_req = LReq::linear_acc(child_reqs, space_between,
                        Some(ref_point_index));
                    let region_req_no_space = LReq::linear_acc(child_reqs, 0.0,
                        Some(ref_point_index));

                    // Compute the fractional position of the reference point
                    let region_before_frac = region_req_no_space.size().fractional_ref_point();
                    let region_after_frac = 1.0 - region_before_frac;

                    // Compute the size that we will allocate to the region
                    let shrink = region_req.flex().shrink();
                    let shrink_before = shrink_frac_before * shrink * region_before_frac;
                    let shrink_after = shrink_frac_after * shrink * region_after_frac;
                    let stretch = region_req.flex().stretch();
                    // let stretch_before = stretch as f64 * region_before_frac;
                    // let stretch_after = stretch as f64 * region_after_frac;
                    let space_before = region_req.size().before_ref() -
                            shrink_before + additional_before;
                    let space_after = region_req.size().after_ref() -
                            shrink_after + additional_after;

                    let offset = if stretch == 0.0 {
                        region_pos + additional_before
                    } else {
                        region_pos
                    };

                    // Allocate the boxes
                    let (req, allocs) = alloc_linear(region_pos, space_before + space_after,
                                                     Some(space_before), child_reqs, space_between,
                                                     Some(ref_point_index));

                    let valid_shrink_frac_before = fast_min(shrink_frac_before, 1.0);
                    let valid_shrink_frac_after = fast_min(shrink_frac_after, 1.0);

                    // For the child aligned with the reference point, get the fractional
                    // reference point, the reference point and the shrink and stretch factors
                    let refch_frac_before =
                            child_reqs[ref_point_index].size().fractional_ref_point();
                    let refch_frac_after = 1.0 - refch_frac_before;
                    let refch_before = child_reqs[ref_point_index].size().before_ref();
                    let refch_after = child_reqs[ref_point_index].size().after_ref();
                    let refch_shrink = child_reqs[ref_point_index].flex().shrink();
                    let refch_stretch = child_reqs[ref_point_index].flex().stretch();

                    // Compute the allocation of shrink and stretch before and after the ref point
                    let mut shrink_before_ref = 0.0;
                    let mut shrink_after_ref = 0.0;
                    let mut stretch_before_ref = 0.0;
                    let mut stretch_after_ref = 0.0;
                    for i in 0..n {
                        if i < ref_point_index {
                            shrink_before_ref += child_reqs[i].flex().shrink();
                            stretch_before_ref += child_reqs[i].flex().stretch() as f64;
                        } else if i > ref_point_index {
                            shrink_after_ref += child_reqs[i].flex().shrink();
                            stretch_after_ref += child_reqs[i].flex().stretch() as f64;
                        } else {
                            shrink_before_ref += child_reqs[i].flex().shrink() * refch_frac_before;
                            stretch_before_ref +=
                                    child_reqs[i].flex().stretch() as f64 * refch_frac_before;
                            shrink_after_ref += child_reqs[i].flex().shrink() * refch_frac_after;
                            stretch_after_ref +=
                                    child_reqs[i].flex().stretch() as f64 * refch_frac_after;
                        }
                    }

                    // Compute the allocation of space to the child aligned with the reference
                    // point
                    let refch_alloc_shrink_before =
                            refch_shrink * refch_frac_before * valid_shrink_frac_before;
                    let refch_alloc_stretch_before = additional_before *
                        stretch_prop(refch_stretch as f64 * refch_frac_before,
                                     stretch_before_ref as f64);
                    let refch_alloc_shrink_after =
                            refch_shrink * refch_frac_after * valid_shrink_frac_after;
                    let refch_alloc_stretch_after = additional_after *
                        stretch_prop(refch_stretch as f64 * refch_frac_after,
                                     stretch_after_ref as f64);

                    let refch_alloc_before = refch_before - refch_alloc_shrink_before +
                            refch_alloc_stretch_before;
                    let refch_alloc_after = refch_after - refch_alloc_shrink_after +
                            refch_alloc_stretch_after;
                    let refch_alloc_size = refch_alloc_before + refch_alloc_after;

                    if child_reqs[ref_point_index].size().has_ref_point() {
                        // Ensure that the child aligned with the reference point has the
                        // correct reference point
                        assert_almost_eq!(allocs[ref_point_index].ref_point.unwrap(),
                                          refch_alloc_before);
                    }
                    // Ensure that it has been allocated the correct size
                    assert_almost_eq!(allocs[ref_point_index].alloc_size, refch_alloc_size);
                    // Ensure that its actual size is not less than the minimum
                    assert_almost_eq!(allocs[ref_point_index].actual_size,
                            fast_max(refch_alloc_size,
                                child_reqs[ref_point_index].size().size() - refch_shrink));

                    // Work backwards from the ref-point
                    let mut before_pos = space_before - refch_alloc_before;
                    // Check its position
                    assert_almost_eq!(allocs[ref_point_index].pos_in_parent, offset + before_pos);
                    before_pos = before_pos - space_between;
                    for i in (0..ref_point_index).rev() {
                        let ch_ref_frac = child_reqs[i].size().fractional_ref_point();
                        let ch_shrink = child_reqs[i].flex().shrink() * valid_shrink_frac_before;
                        let ch_stretch = additional_before *
                                stretch_prop(child_reqs[i].flex().stretch() as f64,
                                            stretch_before_ref as f64);
                        let ch_min_size = child_reqs[i].size().size() -
                                child_reqs[i].flex().shrink();
                        let ch_size = child_reqs[i].size().size() - ch_shrink + ch_stretch;
                        let ch_before_ref = child_reqs[i].size().before_ref() -
                            ch_shrink * ch_ref_frac + ch_stretch * ch_ref_frac;
                        before_pos = before_pos - ch_size;
                        assert_almost_eq!(allocs[i].pos_in_parent, offset + before_pos);
                        assert_almost_eq!(allocs[i].alloc_size, ch_size);
                        assert_almost_eq!(allocs[i].actual_size, fast_max(ch_size, ch_min_size));
                        if child_reqs[i].size().has_ref_point() {
                            assert_almost_eq!(allocs[i].ref_point.unwrap(), ch_before_ref);
                        }
                        before_pos = before_pos - space_between;
                    }
                    // Check that we have got back to the start point
                    assert_almost_eq!(before_pos, offset);

                    // Work forwards from the ref-point
                    let mut after_pos = space_before + refch_alloc_after + space_between;
                    for i in (ref_point_index+1..n) {
                        let ch_ref_frac = child_reqs[i].size().fractional_ref_point();
                        let ch_shrink = child_reqs[i].flex().shrink() * valid_shrink_frac_after;
                        let ch_stretch = additional_after *
                                stretch_prop(child_reqs[i].flex().stretch() as f64,
                                stretch_after_ref as f64);
                        let ch_min_size = child_reqs[i].size().size() -
                                child_reqs[i].flex().shrink();
                        let ch_size = child_reqs[i].size().size() - ch_shrink + ch_stretch;
                        let ch_before_ref = child_reqs[i].size().before_ref() -
                            ch_shrink * ch_ref_frac + ch_stretch * ch_ref_frac;
                        assert_almost_eq!(allocs[i].pos_in_parent, offset + after_pos);
                        assert_almost_eq!(allocs[i].alloc_size, ch_size);
                        assert_almost_eq!(allocs[i].actual_size, fast_max(ch_size, ch_min_size));
                        if child_reqs[i].size().has_ref_point() {
                            assert_almost_eq!(allocs[i].ref_point.unwrap(), ch_before_ref);
                        }
                        after_pos = after_pos + ch_size + space_between;
                    }
                }
            }
        }
    }


    //
    //
    // LAlloc
    //
    //
    #[test]
    fn test_lalloc_new_and_accessors() {
        let a = LAlloc::new_empty();
        assert_eq!(a.pos_in_parent(), 0.0);
        assert_eq!(a.alloc_size(), 0.0);
        assert_eq!(a.actual_size(), 0.0);
        assert_eq!(a.ref_point(), None);

        let b = LAlloc::new(1.0, 10.0, 20.0);
        assert_eq!(b.pos_in_parent(), 1.0);
        assert_eq!(b.alloc_size(), 10.0);
        assert_eq!(b.actual_size(), 20.0);
        assert_eq!(b.ref_point(), None);

        let c = LAlloc::new_ref(1.0, 10.0, 20.0, 5.0);
        assert_eq!(c.pos_in_parent(), 1.0);
        assert_eq!(c.alloc_size(), 10.0);
        assert_eq!(c.actual_size(), 20.0);
        assert_eq!(c.ref_point(), Some(5.0));
    }

    #[test]
    fn test_lalloc_mem() {
        assert_eq!(mem::size_of::<LAlloc>(), 40);
    }

    #[test]
    fn test_lalloc_update() {
        let mut a = LAlloc::new_empty();
        a.update(1.0, 10.0, 20.0);
        assert_eq!(a.pos_in_parent(), 1.0);
        assert_eq!(a.alloc_size(), 10.0);
        assert_eq!(a.actual_size(), 20.0);
        assert_eq!(a.ref_point(), None);

        let mut b = LAlloc::new_empty();
        b.update_ref(1.0, 10.0, 20.0, 5.0);
        assert_eq!(b.pos_in_parent(), 1.0);
        assert_eq!(b.alloc_size(), 10.0);
        assert_eq!(b.actual_size(), 20.0);
        assert_eq!(b.ref_point(), Some(5.0));
    }

    #[test]
    fn test_lalloc_indent() {
        assert_eq!(LAlloc::new(0.0, 8.0, 10.0).indent(2.0),
                   LAlloc::new(2.0, 6.0, 8.0));
        assert_eq!(LAlloc::new_ref(0.0, 8.0, 10.0, 4.0).indent(2.0),
                   LAlloc::new_ref(2.0, 6.0, 8.0, 2.0));
    }

    #[test]
    fn test_lalloc_without_position() {
        assert_eq!(LAlloc::new(100.0, 8.0, 10.0).without_position(),
                   LAlloc::new(0.0, 8.0, 10.0));
        assert_eq!(LAlloc::new_ref(100.0, 8.0, 10.0, 4.0).without_position(),
                   LAlloc::new_ref(0.0, 8.0, 10.0, 4.0));
    }

    fn _alloc_region(req: &LReq, region_pos: f64, region_size: f64) -> LAlloc {
        let mut a = LAlloc::new_empty();
        a.alloc_from_region(req, region_pos, region_size, None);
        return a;
    }

    fn _alloc_region_ref(req: &LReq, region_pos: f64, region_size: f64,
                         region_ref: f64) -> LAlloc {
        let mut a = LAlloc::new_empty();
        a.alloc_from_region(req, region_pos, region_size, Some(region_ref));
        return a;
    }

    #[test]
    fn test_alloc_from_region() {
        //
        // FIXED SIZE
        //
        // Fixed size, too small
        assert_eq!(_alloc_region(
            &LReq::new_fixed_size(10.0), 0.0, 5.0),
            LAlloc::new(0.0, 5.0, 10.0));
        // Fixed size, too small with ref
        assert_eq!(_alloc_region_ref(
            &LReq::new_fixed_size(10.0), 0.0, 5.0, 2.5),
            LAlloc::new(0.0, 5.0, 10.0));

        // Fixed size, natural size
        assert_eq!(_alloc_region(
            &LReq::new_fixed_size(10.0), 0.0, 10.0),
            LAlloc::new(0.0, 10.0, 10.0));
        // Fixed size, natural size with ref
        assert_eq!(_alloc_region_ref(
            &LReq::new_fixed_size(10.0), 0.0, 10.0, 6.0),
            LAlloc::new(0.0, 10.0, 10.0));

        // Fixed size, extra space
        assert_eq!(_alloc_region(
            &LReq::new_fixed_size(10.0), 0.0, 20.0),
            LAlloc::new(0.0, 10.0, 10.0));
        // Fixed size, extra space with ref
        assert_eq!(_alloc_region_ref(
            &LReq::new_fixed_size(10.0), 0.0, 20.0, 12.0),
            LAlloc::new(0.0, 10.0, 10.0));

        //
        // FLEXIBLE SIZE WITH SHRINK
        //
        // Flexible size with shrink, too small
        assert_eq!(_alloc_region(
            &LReq::new_flex_size(10.0, 5.0, 0.0), 0.0, 4.0),
            LAlloc::new(0.0, 4.0, 5.0));
        // Flexible size with shrink, too small with ref
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_size(10.0, 5.0, 0.0), 0.0, 4.0, 2.5),
            LAlloc::new(0.0, 4.0, 5.0));

        // Flexible size with shrink, minimum size
        assert_eq!(_alloc_region(
            &LReq::new_flex_size(10.0, 5.0, 0.0), 0.0, 5.0),
            LAlloc::new(0.0, 5.0, 5.0));
        // Flexible size with shrink, minimum size with ref
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_size(10.0, 5.0, 0.0), 0.0, 5.0, 3.0),
            LAlloc::new(0.0, 5.0, 5.0));

        // Flexible size with shrink, between minimum and natural size
        assert_eq!(_alloc_region(
            &LReq::new_flex_size(10.0, 5.0, 0.0), 0.0, 7.0),
            LAlloc::new(0.0, 7.0, 7.0));
        // Flexible size with shrink, between minimum and natural size with ref
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_size(10.0, 5.0, 0.0), 0.0, 7.0, 4.0),
            LAlloc::new(0.0, 7.0, 7.0));

        // Flexible size with shrink, natural size
        assert_eq!(_alloc_region(
            &LReq::new_flex_size(10.0, 5.0, 0.0), 0.0, 10.0),
            LAlloc::new(0.0, 10.0, 10.0));
        // Flexible size with shrink, natural size with ref
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_size(10.0, 5.0, 0.0), 0.0, 10.0, 6.0),
            LAlloc::new(0.0, 10.0, 10.0));

        // Flexible size with shrink, extra space
        assert_eq!(_alloc_region(
            &LReq::new_flex_size(10.0, 5.0, 0.0), 0.0, 20.0),
            LAlloc::new(0.0, 10.0, 10.0));
        // Flexible size with shrink, extra space with ref
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_size(10.0, 5.0, 0.0), 0.0, 20.0, 12.0),
            LAlloc::new(0.0, 10.0, 10.0));

        //
        // FLEXIBLE SIZE WITH (ARBITRARY) STRETCH
        //
        // Flexible size with stretch, too small
        assert_eq!(_alloc_region(
            &LReq::new_flex_size(10.0, 0.0, 3.14), 0.0, 5.0),
            LAlloc::new(0.0, 5.0, 10.0));
        // Flexible size with stretch, too small with ref
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_size(10.0, 0.0, 3.14), 0.0, 5.0, 2.5),
            LAlloc::new(0.0, 5.0, 10.0));

        // Flexible size with stretch, natural size
        assert_eq!(_alloc_region(
            &LReq::new_flex_size(10.0, 0.0, 3.14), 0.0, 10.0),
            LAlloc::new(0.0, 10.0, 10.0));
        // Flexible size with stretch, natural size with ref
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_size(10.0, 0.0, 3.14), 0.0, 10.0, 6.0),
            LAlloc::new(0.0, 10.0, 10.0));

        // Flexible size with stretch, extra space
        assert_eq!(_alloc_region(
            &LReq::new_flex_size(10.0, 0.0, 3.14), 0.0, 20.0),
            LAlloc::new(0.0, 20.0, 20.0));
        // Flexible size with stretch, extra space with ref
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_size(10.0, 0.0, 3.14), 0.0, 20.0, 12.0),
            LAlloc::new(0.0, 20.0, 20.0));

        //
        // FLEXIBLE SIZE WITH SHRINK AND (ARBITRARY) STRETCH
        //
        // Flexible size with shrink and stretch, too small
        assert_eq!(_alloc_region(
            &LReq::new_flex_size(10.0, 5.0, 3.14), 0.0, 4.0),
            LAlloc::new(0.0, 4.0, 5.0));
        // Flexible size with shrink and stretch, too small with ref
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_size(10.0, 5.0, 3.14), 0.0, 4.0, 2.0),
            LAlloc::new(0.0, 4.0, 5.0));

        // Flexible size with shrink and stretch, extra space
        assert_eq!(_alloc_region(
            &LReq::new_flex_size(10.0, 5.0, 3.14), 0.0, 20.0),
            LAlloc::new(0.0, 20.0, 20.0));
        // Flexible size with shrink and stretch, extra space with ref
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_size(10.0, 5.0, 3.14), 0.0, 20.0, 12.0),
            LAlloc::new(0.0, 20.0, 20.0));

        //
        // ********** AS ABOVE BUT WITH REFERENCE POINT **********
        //
        // FIXED SIZE
        //
        // Fixed ref, too small
        assert_eq!(_alloc_region(
            &LReq::new_fixed_ref(3.0, 7.0), 0.0, 5.0),
            LAlloc::new_ref(0.0, 5.0, 10.0, 3.0));
        // Fixed ref, too small with ref too small
        assert_eq!(_alloc_region_ref(
            &LReq::new_fixed_ref(3.0, 7.0), 0.0, 5.0, 2.0),
            LAlloc::new_ref(0.0, 5.0, 10.0, 2.0));
        // Fixed ref, too small with ref
        assert_eq!(_alloc_region_ref(
            &LReq::new_fixed_ref(3.0, 7.0), 0.0, 5.0, 3.0),
            LAlloc::new_ref(0.0, 5.0, 10.0, 3.0));
        // Fixed ref, too small with ref offset
        assert_eq!(_alloc_region_ref(
            &LReq::new_fixed_ref(3.0, 7.0), 0.0, 5.0, 4.0),
            LAlloc::new_ref(1.0, 4.0, 10.0, 3.0));

        // Fixed ref, natural size
        assert_eq!(_alloc_region(
            &LReq::new_fixed_ref(3.0, 7.0), 0.0, 10.0),
            LAlloc::new_ref(0.0, 10.0, 10.0, 3.0));
        // Fixed ref, natural size with ref too small
        assert_eq!(_alloc_region_ref(
            &LReq::new_fixed_ref(3.0, 7.0), 0.0, 10.0, 2.0),
            LAlloc::new_ref(0.0, 10.0, 10.0, 2.0));
        // Fixed ref, natural size with ref
        assert_eq!(_alloc_region_ref(
            &LReq::new_fixed_ref(3.0, 7.0), 0.0, 10.0, 3.0),
            LAlloc::new_ref(0.0, 10.0, 10.0, 3.0));
        // Fixed ref, natural size with ref offset
        assert_eq!(_alloc_region_ref(
            &LReq::new_fixed_ref(3.0, 7.0), 0.0, 10.0, 4.0),
            LAlloc::new_ref(1.0, 9.0, 10.0, 3.0));

        // Fixed ref, extra space
        assert_eq!(_alloc_region(
            &LReq::new_fixed_ref(3.0, 7.0), 0.0, 20.0),
            LAlloc::new_ref(0.0, 10.0, 10.0, 3.0));
        // Fixed ref, extra space with ref too small
        assert_eq!(_alloc_region_ref(
            &LReq::new_fixed_ref(3.0, 7.0), 0.0, 20.0, 2.0),
            LAlloc::new_ref(0.0, 10.0, 10.0, 2.0));
        // Fixed ref, extra space with ref in correct place
        assert_eq!(_alloc_region_ref(
            &LReq::new_fixed_ref(3.0, 7.0), 0.0, 20.0, 3.0),
            LAlloc::new_ref(0.0, 10.0, 10.0, 3.0));
        // Fixed ref, extra space with ref further along; should offset position
        assert_eq!(_alloc_region_ref(
            &LReq::new_fixed_ref(3.0, 7.0), 0.0, 20.0, 5.0),
            LAlloc::new_ref(2.0, 10.0, 10.0, 3.0));

        //
        // FLEXIBLE SIZE WITH SHRINK
        //
        // Flexible ref with shrink, too small
        assert_eq!(_alloc_region(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 0.0), 0.0, 4.0),
            LAlloc::new_ref(0.0, 4.0, 5.0, 3.0));
        // Flexible ref with shrink, too small with ref too small
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 0.0), 0.0, 4.0, 2.0),
            LAlloc::new_ref(0.0, 4.0, 5.0, 2.0));
        // Flexible ref with shrink, too small with correct ref
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 0.0), 0.0, 4.0, 3.0),
            LAlloc::new_ref(0.0, 4.0, 5.0, 3.0));
        // Flexible ref with shrink, too small with ref offset
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 0.0), 0.0, 4.0, 4.0),
            LAlloc::new_ref(1.0, 3.0, 5.0, 3.0));

        // Flexible ref with shrink, minimum size
        assert_eq!(_alloc_region(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 0.0), 0.0, 5.0),
            LAlloc::new_ref(0.0, 5.0, 5.0, 3.0));
        // Flexible ref with shrink, minimum size with ref too small
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 0.0), 0.0, 5.0, 2.0),
            LAlloc::new_ref(0.0, 5.0, 5.0, 2.0));
        // Flexible ref with shrink, minimum size with ref correct
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 0.0), 0.0, 5.0, 3.0),
            LAlloc::new_ref(0.0, 5.0, 5.0, 3.0));
        // Flexible ref with shrink, minimum size with ref offset
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 0.0), 0.0, 5.0, 4.0),
            LAlloc::new_ref(1.0, 4.0, 5.0, 3.0));

        // Flexible ref with shrink, natural size
        assert_eq!(_alloc_region(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 0.0), 0.0, 10.0),
            LAlloc::new_ref(0.0, 10.0, 10.0, 3.0));
        // Flexible ref with shrink, natural size with ref too small
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 0.0), 0.0, 10.0, 2.0),
            LAlloc::new_ref(0.0, 10.0, 10.0, 2.0));
        // Flexible ref with shrink, natural size with ref correct
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 0.0), 0.0, 10.0, 3.0),
            LAlloc::new_ref(0.0, 10.0, 10.0, 3.0));
        // Flexible ref with shrink, natural size with ref offset
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 0.0), 0.0, 10.0, 4.0),
            LAlloc::new_ref(1.0, 9.0, 9.0, 3.0));

        // Flexible ref with shrink, extra space
        assert_eq!(_alloc_region(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 0.0), 0.0, 20.0),
            LAlloc::new_ref(0.0, 10.0, 10.0, 3.0));
        // Flexible ref with shrink, extra space with ref too small
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 0.0), 0.0, 20.0, 2.0),
            LAlloc::new_ref(0.0, 10.0, 10.0, 2.0));
        // Flexible ref with shrink,extra space with ref correct
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 0.0), 0.0, 20.0, 3.0),
            LAlloc::new_ref(0.0, 10.0, 10.0, 3.0));
        // Flexible ref with shrink, extra space with ref offset
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 0.0), 0.0, 20.0, 4.0),
            LAlloc::new_ref(1.0, 10.0, 10.0, 3.0));

        //
        // FLEXIBLE SIZE WITH SHRINK AND (ARBITRARY) STRETCH
        //
        // Flexible ref with shrink and stretch, natural size
        assert_eq!(_alloc_region(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 3.14), 0.0, 10.0),
            LAlloc::new_ref(0.0, 10.0, 10.0, 3.0));
        // Flexible ref with shrink and stretch, natural size with ref too small
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 3.14), 0.0, 10.0, 2.0),
            LAlloc::new_ref(0.0, 10.0, 10.0, 2.0));
        // Flexible ref with shrink and stretch, natural size with ref correct
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 3.14), 0.0, 10.0, 3.0),
            LAlloc::new_ref(0.0, 10.0, 10.0, 3.0));
        // Flexible ref with shrink and stretch, natural size with ref offset
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 3.14), 0.0, 10.0, 4.0),
            LAlloc::new_ref(0.0, 10.0, 10.0, 4.0));

        // Flexible ref with shrink and stretch, extra space
        assert_eq!(_alloc_region(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 3.14), 0.0, 20.0),
            LAlloc::new_ref(0.0, 20.0, 20.0, 3.0));
        // Flexible ref with shrink and stretch, extra space with ref too small
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 3.14), 0.0, 20.0, 2.0),
            LAlloc::new_ref(0.0, 20.0, 20.0, 2.0));
        // Flexible ref with shrink and stretch,extra space with ref correct
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 3.14), 0.0, 20.0, 3.0),
            LAlloc::new_ref(0.0, 20.0, 20.0, 3.0));
        // Flexible ref with shrink and stretch, extra space with ref offset
        assert_eq!(_alloc_region_ref(
            &LReq::new_flex_ref(3.0, 7.0, 5.0, 3.14), 0.0, 20.0, 4.0),
            LAlloc::new_ref(0.0, 20.0, 20.0, 4.0));
    }

    #[test]
    fn test_alloc_linear() {
        let ds0 = LReq::new_fixed_size(10.0);
        let ds1 = LReq::new_fixed_size(20.0);
        let ds2 = LReq::new_fixed_size(30.0);
        let dr0 = LReq::new_fixed_ref(7.5, 2.5);
        let dr1 = LReq::new_fixed_ref(15.0, 5.0);
        let dr2 = LReq::new_fixed_ref(22.5, 7.5);
        let ks0 = LReq::new_flex_size(10.0, 2.0, 0.0);
        let ks1 = LReq::new_flex_size(20.0, 4.0, 0.0);
        let ks2 = LReq::new_flex_size(30.0, 6.0, 0.0);
        let kr0 = LReq::new_flex_ref(7.5, 2.5, 2.0, 0.0);
        let kr1 = LReq::new_flex_ref(15.0, 5.0, 4.0, 0.0);
        let kr2 = LReq::new_flex_ref(22.5, 7.5, 6.0, 0.0);
        let es0 = LReq::new_flex_size(10.0, 0.0, 1.0);
        let es1 = LReq::new_flex_size(20.0, 0.0, 3.0);
        let es2 = LReq::new_flex_size(30.0, 0.0, 6.0);
        let er0 = LReq::new_flex_ref(7.5, 2.5, 0.0, 1.0);
        let er1 = LReq::new_flex_ref(15.0, 5.0, 0.0, 3.0);
        let er2 = LReq::new_flex_ref(22.5, 7.5, 0.0, 6.0);

        // -- 3 fixed size children, no ref points
        // 3 fixed size children, too little space
        {
            let (req, allocs) = alloc_linear(0.0, 30.0, None,
                &vec![&ds0, &ds1, &ds2], 0.0, None);

            assert_eq!(req, LReq::new_fixed_size(60.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(10.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(30.0, 30.0, 30.0));
        }

        // 3 fixed size children, natural size
        {
            let (req, allocs) = alloc_linear(0.0, 60.0, None,
                &vec![&ds0, &ds1, &ds2], 0.0, None);

            assert_eq!(req, LReq::new_fixed_size(60.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(10.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(30.0, 30.0, 30.0));
        }

        // 3 fixed size children, additional space
        {
            let (req, allocs) = alloc_linear(0.0, 100.0, None,
                &vec![&ds0, &ds1, &ds2], 0.0, None);

            assert_eq!(req, LReq::new_fixed_size(60.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(10.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(30.0, 30.0, 30.0));
        }

        // -- 3 fixed size children, with ref points
        // 3 fixed ref children, too little space
        {
            let (req, allocs) = alloc_linear(0.0, 30.0, None,
                &vec![&dr0, &dr1, &dr2], 0.0, None);

            assert_eq!(req, LReq::new_fixed_size(60.0));
            assert_eq!(allocs[0], LAlloc::new_ref(0.0, 10.0, 10.0, 7.5));
            assert_eq!(allocs[1], LAlloc::new_ref(10.0, 20.0, 20.0, 15.0));
            assert_eq!(allocs[2], LAlloc::new_ref(30.0, 30.0, 30.0, 22.5));
        }

        // 3 fixed ref children, natural size
        {
            let (req, allocs) = alloc_linear(0.0, 60.0, None,
                &vec![&dr0, &dr1, &dr2], 0.0, None);

            assert_eq!(req, LReq::new_fixed_size(60.0));
            assert_eq!(allocs[0], LAlloc::new_ref(0.0, 10.0, 10.0, 7.5));
            assert_eq!(allocs[1], LAlloc::new_ref(10.0, 20.0, 20.0, 15.0));
            assert_eq!(allocs[2], LAlloc::new_ref(30.0, 30.0, 30.0, 22.5));
        }

        // 3 fixed ref children, additional space
        {
            let (req, allocs) = alloc_linear(0.0, 100.0, None,
                &vec![&dr0, &dr1, &dr2], 0.0, None);

            assert_eq!(req, LReq::new_fixed_size(60.0));
            assert_eq!(allocs[0], LAlloc::new_ref(0.0, 10.0, 10.0, 7.5));
            assert_eq!(allocs[1], LAlloc::new_ref(10.0, 20.0, 20.0, 15.0));
            assert_eq!(allocs[2], LAlloc::new_ref(30.0, 30.0, 30.0, 22.5));
        }

        // -- 3 shrinkable children, no ref points
        // 3 shrinkable children, less than minimum size
        {
            let (req, allocs) = alloc_linear(0.0, 30.0, None,
                &vec![&ks0, &ks1, &ks2], 0.0, None);

            assert_eq!(req, LReq::new_flex_size(60.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 8.0, 8.0));
            assert_eq!(allocs[1], LAlloc::new(8.0, 16.0, 16.0));
            assert_eq!(allocs[2], LAlloc::new(24.0, 24.0, 24.0));
        }

        // 3 shrinkable children, minimum size
        {
            let (req, allocs) = alloc_linear(0.0, 48.0, None,
                &vec![&ks0, &ks1, &ks2], 0.0, None);

            assert_eq!(req, LReq::new_flex_size(60.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 8.0, 8.0));
            assert_eq!(allocs[1], LAlloc::new(8.0, 16.0, 16.0));
            assert_eq!(allocs[2], LAlloc::new(24.0, 24.0, 24.0));
        }

        // 3 shrinkable children, between minimum and natural size
        {
            let (req, allocs) = alloc_linear(0.0, 54.0, None,
                &vec![&ks0, &ks1, &ks2], 0.0, None);

            assert_eq!(req, LReq::new_flex_size(60.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 9.0, 9.0));
            assert_eq!(allocs[1], LAlloc::new(9.0, 18.0, 18.0));
            assert_eq!(allocs[2], LAlloc::new(27.0, 27.0, 27.0));
        }

        // 3 shrinkable children, natural size
        {
            let (req, allocs) = alloc_linear(0.0, 60.0, None,
                &vec![&ks0, &ks1, &ks2], 0.0, None);

            assert_eq!(req, LReq::new_flex_size(60.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(10.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(30.0, 30.0, 30.0));
        }

        // 3 shrinkable children, additional space
        {
            let (req, allocs) = alloc_linear(0.0, 100.0, None,
                &vec![&ks0, &ks1, &ks2], 0.0, None);

            assert_eq!(req, LReq::new_flex_size(60.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(10.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(30.0, 30.0, 30.0));
        }

        // -- 3 shrinkable children, with ref points
        // 3 shrinkable ref children, less than minimum size
        {
            let (req, allocs) = alloc_linear(0.0, 30.0, None,
                &vec![&kr0, &kr1, &kr2], 0.0, None);

            assert_eq!(req, LReq::new_flex_size(60.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new_ref(0.0, 8.0, 8.0, 6.0));
            assert_eq!(allocs[1], LAlloc::new_ref(8.0, 16.0, 16.0, 12.0));
            assert_eq!(allocs[2], LAlloc::new_ref(24.0, 24.0, 24.0, 18.0));
        }

        // 3 shrinkable ref children, minimum size
        {
            let (req, allocs) = alloc_linear(0.0, 48.0, None,
                &vec![&kr0, &kr1, &kr2], 0.0, None);

            assert_eq!(req, LReq::new_flex_size(60.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new_ref(0.0, 8.0, 8.0, 6.0));
            assert_eq!(allocs[1], LAlloc::new_ref(8.0, 16.0, 16.0, 12.0));
            assert_eq!(allocs[2], LAlloc::new_ref(24.0, 24.0, 24.0, 18.0));
        }

        // 3 shrinkable ref children, between minimum and natural size
        {
            let (req, allocs) = alloc_linear(0.0, 54.0, None,
                &vec![&kr0, &kr1, &kr2], 0.0, None);

            assert_eq!(req, LReq::new_flex_size(60.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new_ref(0.0, 9.0, 9.0, 6.75));
            assert_eq!(allocs[1], LAlloc::new_ref(9.0, 18.0, 18.0, 13.5));
            assert_eq!(allocs[2], LAlloc::new_ref(27.0, 27.0, 27.0, 20.25));
        }

        // 3 shrinkable ref children, natural size
        {
            let (req, allocs) = alloc_linear(0.0, 60.0, None,
                &vec![&kr0, &kr1, &kr2], 0.0, None);

            assert_eq!(req, LReq::new_flex_size(60.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new_ref(0.0, 10.0, 10.0, 7.5));
            assert_eq!(allocs[1], LAlloc::new_ref(10.0, 20.0, 20.0, 15.0));
            assert_eq!(allocs[2], LAlloc::new_ref(30.0, 30.0, 30.0, 22.5));
        }

        // 3 shrinkable ref children, additional space
        {
            let (req, allocs) = alloc_linear(0.0, 100.0, None,
                &vec![&kr0, &kr1, &kr2], 0.0, None);

            assert_eq!(req, LReq::new_flex_size(60.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new_ref(0.0, 10.0, 10.0, 7.5));
            assert_eq!(allocs[1], LAlloc::new_ref(10.0, 20.0, 20.0, 15.0));
            assert_eq!(allocs[2], LAlloc::new_ref(30.0, 30.0, 30.0, 22.5));
        }

        // -- 3 stretchable children, no ref points
        // 3 stretchable children, too little space
        {
            let (req, allocs) = alloc_linear(0.0, 30.0, None,
                &vec![&es0, &es1, &es2], 0.0, None);

            assert_eq!(req, LReq::new_flex_size(60.0, 0.0, 10.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(10.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(30.0, 30.0, 30.0));
        }

        // 3 fixed size children, natural size
        {
            let (req, allocs) = alloc_linear(0.0, 60.0, None,
                &vec![&es0, &es1, &es2], 0.0, None);

            assert_eq!(req, LReq::new_flex_size(60.0, 0.0, 10.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(10.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(30.0, 30.0, 30.0));
        }

        // 3 fixed size children, additional space
        {
            let (req, allocs) = alloc_linear(0.0, 100.0, None,
                &vec![&es0, &es1, &es2], 0.0, None);

            assert_eq!(req, LReq::new_flex_size(60.0, 0.0, 10.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 14.0, 14.0));
            assert_eq!(allocs[1], LAlloc::new(14.0, 32.0, 32.0));
            assert_eq!(allocs[2], LAlloc::new(46.0, 54.0, 54.0));
        }

        // -- 3 stretchable children, with ref points
        // 3 stretchable children, too little space
        {
            let (req, allocs) = alloc_linear(0.0, 30.0, None,
                &vec![&er0, &er1, &er2], 0.0, None);

            assert_eq!(req, LReq::new_flex_size(60.0, 0.0, 10.0));
            assert_eq!(allocs[0], LAlloc::new_ref(0.0, 10.0, 10.0, 7.5));
            assert_eq!(allocs[1], LAlloc::new_ref(10.0, 20.0, 20.0, 15.0));
            assert_eq!(allocs[2], LAlloc::new_ref(30.0, 30.0, 30.0, 22.5));
        }

        // 3 fixed size children, natural size
        {
            let (req, allocs) = alloc_linear(0.0, 60.0, None,
                &vec![&er0, &er1, &er2], 0.0, None);

            assert_eq!(req, LReq::new_flex_size(60.0, 0.0, 10.0));
            assert_eq!(allocs[0], LAlloc::new_ref(0.0, 10.0, 10.0, 7.5));
            assert_eq!(allocs[1], LAlloc::new_ref(10.0, 20.0, 20.0, 15.0));
            assert_eq!(allocs[2], LAlloc::new_ref(30.0, 30.0, 30.0, 22.5));
        }

        // 3 fixed size children, additional space
        {
            let (req, allocs) = alloc_linear(0.0, 100.0, None,
                &vec![&er0, &er1, &er2], 0.0, None);

            assert_eq!(req, LReq::new_flex_size(60.0, 0.0, 10.0));
            assert_eq!(allocs[0], LAlloc::new_ref(0.0, 14.0, 14.0, 10.5));
            assert_eq!(allocs[1], LAlloc::new_ref(14.0, 32.0, 32.0, 24.0));
            assert_eq!(allocs[2], LAlloc::new_ref(46.0, 54.0, 54.0, 40.5));
        }
    }

    #[test]
    fn test_alloc_linear_ref() {
        let ds0 = LReq::new_fixed_size(10.0);
        let ds1 = LReq::new_fixed_size(20.0);
        let ds2 = LReq::new_fixed_size(30.0);
        let dr0 = LReq::new_fixed_ref(7.5, 2.5);
        let dr1 = LReq::new_fixed_ref(15.0, 5.0);
        let dr2 = LReq::new_fixed_ref(22.5, 7.5);
        let ks0 = LReq::new_flex_size(10.0, 2.0, 0.0);
        let ks1 = LReq::new_flex_size(20.0, 4.0, 0.0);
        let ks2 = LReq::new_flex_size(30.0, 6.0, 0.0);
        let kr0 = LReq::new_flex_ref(7.5, 2.5, 2.0, 0.0);
        let kr1 = LReq::new_flex_ref(15.0, 5.0, 4.0, 0.0);
        let kr2 = LReq::new_flex_ref(22.5, 7.5, 6.0, 0.0);
        let es0 = LReq::new_flex_size(10.0, 0.0, 1.0);
        let es1 = LReq::new_flex_size(20.0, 0.0, 3.0);
        let es2 = LReq::new_flex_size(30.0, 0.0, 6.0);
        let er0 = LReq::new_flex_ref(7.5, 2.5, 0.0, 1.0);
        let er1 = LReq::new_flex_ref(15.0, 5.0, 0.0, 3.0);
        let er2 = LReq::new_flex_ref(22.5, 7.5, 0.0, 6.0);

        // -- 3 fixed size children, no ref points
        // 3 fixed size children, too little space
        {
            let (req, allocs) = alloc_linear(0.0, 30.0, Some(15.0),
                &vec![&ds0, &ds1, &ds2], 0.0, Some(0));

            assert_eq!(req, LReq::new_fixed_ref(5.0, 55.0));
            assert_eq!(allocs[0], LAlloc::new(10.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(20.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(40.0, 30.0, 30.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 30.0, Some(15.0),
                &vec![&ds0, &ds1, &ds2], 0.0, Some(1));

            assert_eq!(req, LReq::new_fixed_ref(20.0, 40.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(10.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(30.0, 30.0, 30.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 30.0, Some(15.0),
                &vec![&ds0, &ds1, &ds2], 0.0, Some(2));

            assert_eq!(req, LReq::new_fixed_ref(45.0, 15.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(10.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(30.0, 30.0, 30.0));
        }

        // 3 fixed size children, natural size
        {
            let (req, allocs) = alloc_linear(0.0, 70.0, Some(15.0),
                &vec![&ds0, &ds1, &ds2], 0.0, Some(0));

            assert_eq!(req, LReq::new_fixed_ref(5.0, 55.0));
            assert_eq!(allocs[0], LAlloc::new(10.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(20.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(40.0, 30.0, 30.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 60.0, Some(15.0),
                &vec![&ds0, &ds1, &ds2], 0.0, Some(1));

            assert_eq!(req, LReq::new_fixed_ref(20.0, 40.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(10.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(30.0, 30.0, 30.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 60.0, Some(15.0),
                &vec![&ds0, &ds1, &ds2], 0.0, Some(2));

            assert_eq!(req, LReq::new_fixed_ref(45.0, 15.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(10.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(30.0, 30.0, 30.0));
        }

        // 3 fixed size children, additional space
        {
            let (req, allocs) = alloc_linear(0.0, 1000.0, Some(15.0),
                &vec![&ds0, &ds1, &ds2], 0.0, Some(0));

            assert_eq!(req, LReq::new_fixed_ref(5.0, 55.0));
            assert_eq!(allocs[0], LAlloc::new(10.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(20.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(40.0, 30.0, 30.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 1000.0, Some(40.0),
                &vec![&ds0, &ds1, &ds2], 0.0, Some(1));

            assert_eq!(req, LReq::new_fixed_ref(20.0, 40.0));
            assert_eq!(allocs[0], LAlloc::new(20.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(30.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(50.0, 30.0, 30.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 1000.0, Some(85.0),
                &vec![&ds0, &ds1, &ds2], 0.0, Some(2));

            assert_eq!(req, LReq::new_fixed_ref(45.0, 15.0));
            assert_eq!(allocs[0], LAlloc::new(40.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(50.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(70.0, 30.0, 30.0));
        }

        // -- 3 fixed size children, with ref points
        // 3 fixed ref children, too little space
        {
            let (req, allocs) = alloc_linear(0.0, 30.0, Some(15.0),
                &vec![&dr0, &dr1, &dr2], 0.0, Some(0));

            assert_eq!(req, LReq::new_fixed_ref(7.5, 52.5));
            assert_eq!(allocs[0], LAlloc::new_ref(7.5, 10.0, 10.0, 7.5));
            assert_eq!(allocs[1], LAlloc::new_ref(17.5, 20.0, 20.0, 15.0));
            assert_eq!(allocs[2], LAlloc::new_ref(37.5, 30.0, 30.0, 22.5));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 30.0, Some(15.0),
                &vec![&dr0, &dr1, &dr2], 0.0, Some(1));

            assert_eq!(req, LReq::new_fixed_ref(25.0, 35.0));
            assert_eq!(allocs[0], LAlloc::new_ref(0.0, 10.0, 10.0, 7.5));
            assert_eq!(allocs[1], LAlloc::new_ref(10.0, 20.0, 20.0, 15.0));
            assert_eq!(allocs[2], LAlloc::new_ref(30.0, 30.0, 30.0, 22.5));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 30.0, Some(15.0),
                &vec![&dr0, &dr1, &dr2], 0.0, Some(2));

            assert_eq!(req, LReq::new_fixed_ref(52.5, 7.5));
            assert_eq!(allocs[0], LAlloc::new_ref(0.0, 10.0, 10.0, 7.5));
            assert_eq!(allocs[1], LAlloc::new_ref(10.0, 20.0, 20.0, 15.0));
            assert_eq!(allocs[2], LAlloc::new_ref(30.0, 30.0, 30.0, 22.5));
        }

        // 3 fixed ref children, natural size
        {
            let (req, allocs) = alloc_linear(0.0, 60.0, Some(15.0),
                &vec![&dr0, &dr1, &dr2], 0.0, Some(0));

            assert_eq!(req, LReq::new_fixed_ref(7.5, 52.5));
            assert_eq!(allocs[0], LAlloc::new_ref(7.5, 10.0, 10.0, 7.5));
            assert_eq!(allocs[1], LAlloc::new_ref(17.5, 20.0, 20.0, 15.0));
            assert_eq!(allocs[2], LAlloc::new_ref(37.5, 30.0, 30.0, 22.5));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 60.0, Some(15.0),
                &vec![&dr0, &dr1, &dr2], 0.0, Some(1));

            assert_eq!(req, LReq::new_fixed_ref(25.0, 35.0));
            assert_eq!(allocs[0], LAlloc::new_ref(0.0, 10.0, 10.0, 7.5));
            assert_eq!(allocs[1], LAlloc::new_ref(10.0, 20.0, 20.0, 15.0));
            assert_eq!(allocs[2], LAlloc::new_ref(30.0, 30.0, 30.0, 22.5));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 60.0, Some(15.0),
                &vec![&dr0, &dr1, &dr2], 0.0, Some(2));

            assert_eq!(req, LReq::new_fixed_ref(52.5, 7.5));
            assert_eq!(allocs[0], LAlloc::new_ref(0.0, 10.0, 10.0, 7.5));
            assert_eq!(allocs[1], LAlloc::new_ref(10.0, 20.0, 20.0, 15.0));
            assert_eq!(allocs[2], LAlloc::new_ref(30.0, 30.0, 30.0, 22.5));
        }

        // 3 fixed ref children, additional space
        {
            let (req, allocs) = alloc_linear(0.0, 1000.0, Some(15.0),
                &vec![&dr0, &dr1, &dr2], 0.0, Some(0));

            assert_eq!(req, LReq::new_fixed_ref(7.5, 52.5));
            assert_eq!(allocs[0], LAlloc::new_ref(7.5, 10.0, 10.0, 7.5));
            assert_eq!(allocs[1], LAlloc::new_ref(17.5, 20.0, 20.0, 15.0));
            assert_eq!(allocs[2], LAlloc::new_ref(37.5, 30.0, 30.0, 22.5));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 1000.0, Some(45.0),
                &vec![&dr0, &dr1, &dr2], 0.0, Some(1));

            assert_eq!(req, LReq::new_fixed_ref(25.0, 35.0));
            assert_eq!(allocs[0], LAlloc::new_ref(20.0, 10.0, 10.0, 7.5));
            assert_eq!(allocs[1], LAlloc::new_ref(30.0, 20.0, 20.0, 15.0));
            assert_eq!(allocs[2], LAlloc::new_ref(50.0, 30.0, 30.0, 22.5));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 1000.0, Some(82.5),
                &vec![&dr0, &dr1, &dr2], 0.0, Some(2));

            assert_eq!(req, LReq::new_fixed_ref(52.5, 7.5));
            assert_eq!(allocs[0], LAlloc::new_ref(30.0, 10.0, 10.0, 7.5));
            assert_eq!(allocs[1], LAlloc::new_ref(40.0, 20.0, 20.0, 15.0));
            assert_eq!(allocs[2], LAlloc::new_ref(60.0, 30.0, 30.0, 22.5));
        }

        // -- 3 shrinkable children, no ref points
        // 3 shrinkable children, less than minimum size
        {
            let (req, allocs) = alloc_linear(0.0, 30.0, Some(4.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(0));

            assert_eq!(req, LReq::new_flex_ref(5.0, 55.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 8.0, 8.0));
            assert_eq!(allocs[1], LAlloc::new(8.0, 16.0, 16.0));
            assert_eq!(allocs[2], LAlloc::new(24.0, 24.0, 24.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 30.0, Some(15.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(0));

            assert_eq!(req, LReq::new_flex_ref(5.0, 55.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(10.0, 9.0, 9.0));
            assert_eq!(allocs[1], LAlloc::new(19.0, 16.0, 16.0));
            assert_eq!(allocs[2], LAlloc::new(35.0, 24.0, 24.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 30.0, Some(16.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(1));

            assert_eq!(req, LReq::new_flex_ref(20.0, 40.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 8.0, 8.0));
            assert_eq!(allocs[1], LAlloc::new(8.0, 16.0, 16.0));
            assert_eq!(allocs[2], LAlloc::new(24.0, 24.0, 24.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 30.0, Some(24.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(1));

            assert_eq!(req, LReq::new_flex_ref(20.0, 40.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(4.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(14.0, 18.0, 18.0));
            assert_eq!(allocs[2], LAlloc::new(32.0, 24.0, 24.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 30.0, Some(36.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(2));

            assert_eq!(req, LReq::new_flex_ref(45.0, 15.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 8.0, 8.0));
            assert_eq!(allocs[1], LAlloc::new(8.0, 16.0, 16.0));
            assert_eq!(allocs[2], LAlloc::new(24.0, 24.0, 24.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 30.0, Some(55.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(2));

            assert_eq!(req, LReq::new_flex_ref(45.0, 15.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(10.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(20.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(40.0, 27.0, 27.0));
        }

        // 3 shrinkable children, minimum size
        {
            let (req, allocs) = alloc_linear(0.0, 48.0, Some(4.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(0));

            assert_eq!(req, LReq::new_flex_ref(5.0, 55.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 8.0, 8.0));
            assert_eq!(allocs[1], LAlloc::new(8.0, 16.0, 16.0));
            assert_eq!(allocs[2], LAlloc::new(24.0, 24.0, 24.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 59.0, Some(15.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(0));

            assert_eq!(req, LReq::new_flex_ref(5.0, 55.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(10.0, 9.0, 9.0));
            assert_eq!(allocs[1], LAlloc::new(19.0, 16.0, 16.0));
            assert_eq!(allocs[2], LAlloc::new(35.0, 24.0, 24.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 48.0, Some(16.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(1));

            assert_eq!(req, LReq::new_flex_ref(20.0, 40.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 8.0, 8.0));
            assert_eq!(allocs[1], LAlloc::new(8.0, 16.0, 16.0));
            assert_eq!(allocs[2], LAlloc::new(24.0, 24.0, 24.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 56.0, Some(24.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(1));

            assert_eq!(req, LReq::new_flex_ref(20.0, 40.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(4.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(14.0, 18.0, 18.0));
            assert_eq!(allocs[2], LAlloc::new(32.0, 24.0, 24.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 48.0, Some(36.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(2));

            assert_eq!(req, LReq::new_flex_ref(45.0, 15.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 8.0, 8.0));
            assert_eq!(allocs[1], LAlloc::new(8.0, 16.0, 16.0));
            assert_eq!(allocs[2], LAlloc::new(24.0, 24.0, 24.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 67.0, Some(55.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(2));

            assert_eq!(req, LReq::new_flex_ref(45.0, 15.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(10.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(20.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(40.0, 27.0, 27.0));
        }

        // 3 shrinkable children, between minimum and natural size
        {
            let (req, allocs) = alloc_linear(0.0, 53.5, Some(4.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(0));

            assert_eq!(req, LReq::new_flex_ref(5.0, 55.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 8.5, 8.5));
            assert_eq!(allocs[1], LAlloc::new(8.5, 18.0, 18.0));
            assert_eq!(allocs[2], LAlloc::new(26.5, 27.0, 27.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 64.5, Some(15.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(0));

            assert_eq!(req, LReq::new_flex_ref(5.0, 55.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(10.0, 9.5, 9.5));
            assert_eq!(allocs[1], LAlloc::new(19.5, 18.0, 18.0));
            assert_eq!(allocs[2], LAlloc::new(37.5, 27.0, 27.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 52.0, Some(16.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(1));

            assert_eq!(req, LReq::new_flex_ref(20.0, 40.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 8.0, 8.0));
            assert_eq!(allocs[1], LAlloc::new(8.0, 17.0, 17.0));
            assert_eq!(allocs[2], LAlloc::new(25.0, 27.0, 27.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 61.0, Some(25.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(1));

            assert_eq!(req, LReq::new_flex_ref(20.0, 40.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(5.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(15.0, 19.0, 19.0));
            assert_eq!(allocs[2], LAlloc::new(34.0, 27.0, 27.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 49.5, Some(36.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(2));

            assert_eq!(req, LReq::new_flex_ref(45.0, 15.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(0.0, 8.0, 8.0));
            assert_eq!(allocs[1], LAlloc::new(8.0, 16.0, 16.0));
            assert_eq!(allocs[2], LAlloc::new(24.0, 25.5, 25.5));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 68.5, Some(55.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(2));

            assert_eq!(req, LReq::new_flex_ref(45.0, 15.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(10.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(20.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(40.0, 28.5, 28.5));
        }

        // 3 shrinkable children, natural size
        {
            let (req, allocs) = alloc_linear(0.0, 70.0, Some(15.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(0));

            assert_eq!(req, LReq::new_flex_ref(5.0, 55.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(10.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(20.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(40.0, 30.0, 30.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 65.0, Some(25.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(1));

            assert_eq!(req, LReq::new_flex_ref(20.0, 40.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(5.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(15.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(35.0, 30.0, 30.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 70.0, Some(55.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(2));

            assert_eq!(req, LReq::new_flex_ref(45.0, 15.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(10.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(20.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(40.0, 30.0, 30.0));
        }

        // 3 shrinkable children, additional space
        {
            let (req, allocs) = alloc_linear(0.0, 500.0, Some(15.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(0));

            assert_eq!(req, LReq::new_flex_ref(5.0, 55.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(10.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(20.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(40.0, 30.0, 30.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 500.0, Some(25.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(1));

            assert_eq!(req, LReq::new_flex_ref(20.0, 40.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(5.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(15.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(35.0, 30.0, 30.0));
        }

        {
            let (req, allocs) = alloc_linear(0.0, 500.0, Some(55.0),
                &vec![&ks0, &ks1, &ks2], 0.0, Some(2));

            assert_eq!(req, LReq::new_flex_ref(45.0, 15.0, 12.0, 0.0));
            assert_eq!(allocs[0], LAlloc::new(10.0, 10.0, 10.0));
            assert_eq!(allocs[1], LAlloc::new(20.0, 20.0, 20.0));
            assert_eq!(allocs[2], LAlloc::new(40.0, 30.0, 30.0));
        }

        // -- 3 shrinkable children with ref points

        // too little space
        alloc_linear_ref_test(&vec![&kr0, &kr1, &kr2], 1.2, 1.3, 0.0, 0.0);
        alloc_linear_ref_test(&vec![&kr0, &kr1, &kr2], 1.3, 1.2, 0.0, 0.0);

        // too little space before ref, minimum space after
        alloc_linear_ref_test(&vec![&kr0, &kr1, &kr2], 1.3, 1.0, 0.0, 0.0);
        // minimum space before ref, too little space after
        alloc_linear_ref_test(&vec![&kr0, &kr1, &kr2], 1.0, 1.3, 0.0, 0.0);

        // 40% shrink before ref, minimum space after
        alloc_linear_ref_test(&vec![&kr0, &kr1, &kr2], 0.4, 1.0, 0.0, 0.0);
        // minimum space before ref, 40% shrink after
        alloc_linear_ref_test(&vec![&kr0, &kr1, &kr2], 1.0, 0.4, 0.0, 0.0);

        // 40% shrink before ref, 60% shrink after
        alloc_linear_ref_test(&vec![&kr0, &kr1, &kr2], 0.4, 0.6, 0.0, 0.0);
        // 60% shrink before ref, 40% shrink after
        alloc_linear_ref_test(&vec![&kr0, &kr1, &kr2], 0.6, 0.4, 0.0, 0.0);

        // preferred size before ref, 60% shrink after
        alloc_linear_ref_test(&vec![&kr0, &kr1, &kr2], 0.0, 0.6, 0.0, 0.0);
        // 60% shrink before ref, preferred size after
        alloc_linear_ref_test(&vec![&kr0, &kr1, &kr2], 0.6, 0.0, 0.0, 0.0);

        // 40% shrink before ref, 120 stretch space after
        alloc_linear_ref_test(&vec![&kr0, &kr1, &kr2], 0.4, 0.0, 0.0, 120.0);
        // 120 stretch before ref, 40% shrink after
        alloc_linear_ref_test(&vec![&kr0, &kr1, &kr2], 0.0, 0.4, 120.0, 0.0);

        // 240 stretch before ref, 120 stretch space after
        alloc_linear_ref_test(&vec![&kr0, &kr1, &kr2], 0.0, 0.0, 240.0, 120.0);
        // 120 stretch before ref, 240 stretch after
        alloc_linear_ref_test(&vec![&kr0, &kr1, &kr2], 0.0, 0.0, 120.0, 240.0);

        // -- 3 stretchable children, no ref points

        // too little space
        alloc_linear_ref_test(&vec![&es0, &es1, &es2], 1.2, 1.3, 0.0, 0.0);
        alloc_linear_ref_test(&vec![&es0, &es1, &es2], 1.3, 1.2, 0.0, 0.0);

        // too little space before ref, minimum space after
        alloc_linear_ref_test(&vec![&es0, &es1, &es2], 1.3, 1.0, 0.0, 0.0);
        // minimum space before ref, too little space after
        alloc_linear_ref_test(&vec![&es0, &es1, &es2], 1.0, 1.3, 0.0, 0.0);

        // 40% shrink before ref, minimum space after
        alloc_linear_ref_test(&vec![&es0, &es1, &es2], 0.4, 1.0, 0.0, 0.0);
        // minimum space before ref, 40% shrink after
        alloc_linear_ref_test(&vec![&es0, &es1, &es2], 1.0, 0.4, 0.0, 0.0);

        // 40% shrink before ref, 60% shrink after
        alloc_linear_ref_test(&vec![&es0, &es1, &es2], 0.4, 0.6, 0.0, 0.0);
        // 60% shrink before ref, 40% shrink after
        alloc_linear_ref_test(&vec![&es0, &es1, &es2], 0.6, 0.4, 0.0, 0.0);

        // preferred size before ref, 60% shrink after
        alloc_linear_ref_test(&vec![&es0, &es1, &es2], 0.0, 0.6, 0.0, 0.0);
        // 60% shrink before ref, preferred size after
        alloc_linear_ref_test(&vec![&es0, &es1, &es2], 0.6, 0.0, 0.0, 0.0);

        // 40% shrink before ref, 120 stretch space after
        alloc_linear_ref_test(&vec![&es0, &es1, &es2], 0.4, 0.0, 0.0, 120.0);
        // 120 stretch before ref, 40% shrink after
        alloc_linear_ref_test(&vec![&es0, &es1, &es2], 0.0, 0.4, 120.0, 0.0);

        // 240 stretch before ref, 120 stretch space after
        alloc_linear_ref_test(&vec![&es0, &es1, &es2], 0.0, 0.0, 240.0, 120.0);
        // 120 stretch before ref, 240 stretch after
        alloc_linear_ref_test(&vec![&es0, &es1, &es2], 0.0, 0.0, 120.0, 240.0);

        // -- 3 stretchable children, with ref points

        // too little space
        alloc_linear_ref_test(&vec![&er0, &er1, &er2], 1.2, 1.3, 0.0, 0.0);
        alloc_linear_ref_test(&vec![&er0, &er1, &er2], 1.3, 1.2, 0.0, 0.0);

        // too little space before ref, minimum space after
        alloc_linear_ref_test(&vec![&er0, &er1, &er2], 1.3, 1.0, 0.0, 0.0);
        // minimum space before ref, too little space after
        alloc_linear_ref_test(&vec![&er0, &er1, &er2], 1.0, 1.3, 0.0, 0.0);

        // 40% shrink before ref, minimum space after
        alloc_linear_ref_test(&vec![&er0, &er1, &er2], 0.4, 1.0, 0.0, 0.0);
        // minimum space before ref, 40% shrink after
        alloc_linear_ref_test(&vec![&er0, &er1, &er2], 1.0, 0.4, 0.0, 0.0);

        // 40% shrink before ref, 60% shrink after
        alloc_linear_ref_test(&vec![&er0, &er1, &er2], 0.4, 0.6, 0.0, 0.0);
        // 60% shrink before ref, 40% shrink after
        alloc_linear_ref_test(&vec![&er0, &er1, &er2], 0.6, 0.4, 0.0, 0.0);

        // preferred size before ref, 60% shrink after
        alloc_linear_ref_test(&vec![&er0, &er1, &er2], 0.0, 0.6, 0.0, 0.0);
        // 60% shrink before ref, preferred size after
        alloc_linear_ref_test(&vec![&er0, &er1, &er2], 0.6, 0.0, 0.0, 0.0);

        // 40% shrink before ref, 100 stretch space after
        alloc_linear_ref_test(&vec![&er0, &er1, &er2], 0.4, 0.0, 0.0, 100.0);
        // 100 stretch before ref, 40% shrink after
        alloc_linear_ref_test(&vec![&er0, &er1, &er2], 0.0, 0.4, 100.0, 0.0);

        // 200 stretch before ref, 100 stretch space after
        alloc_linear_ref_test(&vec![&er0, &er1, &er2], 0.0, 0.0, 200.0, 100.0);
        // 100 stretch before ref, 200 stretch after
        alloc_linear_ref_test(&vec![&er0, &er1, &er2], 0.0, 0.0, 100.0, 200.0);
    }


    #[bench]
    fn bench_lalloc_linear_alloc(bench: &mut test::Bencher) {
        let num_children = 100;
        let num_parents = 100;

        let natsize_type_range: Range<i32> = Range::new(0, 8);
        let size_range = Range::new(5.0, 25.0);
        let flex_type_range: Range<i32> = Range::new(0, 2);
        let flex_range = Range::new(1.0, 3.0);
        let mut rng = rand::thread_rng();

        let mut child_reqs = Vec::with_capacity(num_children);
        let mut child_allocs = Vec::with_capacity(num_children);
        let mut parent_reqs = Vec::with_capacity(num_parents);

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
            child_reqs.push(LReq::new(size, flex));
            child_allocs.push(LAlloc::new_empty());
        }

        let child_req_refs: Vec<&LReq> = child_reqs.iter().collect();
        // let mut child_alloc_refs : Vec<&mut LAlloc> = child_allocs.iter_mut().collect();
        let mut child_req_and_alloc_refs: Vec<(&LReq, &mut LAlloc)> = Vec::with_capacity(child_reqs.len());
        for (r, mut a) in child_reqs.iter().zip(child_allocs.iter_mut()) {
            child_req_and_alloc_refs.push((r, a));
        }

        parent_reqs.clear();
        for _ in 0..num_parents {
            let p = LReq::linear_acc(&child_req_refs, 0.0, None);
            parent_reqs.push(p);
        }

        bench.iter(|| {
            for i in 0..num_parents {
                let parent = &parent_reqs[i];
                LAlloc::alloc_linear(&mut child_req_and_alloc_refs, parent,
                                     0.0, parent.size().size(), None, 0.0,
                                     None);
            }
        });
    }
}
