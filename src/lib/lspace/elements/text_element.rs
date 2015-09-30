use std::rc::Rc;
use std::cell::RefCell;
use std::string::String;

use cairo::Context;

use layout::lalloc::LAlloc;
use geom::bbox2::BBox2;
use elements::element_ctx::{ElementContext};
use elements::element::{ElementReq, ElementAlloc, TElementLayout, TElement};


pub struct TextElement {
    req: Rc<ElementReq>,
    alloc: ElementAlloc,
    text: String,
}

impl TextElement {
    pub fn new(text: &String, cairo_ctx: &Context, elem_ctx: &RefCell<ElementContext>) -> TextElement {
        let req = elem_ctx.borrow_mut().text_shared_req(text.clone(), cairo_ctx);
        return TextElement{text: text.clone(),
                           req: req,
                           alloc: ElementAlloc::new()};
    }
}

impl TElementLayout for TextElement {
    fn element_req(&self) -> &ElementReq {
        return &*self.req;
    }

    fn element_alloc(&self) -> &ElementAlloc {
        return &self.alloc;
    }

    fn element_req_and_mut_alloc(&mut self) -> (&ElementReq, &mut ElementAlloc) {
        return (&*self.req, &mut self.alloc);
    }
}

impl TElement for TextElement {
    fn draw_self(&self, cairo_ctx: &Context, visible_region: &BBox2) {
        let y = match self.alloc.y_alloc.ref_point() {
            None => 0.0,
            Some(ref_point) => ref_point
        };
        cairo_ctx.move_to(0.0, y);
        cairo_ctx.show_text(self.text.as_str());
    }

    fn draw(&self, cairo_ctx: &Context, visible_region: &BBox2) {
        self.draw_self(cairo_ctx, visible_region);
    }

    fn update_x_req(&mut self) {
        // Nothing to do; requisition is shared
    }

    fn allocate_x(&mut self) {
        // Nothing to do; no children
    }

    fn update_y_req(&mut self) {
        // Nothing to do; requisition is shared
    }

    fn allocate_y(&mut self) {
        // Nothing to do; no children
    }
}
