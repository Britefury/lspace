use layout::lreq::{LReq};
use layout::lalloc::{LAlloc};


#[derive(Debug, PartialEq, Copy, Clone)]
pub enum FlowIndent {
    NoIndent,
    First{indent: f64},
    ExceptFirst{indent: f64}
}


pub struct FlowLine {
    y_req: LReq,
    pos_x_in_parent: f64,
    pos_y_in_parent: f64,
    start: usize,
    end: usize
}

impl FlowLine {
    fn new_x(pos_x_in_parent: f64, start: usize, end: usize) -> FlowLine {
        return FlowLine{y_req: LReq::new_empty(),
                        pos_x_in_parent: pos_x_in_parent, pos_y_in_parent: 0.0,
                        start: start, end: end};
    }

}


pub fn requisition_x(child_x_reqs: &[&LReq], x_spacing: f64, indentation: FlowIndent) -> LReq {
    let (one_line, sep_lines) = match indentation {
        FlowIndent::NoIndent => {
            let one_line = LReq::linear_acc(child_x_reqs, x_spacing, None);
            let sep_lines = LReq::perpendicular_acc(child_x_reqs);
            (one_line, sep_lines)
        },

        FlowIndent::First{indent} => {
            let one_line = LReq::linear_acc(child_x_reqs, x_spacing, None);
            let indented : Vec<LReq> = child_x_reqs.iter().enumerate().map(|(i,ref x)| {
                    if i == 0 {
                        x.indent(indent)
                    } else {
                        ***x
                    }
                }).collect();
            let indented_refs : Vec<&LReq> = indented.iter().collect();
            let sep_lines = LReq::perpendicular_acc(&indented_refs);
            (one_line.indent(indent), sep_lines)
        },

        FlowIndent::ExceptFirst{indent} => {
            let indent_req = LReq::new_fixed_size(indent);
            let one_line = LReq::linear_acc(child_x_reqs, x_spacing, None);
            let indented : Vec<LReq> = child_x_reqs.iter().enumerate().map(|(i,ref x)| {
                    if i != 0 {
                        x.indent(indent)
                    } else {
                        ***x
                    }
                }).collect();
            let indented_refs : Vec<&LReq> = indented.iter().collect();
            let sep_lines = LReq::perpendicular_acc(&indented_refs);
            (one_line, sep_lines)
        }
    };

    let min_size = sep_lines.min_size();
    let preferred_size = one_line.size().size();

    return LReq::new_flex_size_min(preferred_size, min_size, one_line.flex().stretch());
}

pub fn alloc_x(box_x_req: &LReq, box_x_alloc: &LAlloc, child_x_reqs: &Vec<&LReq>,
               child_x_allocs: &mut Vec<&mut LAlloc>, x_spacing: f64,
               indentation: FlowIndent) -> Vec<FlowLine> {
    if box_x_req.size().size() <= box_x_alloc.alloc_size() {
        let (indent, line_req, line_alloc) = match indentation {
            FlowIndent::First{indent} => (indent, box_x_req.dedent(indent),
                                          box_x_alloc.indent(indent)),
            _ => (0.0, *box_x_req, *box_x_alloc)
        };
        LAlloc::alloc_linear(child_x_reqs, child_x_allocs, box_x_req, line_alloc.pos_in_parent(),
                             line_alloc.alloc_size(), line_alloc.ref_point(), x_spacing, None);
        let line = FlowLine::new_x(indent, 0, child_x_reqs.len());
        return vec![line];
    } else {
        let n = child_x_reqs.len();
        let mut x = 0.0;
        let mut lines : Vec<FlowLine> = Vec::new();
        let mut line_i_0 = 0;
        let mut line_x = 0.0;
        let mut n_lines = 0;

        for i in 0..n {
            if x > box_x_alloc.alloc_size() || i == 0 {
                if i > 0 {
                    let line_alloc = box_x_alloc.indent(line_x);
                    // Record the existing line
                    lines.push(FlowLine::new_x(line_x, line_i_0, i));
                    LAlloc::alloc_linear(&child_x_reqs[line_i_0..i],
                                         &mut child_x_allocs[line_i_0..i],
                                         &box_x_req.dedent(line_x), line_alloc.pos_in_parent(),
                                         line_alloc.alloc_size(), line_alloc.ref_point(),
                                         x_spacing, None);
                }

                // Start a new line
                line_i_0 = i;
                line_x = match indentation {
                    FlowIndent::First{indent} if n_lines == 0 => indent,
                    FlowIndent::ExceptFirst{indent} if n_lines != 0 => indent,
                    _ => 0.0
                };
                n_lines = n_lines + 1;

                x = line_x;
            }

            if i > line_i_0 {
                // Not the first elmenet in the line
                x = x + x_spacing;
            }
            x = x + child_x_reqs[i].size().size();
        }

        {
            let line_alloc = box_x_alloc.indent(line_x);

            lines.push(FlowLine::new_x(line_x, line_i_0, n));
            LAlloc::alloc_linear(&child_x_reqs[line_i_0..n], &mut child_x_allocs[line_i_0..n],
                                 &box_x_req.dedent(line_x), line_alloc.pos_in_parent(),
                                 line_alloc.alloc_size(), line_alloc.ref_point(), x_spacing, None);
        }

        return lines;
    }
}

pub fn requisition_y(child_y_reqs: &[&LReq], y_spacing: f64, lines: &mut Vec<FlowLine>) -> LReq {
    for i in 0..lines.len() {
        let line_y_req = {
            let line = &lines[i];
            LReq::perpendicular_acc(&child_y_reqs[line.start..line.end])
        };
        lines[i].y_req = line_y_req;
    }

    let line_y_reqs: Vec<&LReq> = lines.iter().map(|x| &x.y_req).collect();
    return LReq::linear_acc(&line_y_reqs, y_spacing, None);
}

pub fn alloc_y(box_y_req: &LReq, box_y_alloc: &LAlloc, child_y_reqs: &[&LReq],
               child_y_allocs: &mut [&mut LAlloc], y_spacing: f64,
               lines: &mut Vec<FlowLine>) {
    let mut line_y_allocs : Vec<LAlloc> = (0..lines.len()).map(|_| LAlloc::new_empty()).collect();
    {
        // Allocate lines
        let line_y_reqs: Vec<&LReq> = lines.iter().map(|x| &x.y_req).collect();
        let mut line_y_alloc_refs : Vec<&mut LAlloc> = line_y_allocs.iter_mut().collect();
        LAlloc::alloc_linear(&line_y_reqs, &mut line_y_alloc_refs, box_y_req,
                             box_y_alloc.pos_in_parent(), box_y_alloc.alloc_size(),
                             box_y_alloc.ref_point(), y_spacing, None);

        // Allocate children
        for l in 0..lines.len() {
            let line = &lines[l];
            let line_y_alloc = &line_y_alloc_refs[l];
            for i in line.start..line.end {
                child_y_allocs[i].alloc_from_region(child_y_reqs[i], line_y_alloc.pos_in_parent(),
                                                    line_y_alloc.alloc_size(),
                                                    line_y_alloc.ref_point());
            }
        }
    }

    for l in 0..lines.len() {
        lines[l].pos_y_in_parent = line_y_allocs[l].pos_in_parent();
    }
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


    fn f_layout(x_reqs: &Vec<&LReq>, y_reqs: &Vec<&LReq>, x_spacing: f64,
                y_spacing: f64, indentation: FlowIndent, width: f64) ->
                    (LReq, LReq, Vec<LAlloc>, Vec<LAlloc>, Vec<FlowLine>) {
        let n = x_reqs.len();
        assert_eq!(n, y_reqs.len());

        let mut x_allocs : Vec<LAlloc> = (0..n).map(|_| LAlloc::new_empty()).collect();
        let mut y_allocs : Vec<LAlloc> = (0..n).map(|_| LAlloc::new_empty()).collect();

        let (box_x_req, box_y_req, lines) = {
            let mut x_alloc_refs : Vec<&mut LAlloc> = x_allocs.iter_mut().collect();
            let mut y_alloc_refs : Vec<&mut LAlloc> = y_allocs.iter_mut().collect();

            let box_x_req = requisition_x(x_reqs, x_spacing, indentation);
            let box_x_alloc = LAlloc::new(0.0, width, width);

            let mut lines = alloc_x(&box_x_req, &box_x_alloc, x_reqs, &mut x_alloc_refs,
                                    x_spacing, indentation);

            let box_y_req = requisition_y(y_reqs, y_spacing, &mut lines);
            let box_y_alloc = LAlloc::new_from_req(&box_y_req, 0.0);

            alloc_y(&box_y_req, &box_y_alloc, y_reqs, &mut y_alloc_refs, y_spacing, &mut lines);

            (box_x_req, box_y_req, lines)
        };

        return (box_x_req, box_y_req, x_allocs, y_allocs, lines);
    }

    #[test]
    fn test_flow_layout() {
        let ch0x = LReq::new_fixed_size(10.0);
        let ch1x = LReq::new_fixed_size(20.0);
        let ch2x = LReq::new_fixed_size(30.0);
        let ch0y = LReq::new_fixed_ref(6.0, 4.0);
        let ch1y = LReq::new_fixed_ref(8.0, 2.0);
        let ch2y = LReq::new_fixed_ref(5.0, 6.0);

        let (box_x_req, box_y_req, x_allocs, y_allocs, lines) = f_layout(
            &vec![&ch0x, &ch1x, &ch2x], &vec![&ch0y, &ch1y, &ch2y],
            0.0, 0.0, FlowIndent::NoIndent, 60.0);

        assert_eq!(box_x_req, LReq::new_flex_size(60.0, 30.0, 0.0));
        assert_eq!(box_y_req, LReq::new_fixed_size(14.0));

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].y_req, LReq::new_fixed_ref(8.0, 6.0));

        assert_eq!(x_allocs[0], LAlloc::new(0.0, 10.0, 10.0));
        assert_eq!(x_allocs[1], LAlloc::new(10.0, 20.0, 20.0));
        assert_eq!(x_allocs[2], LAlloc::new(30.0, 30.0, 30.0));

        assert_eq!(y_allocs[0], LAlloc::new_ref(2.0, 10.0, 10.0, 6.0));
        assert_eq!(y_allocs[1], LAlloc::new_ref(0.0, 10.0, 10.0, 8.0));
        assert_eq!(y_allocs[2], LAlloc::new_ref(3.0, 11.0, 11.0, 5.0));
    }

    #[test]
    fn test_flow_layout_large() {
        let n = 100;

        let x_reqs: Vec<LReq> = (0..n).map(|_| LReq::new_fixed_size(10.0)).collect();
        let y_reqs: Vec<LReq> = (0..n).map(|i| LReq::new_fixed_ref(6.0+(i%2) as f64,
                                                          3.0+(1-i%2) as f64)).collect();
        let x_req_refs: Vec<&LReq> = x_reqs.iter().collect();
        let y_req_refs: Vec<&LReq> = y_reqs.iter().collect();

        {
            let (box_x_req, box_y_req, x_allocs, y_allocs, lines) = f_layout(
                &x_req_refs, &y_req_refs,
                0.0, 0.0, FlowIndent::NoIndent, 1000.0);

            assert_eq!(lines.len(), 1);
            assert_eq!(lines[0].y_req, LReq::new_fixed_ref(7.0, 4.0));

            assert_eq!(box_x_req, LReq::new_flex_size(1000.0, 990.0, 0.0));
            assert_eq!(box_y_req, LReq::new_fixed_size(11.0));

            for i in 0..n {
                assert_eq!(x_allocs[i], LAlloc::new((i*10) as f64, 10.0, 10.0));
                assert_eq!(y_allocs[i], LAlloc::new_ref((1-i%2) as f64, 10.0, 10.0, 6.0+(i%2) as f64));
            }
        }

        {
            let (box_x_req, box_y_req, x_allocs, y_allocs, lines) = f_layout(
                &x_req_refs, &y_req_refs,
                0.0, 0.0, FlowIndent::NoIndent, 500.0);

            assert_eq!(lines.len(), 2);
            assert_eq!(lines[0].y_req, LReq::new_fixed_ref(7.0, 4.0));
            assert_eq!(lines[1].y_req, LReq::new_fixed_ref(7.0, 4.0));

            assert_eq!(box_x_req, LReq::new_flex_size(1000.0, 990.0, 0.0));
            assert_eq!(box_y_req, LReq::new_fixed_size(22.0));

            for i in 0..51 {
                assert_eq!(x_allocs[i], LAlloc::new((i*10) as f64, 10.0, 10.0));
                assert_eq!(y_allocs[i], LAlloc::new_ref((1-i%2) as f64, 10.0, 10.0,
                                                        6.0+(i%2) as f64));
            }
            for i in 51..n {
                assert_eq!(x_allocs[i], LAlloc::new(((i-51)*10) as f64, 10.0, 10.0));
                assert_eq!(y_allocs[i], LAlloc::new_ref(11.0+(1-i%2) as f64, 10.0, 10.0,
                                                        6.0+(i%2) as f64));
            }
        }

        {
            let (box_x_req, box_y_req, x_allocs, y_allocs, lines) = f_layout(
                &x_req_refs, &y_req_refs,
                0.0, 0.0, FlowIndent::NoIndent, 250.0);

            assert_eq!(lines.len(), 4);
            assert_eq!(lines[0].y_req, LReq::new_fixed_ref(7.0, 4.0));
            assert_eq!(lines[1].y_req, LReq::new_fixed_ref(7.0, 4.0));
            assert_eq!(lines[2].y_req, LReq::new_fixed_ref(7.0, 4.0));
            assert_eq!(lines[3].y_req, LReq::new_fixed_ref(7.0, 4.0));

            assert_eq!(box_x_req, LReq::new_flex_size(1000.0, 990.0, 0.0));
            assert_eq!(box_y_req, LReq::new_fixed_size(44.0));

            for i in 0..26 {
                assert_eq!(x_allocs[i], LAlloc::new((i*10) as f64, 10.0, 10.0));
                assert_eq!(y_allocs[i], LAlloc::new_ref((1-i%2) as f64, 10.0, 10.0,
                                                        6.0+(i%2) as f64));
            }
            for i in 26..52 {
                assert_eq!(x_allocs[i], LAlloc::new(((i-26)*10) as f64, 10.0, 10.0));
                assert_eq!(y_allocs[i], LAlloc::new_ref(11.0+(1-i%2) as f64, 10.0, 10.0,
                                                        6.0+(i%2) as f64));
            }
            for i in 52..78 {
                assert_eq!(x_allocs[i], LAlloc::new(((i-52)*10) as f64, 10.0, 10.0));
                assert_eq!(y_allocs[i], LAlloc::new_ref(22.0+(1-i%2) as f64, 10.0, 10.0,
                                                        6.0+(i%2) as f64));
            }
            for i in 78..n {
                assert_eq!(x_allocs[i], LAlloc::new(((i-78)*10) as f64, 10.0, 10.0));
                assert_eq!(y_allocs[i], LAlloc::new_ref(33.0+(1-i%2) as f64, 10.0, 10.0,
                                                        6.0+(i%2) as f64));
            }
        }

        {
            let (box_x_req, box_y_req, x_allocs, y_allocs, lines) = f_layout(
                &x_req_refs, &y_req_refs,
                0.0, 0.0, FlowIndent::First{indent:15.0}, 250.0);

            assert_eq!(lines.len(), 4);
            assert_eq!(lines[0].y_req, LReq::new_fixed_ref(7.0, 4.0));
            assert_eq!(lines[1].y_req, LReq::new_fixed_ref(7.0, 4.0));
            assert_eq!(lines[2].y_req, LReq::new_fixed_ref(7.0, 4.0));
            assert_eq!(lines[3].y_req, LReq::new_fixed_ref(7.0, 4.0));

            assert_eq!(box_x_req, LReq::new_flex_size(1015.0, 990.0, 0.0));
            assert_eq!(box_y_req, LReq::new_fixed_size(44.0));

            for i in 0..24 {
                assert_eq!(x_allocs[i], LAlloc::new(15.0+(i*10) as f64, 10.0, 10.0));
                assert_eq!(y_allocs[i], LAlloc::new_ref((1-i%2) as f64, 10.0, 10.0,
                                                        6.0+(i%2) as f64));
            }
            for i in 24..50 {
                assert_eq!(x_allocs[i], LAlloc::new(((i-24)*10) as f64, 10.0, 10.0));
                assert_eq!(y_allocs[i], LAlloc::new_ref(11.0+(1-i%2) as f64, 10.0, 10.0,
                                                        6.0+(i%2) as f64));
            }
            for i in 50..76 {
                assert_eq!(x_allocs[i], LAlloc::new(((i-50)*10) as f64, 10.0, 10.0));
                assert_eq!(y_allocs[i], LAlloc::new_ref(22.0+(1-i%2) as f64, 10.0, 10.0,
                                                        6.0+(i%2) as f64));
            }
            for i in 76..n {
                assert_eq!(x_allocs[i], LAlloc::new(((i-76)*10) as f64, 10.0, 10.0));
                assert_eq!(y_allocs[i], LAlloc::new_ref(33.0+(1-i%2) as f64, 10.0, 10.0,
                                                        6.0+(i%2) as f64));
            }
        }

        {
            let (box_x_req, box_y_req, x_allocs, y_allocs, lines) = f_layout(
                &x_req_refs, &y_req_refs,
                0.0, 0.0, FlowIndent::ExceptFirst{indent:15.0}, 300.0);

            assert_eq!(lines.len(), 4);
            assert_eq!(lines[0].y_req, LReq::new_fixed_ref(7.0, 4.0));
            assert_eq!(lines[1].y_req, LReq::new_fixed_ref(7.0, 4.0));
            assert_eq!(lines[2].y_req, LReq::new_fixed_ref(7.0, 4.0));
            assert_eq!(lines[3].y_req, LReq::new_fixed_ref(7.0, 4.0));

            assert_eq!(box_x_req, LReq::new_flex_size(1000.0, 975.0, 0.0));
            assert_eq!(box_y_req, LReq::new_fixed_size(44.0));

            for i in 0..31 {
                assert_eq!(x_allocs[i], LAlloc::new((i*10) as f64, 10.0, 10.0));
                assert_eq!(y_allocs[i], LAlloc::new_ref((1-i%2) as f64, 10.0, 10.0,
                                                        6.0+(i%2) as f64));
            }
            for i in 31..60 {
                assert_eq!(x_allocs[i], LAlloc::new(15.0+((i-31)*10) as f64, 10.0, 10.0));
                assert_eq!(y_allocs[i], LAlloc::new_ref(11.0+(1-i%2) as f64, 10.0, 10.0,
                                                        6.0+(i%2) as f64));
            }
            for i in 60..89 {
                assert_eq!(x_allocs[i], LAlloc::new(15.0+((i-60)*10) as f64, 10.0, 10.0));
                assert_eq!(y_allocs[i], LAlloc::new_ref(22.0+(1-i%2) as f64, 10.0, 10.0,
                                                        6.0+(i%2) as f64));
            }
            for i in 89..n {
                assert_eq!(x_allocs[i], LAlloc::new(15.0+((i-89)*10) as f64, 10.0, 10.0));
                assert_eq!(y_allocs[i], LAlloc::new_ref(33.0+(1-i%2) as f64, 10.0, 10.0,
                                                        6.0+(i%2) as f64));
            }
        }
    }

    #[bench]
    fn bench_flow_layout(bench: &mut test::Bencher) {
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
                    parent_x_reqs[i] = requisition_x(&child_x_req_refs, 0.0, FlowIndent::NoIndent);
                    parent_x_allocs[i] = LAlloc::new(0.0, 500.0, 500.0);

                    let mut lines = alloc_x(&parent_x_reqs[i], &parent_x_allocs[i],
                            &child_x_req_refs, &mut child_x_alloc_refs, 0.0, FlowIndent::NoIndent);

                    parent_y_reqs[i] = requisition_y(&child_y_req_refs, 0.0, &mut lines);
                    parent_y_allocs[i] = LAlloc::new_from_req(&parent_y_reqs[i], 0.0);

                    alloc_y(&parent_y_reqs[i], &parent_y_allocs[i],
                            &child_y_req_refs, &mut child_y_alloc_refs, 0.0,
                            &mut lines);
                }
            }
        });
    }
}
