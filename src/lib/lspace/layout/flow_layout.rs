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



pub fn requisition_x(child_reqs: &[&LReq], x_spacing: f64, indentation: FlowIndent) -> LReq {
    let (one_line, sep_lines) = match indentation {
        FlowIndent::NoIndent => {
            let one_line = LReq::linear_acc(child_reqs, x_spacing, None);
            let sep_lines = LReq::perpendicular_acc(child_reqs);
            (one_line, sep_lines)
        },

        FlowIndent::First{indent} => {
            let one_line = LReq::linear_acc(child_reqs, x_spacing, None);
            let indented : Vec<LReq> = child_reqs.iter().enumerate().map(|(i,ref x)| {
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
            let one_line = LReq::linear_acc(child_reqs, x_spacing, None);
            let indented : Vec<LReq> = child_reqs.iter().enumerate().map(|(i,ref x)| {
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

pub fn requisition_y(child_reqs: &[&LReq], y_spacing: f64, lines: &mut Vec<FlowLine>) -> LReq {
    for i in 0..lines.len() {
        let line_req = {
            let line = &lines[i];
            LReq::perpendicular_acc(&child_reqs[line.start..line.end])
        };
        lines[i].y_req = line_req;
    }

    let line_reqs: Vec<&LReq> = lines.iter().map(|x| &x.y_req).collect();
    return LReq::linear_acc(&line_reqs, y_spacing, None);
}

pub fn alloc_x(box_req: &LReq, box_alloc: &LAlloc, child_reqs: &Vec<&LReq>,
               child_allocs: &mut Vec<&mut LAlloc>, x_spacing: f64,
               indentation: FlowIndent) -> Vec<FlowLine> {
    if box_req.size().size() > box_alloc.alloc_size() {
        let (indent, line_req, line_alloc) = match indentation {
            FlowIndent::First{indent} => (indent, box_req.dedent(indent), box_alloc.indent(indent)),
            _ => (0.0, *box_req, *box_alloc)
        };
        LAlloc::alloc_linear(child_reqs, child_allocs, box_req, line_alloc.pos_in_parent(),
                             line_alloc.alloc_size(),
                             line_alloc.ref_point(), x_spacing, None);
        let line = FlowLine::new_x(indent, 0, child_reqs.len());
        return vec![line];
    } else {
        let n = child_reqs.len();
        let mut x = 0.0;
        let mut lines : Vec<FlowLine> = Vec::new();
        let mut line_i_0 = 0;
        let mut line_x = 0.0;
        let mut n_lines = 0;

        for i in 0..n {
            if x > box_alloc.alloc_size() || i == 0 {
                if i > 0 {
                    let line_alloc = box_alloc.indent(line_x);
                    // Record the existing line
                    lines.push(FlowLine::new_x(line_x, line_i_0, i));
                    LAlloc::alloc_linear(&child_reqs[line_i_0..i], &mut child_allocs[line_i_0..i],
                                         &box_req.dedent(line_x), line_alloc.pos_in_parent(),
                                         line_alloc.alloc_size(),
                                         line_alloc.ref_point(), x_spacing, None);
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
            x = x + child_reqs[i].size().size();
        }

        {
            let line_alloc = box_alloc.indent(line_x);

            lines.push(FlowLine::new_x(line_x, line_i_0, n));
            LAlloc::alloc_linear(&child_reqs[line_i_0..n], &mut child_allocs[line_i_0..n],
                                 &box_req.dedent(line_x), line_alloc.pos_in_parent(),
                                 line_alloc.alloc_size(),
                                 line_alloc.ref_point(), x_spacing, None);
        }

        return lines;
    }
}

pub fn alloc_y(box_req: &LReq, box_alloc: &LAlloc, child_reqs: &[&LReq],
               child_allocs: &mut [&mut LAlloc], y_spacing: f64,
               lines: &mut Vec<FlowLine>) {
    let mut line_allocs : Vec<LAlloc> = (0..lines.len()).map(|_| LAlloc::new_empty()).collect();
    {
        // Allocate lines
        let line_reqs: Vec<&LReq> = lines.iter().map(|x| &x.y_req).collect();
        let mut line_alloc_refs : Vec<&mut LAlloc> = line_allocs.iter_mut().collect();
        LAlloc::alloc_linear(&line_reqs, &mut line_alloc_refs, box_req, box_alloc.pos_in_parent(),
                             box_alloc.alloc_size(), box_alloc.ref_point(), y_spacing, None);

        // Allocate children
        for l in 0..lines.len() {
            let line = &lines[l];
            let line_alloc = &line_alloc_refs[l];
            for i in line.start..line.end {
                child_allocs[i].alloc_from_region(child_reqs[i], line_alloc.pos_in_parent(),
                                                  line_alloc.alloc_size(), line_alloc.ref_point());
            }
        }
    }

    for l in 0..lines.len() {
        lines[l].pos_y_in_parent = line_allocs[l].pos_in_parent();
    }
}
