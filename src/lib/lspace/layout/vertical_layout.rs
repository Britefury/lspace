use layout::lreq::{LReq};
use layout::lalloc::{LAlloc};


pub fn requisition_x(child_reqs: &[&LReq]) -> LReq {
    return LReq::perpendicular_acc(child_reqs);
}

pub fn requisition_y(child_reqs: &[&LReq], y_spacing: f64,
                     ref_point_index: Option<usize>) -> LReq {
    return LReq::linear_acc(child_reqs, y_spacing, ref_point_index);
}

pub fn alloc_x(box_req: &LReq, box_alloc: &LAlloc, child_reqs: &[&LReq],
               child_allocs: &mut [&mut LAlloc]) {
    debug_assert!(child_reqs.len() == child_allocs.len());
    for i in 0..child_reqs.len() {
       child_allocs[i].alloc_from_region(child_reqs[i], box_alloc.pos_in_parent(),
                                         box_alloc.alloc_size(), box_alloc.ref_point());
    }
}

pub fn alloc_y(box_req: &LReq, box_alloc: &LAlloc, child_reqs: &[&LReq],
               child_allocs: &mut [&mut LAlloc], y_spacing: f64,
               ref_point_index: Option<usize>) {
    LAlloc::alloc_linear(child_reqs, child_allocs, box_req, box_alloc.pos_in_parent(),
                        box_alloc.alloc_size(), box_alloc.ref_point(), y_spacing, ref_point_index);
}

#[cfg(test)]
mod tests {
    extern crate rand;
    extern crate test;

    use std::mem;
    use self::rand::distributions::{Range, IndependentSample};
    use super::*;

    use layout::lreq::{LNatSize, LFlex, LReq};
    use layout::lalloc::{LAlloc};


    #[bench]
    fn bench_vertical_layout(bench: &mut test::Bencher) {
        let num_children = 100;
        let num_parents = 100;
        let num_repeats = 100;

        let natsize_type_range: Range<i32> = Range::new(0, 8);
        let size_range = Range::new(5.0, 25.0);
        let flex_type_range: Range<i32> = Range::new(0, 2);
        let flex_range = Range::new(1.0, 3.0);
        let mut rng = rand::thread_rng();

        let mut child_x_reqs = Vec::with_capacity(num_children);
        let mut child_y_reqs = Vec::with_capacity(num_children);
        let mut child_x_allocs = Vec::with_capacity(num_children);
        let mut child_y_allocs = Vec::with_capacity(num_children);
        let mut parent_x_reqs = Vec::with_capacity(num_parents);
        let mut parent_y_reqs = Vec::with_capacity(num_parents);
        let mut parent_x_allocs = Vec::with_capacity(num_parents);
        let mut parent_y_allocs = Vec::with_capacity(num_parents);

        for _ in 0..num_children {
            let size_x = LNatSize::new_size(size_range.ind_sample(&mut rng));
            let size_y = LNatSize::new_ref(size_range.ind_sample(&mut rng) * 0.5,
                                     size_range.ind_sample(&mut rng) * 0.5);
            let flex_x = match flex_type_range.ind_sample(&mut rng) {
                0 => LFlex::new_fixed(),
                1 => LFlex::new_flex(flex_range.ind_sample(&mut rng),
                                     0.0),
                _ => {panic!();},
            };
            let flex_y = LFlex::new_fixed();
            child_x_reqs.push(LReq::new(size_x, flex_x));
            child_y_reqs.push(LReq::new(size_y, flex_y));
            child_x_allocs.push(LAlloc::new_empty());
            child_y_allocs.push(LAlloc::new_empty());
        }

        let child_x_req_refs: Vec<&LReq> = child_x_reqs.iter().collect();
        let child_y_req_refs: Vec<&LReq> = child_y_reqs.iter().collect();
        let mut child_x_alloc_refs : Vec<&mut LAlloc> = child_x_allocs.iter_mut().collect();
        let mut child_y_alloc_refs : Vec<&mut LAlloc> = child_y_allocs.iter_mut().collect();

        for _ in 0..num_parents {
            parent_x_reqs.push(LReq::new_empty());
            parent_y_reqs.push(LReq::new_empty());
            parent_x_allocs.push(LAlloc::new_empty());
            parent_y_allocs.push(LAlloc::new_empty());
        }

        bench.iter(|| {
            for _ in 0..num_repeats {
                for i in 0..num_parents {
                    parent_x_reqs[i] = requisition_x(&child_x_req_refs);
                    parent_x_allocs[i] = LAlloc::new_from_req(&parent_x_reqs[i], 0.0);

                    alloc_x(&parent_x_reqs[i], &parent_x_allocs[i],
                            &child_x_req_refs, &mut child_x_alloc_refs);

                    parent_y_reqs[i] = requisition_y(&child_y_req_refs, 0.0, None);
                    parent_y_allocs[i] = LAlloc::new_from_req(&parent_y_reqs[i], 0.0);

                    alloc_y(&parent_y_reqs[i], &parent_y_allocs[i],
                            &child_y_req_refs, &mut child_y_alloc_refs, 0.0, None);
                }
            }
        });
    }
}
