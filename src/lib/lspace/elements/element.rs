use cairo::Context;

use std::rc::Rc;
use std::cell::RefCell;

use layout::lreq::LReq;
use layout::lalloc::LAlloc;
use geom::bbox2::BBox2;

const LAYOUT_FLAG_X_REQ_DIRTY: u8       = 0b00000001;
const LAYOUT_FLAG_Y_REQ_DIRTY: u8       = 0b00000010;
const LAYOUT_FLAG_X_ALLOC_DIRTY: u8     = 0b00000100;
const LAYOUT_FLAG_Y_ALLOC_DIRTY: u8     = 0b00001000;
const LAYOUT_FLAGS_ALL_DIRTY: u8        = 0b00001111;
const LAYOUT_FLAGS_ALL_CLEAN: u8        = 0b00000000;


pub struct ElementChildRef {
    x: Box<TElement>
}


impl ElementChildRef {
    pub fn new<T: TElement + 'static>(x: T) -> ElementChildRef {
        return ElementChildRef{x: Box::new(x)};
    }

    pub fn get<'a>(&'a self) -> &'a TElement {
        return &*self.x;
    }

    pub fn get_mut<'a>(&'a mut self) -> &'a mut TElement {
        return &mut *self.x;
    }
}


pub struct ElementReq {
    pub x_req: LReq,
    pub y_req: LReq,
}

impl ElementReq {
    pub fn new() -> ElementReq {
        return ElementReq{x_req: LReq::new_empty(), y_req: LReq::new_empty()};
    }

    pub fn new_from_reqs(x_req: LReq, y_req: LReq) -> ElementReq {
        return ElementReq{x_req: x_req, y_req: y_req};
    }
}


pub struct ElementAlloc {
    pub x_alloc: LAlloc,
    pub y_alloc: LAlloc,
    pub layout_flags: u8,
}

impl ElementAlloc {
    pub fn new() -> ElementAlloc {
        return ElementAlloc{x_alloc: LAlloc::new_empty(), y_alloc: LAlloc::new_empty(),
                            layout_flags: LAYOUT_FLAGS_ALL_CLEAN};
    }
}


pub trait TElementLayout {
    fn element_req(&self) -> &ElementReq;
    fn element_alloc(&self) -> &ElementAlloc;
    fn element_req_and_mut_alloc(&mut self) -> (&ElementReq, &mut ElementAlloc);

    fn x_req(&self) -> &LReq {
        return &self.element_req().x_req;
    }

    fn x_alloc(&self) -> &LAlloc {
        return &self.element_alloc().x_alloc;
    }

    fn x_req_and_mut_alloc(&mut self) -> (&LReq, &mut LAlloc) {
        let ra = self.element_req_and_mut_alloc();
        return (&ra.0.x_req, &mut ra.1.x_alloc);
    }

    fn y_req(&self) -> &LReq {
        return &self.element_req().y_req;
    }

    fn y_alloc(&self) -> &LAlloc {
        return &self.element_alloc().y_alloc;
    }

    fn y_req_and_mut_alloc(&mut self) -> (&LReq, &mut LAlloc) {
        let ra = self.element_req_and_mut_alloc();
        return (&ra.0.y_req, &mut ra.1.y_alloc);
    }
}


pub trait TElement : TElementLayout {
    fn draw_self(&self, cairo_ctx: &Context, visible_region: &BBox2) {
    }

    fn draw(&self, cairo_ctx: &Context, visible_region: &BBox2);

    fn update_x_req(&mut self) {
    }

    fn allocate_x(&mut self) {
    }

    fn update_y_req(&mut self) {
    }

    fn allocate_y(&mut self) {
    }
}
