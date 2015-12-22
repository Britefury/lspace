use cairo::Context;

use std::cell::{Ref, RefMut};

use layout::lreq::LReq;
use layout::lalloc::LAlloc;
use layout::flow_layout;
use geom::bbox2::BBox2;

use elements::element_layout::{ElementReq, ElementAlloc};
use elements::element::{TElement, ElementRef, ElemBorrow, ElemBorrowMut};
use elements::container::TContainerElement;


pub struct FlowElement {
    req: ElementReq,
    alloc: ElementAlloc,
    children: Vec<ElementRef>,
    x_spacing: f64,
    y_spacing: f64,
    indentation: flow_layout::FlowIndent,
    lines: Vec<flow_layout::FlowLine>
}

impl FlowElement {
    pub fn new(children: Vec<ElementRef>, x_spacing: f64, y_spacing: f64,
               indentation: flow_layout::FlowIndent) -> FlowElement {
        return FlowElement{req: ElementReq::new(), alloc: ElementAlloc::new(),
                           children: children, x_spacing: x_spacing, y_spacing: y_spacing,
                           indentation: indentation, lines: Vec::new()};
    }
}

impl TElement for FlowElement {
    fn element_req(&self) -> &ElementReq {
        return &self.req;
    }

    fn element_alloc(&self) -> &ElementAlloc {
        return &self.alloc;
    }

    fn element_req_and_mut_alloc(&mut self) -> (&ElementReq, &mut ElementAlloc) {
        return (&self.req, &mut self.alloc);
    }

    fn draw(&self, cairo_ctx: &Context, visible_region: &BBox2) {
        self.draw_self(cairo_ctx, visible_region);
        self.draw_children(cairo_ctx, visible_region);
    }

    fn update_x_req(&mut self) {
        self.update_children_x_req();
        let child_refs: Vec<ElemBorrow> = self.children.iter().map(|c| c.get()).collect();
        let child_x_reqs: Vec<&LReq> = child_refs.iter().map(|c| c.x_req()).collect();
        self.req.x_req = flow_layout::requisition_x(&child_x_reqs, self.x_spacing, self.indentation);
    }

    fn allocate_x(&mut self) {
        {
            let mut child_refs: Vec<ElemBorrowMut> = self.children.iter_mut().map(|c| c.get_mut()).collect();
            let mut x_pairs: Vec<(&LReq, &mut LAlloc)> = child_refs.iter_mut().map(
                    |c| c.x_req_and_mut_alloc()).collect();
            self.lines = flow_layout::alloc_x(&self.req.x_req,
                    &self.alloc.x_alloc.without_position(),
                    &mut x_pairs, self.x_spacing, self.indentation);

        }
        self.allocate_children_x();
    }

    fn update_y_req(&mut self) {
        self.update_children_y_req();
        let child_refs: Vec<ElemBorrow> = self.children.iter().map(|c| c.get()).collect();
        let child_y_reqs: Vec<&LReq> = child_refs.iter().map(|c| c.y_req()).collect();
        self.req.y_req = flow_layout::requisition_y(&child_y_reqs, self.y_spacing, &mut self.lines);
    }

    fn allocate_y(&mut self) {
        {
            let mut child_refs: Vec<ElemBorrowMut> = self.children.iter_mut().map(|c| c.get_mut()).collect();
            let mut y_pairs: Vec<(&LReq, &mut LAlloc)> = child_refs.iter_mut().map(
                    |c| c.y_req_and_mut_alloc()).collect();
            flow_layout::alloc_y(&self.req.y_req,
                &self.alloc.y_alloc.without_position(),
                &mut y_pairs, self.y_spacing, &mut self.lines);
        }
        self.allocate_children_y();
    }
}


impl TContainerElement for FlowElement {
    fn children<'a>(&'a self) -> &'a Vec<ElementRef> {
        return &self.children;
    }

    fn children_mut<'a>(&'a mut self) -> &'a mut Vec<ElementRef> {
        return &mut self.children;
    }
}
