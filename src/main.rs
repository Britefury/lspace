#![cfg_attr(not(feature = "gtk_3_10"), allow(unused_variables, unused_mut))]
#![feature(convert)]

extern crate gtk;
extern crate cairo;
extern crate lspace;

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::string::String;

use gtk::traits::*;
use gtk::signal::Inhibit;
use cairo::{Context, RectangleInt};

use lspace::layout::lreq::{LReq};
use lspace::layout::lalloc::{LAlloc};
use lspace::layout::lbox::{LBox};



const LAYOUT_FLAG_X_REQ_DIRTY: u8       = 0b00000001;
const LAYOUT_FLAG_Y_REQ_DIRTY: u8       = 0b00000010;
const LAYOUT_FLAG_X_ALLOC_DIRTY: u8     = 0b00000100;
const LAYOUT_FLAG_Y_ALLOC_DIRTY: u8     = 0b00001000;
const LAYOUT_FLAGS_ALL_DIRTY: u8        = 0b00001111;
const LAYOUT_FLAGS_ALL_CLEAN: u8        = 0b00000000;



struct ElementLayout {
    l_box: LBox,
    layout_flags: u8,
}

impl ElementLayout {
    fn new() -> ElementLayout {
        return ElementLayout{l_box: LBox::new_empty(), layout_flags: LAYOUT_FLAGS_ALL_DIRTY
        };
    }
}


#[derive(Copy, Clone, Debug)]
struct SharedReq {
    x_req: LReq,
    y_req: LReq,
    layout_flags: u8,
}

impl SharedReq {
    fn new(x_req: LReq, y_req: LReq) -> SharedReq {
        return SharedReq{x_req: x_req, y_req: y_req, layout_flags: LAYOUT_FLAGS_ALL_CLEAN};
    }
}

struct ElementLayoutWithSharedReq {
    x_alloc: LAlloc,
    y_alloc: LAlloc,
    layout_flags: u8,
}

impl ElementLayoutWithSharedReq {
    fn new() -> ElementLayoutWithSharedReq {
        return ElementLayoutWithSharedReq{x_alloc: LAlloc::new_empty(), y_alloc: LAlloc::new_empty(),
                                          layout_flags: LAYOUT_FLAGS_ALL_DIRTY};
    }
}


trait TElement {
    fn draw_self(&self, cairo_ctx: &Context) {
    }

    fn draw(&self, cairo_ctx: &Context);

    fn x_req(&self) -> &LReq;
    fn x_alloc(&self) -> &LAlloc;
    fn x_alloc_mut(&mut self) -> &mut LAlloc;
    fn y_req(&self) -> &LReq;
    fn y_alloc(&self) -> &LAlloc;
    fn y_alloc_mut(&mut self) -> &mut LAlloc;

    fn update_x_req(&mut self) {
    }

    fn allocate_x(&mut self) {
    }

    fn update_y_req(&mut self) {
    }

    fn allocate_y(&mut self) {
    }

}

trait TBranchElement : TElement {
    fn children<'a>(&'a self) -> &'a Vec<Box<TElement>>;
    fn children_mut<'a>(&'a mut self) -> &'a mut Vec<Box<TElement>>;


    fn draw_children(&self, cairo_ctx: &Context) {
        for child in self.children() {
            let xa = child.x_alloc();
            let ya = child.y_alloc();
            cairo_ctx.save();
            cairo_ctx.translate(xa.pos_in_parent(), xa.pos_in_parent());
            child.draw(cairo_ctx);
            cairo_ctx.restore();
        }
    }

    fn update_children_x_req(&mut self) {
        for child in self.children_mut() {
            child.update_x_req();
        }
    }

    fn update_children_y_req(&mut self) {
        for child in self.children_mut() {
            child.update_x_req();
        }
    }

    fn allocate_children_x(&mut self) {
        for child in self.children_mut() {
            child.allocate_x();
        }
    }

    fn allocate_children_y(&mut self) {
        for child in self.children_mut() {
            child.allocate_y();
        }
    }
}



struct TextElement {
    layout: ElementLayoutWithSharedReq,
    req: Rc<SharedReq>,
    text: String,
}

impl TextElement {
    fn new(text: &String, cairo_ctx: &Context, req_table: &mut HashMap<String, Rc<SharedReq>>) -> TextElement {
        let req_entry = req_table.entry(text.clone());
        let req = match req_entry {
            Entry::Vacant(v) => {
                let font_extents = cairo_ctx.font_extents();
                let text_extents = cairo_ctx.text_extents(text.clone().as_str());
                let x_req = LReq::new_fixed_size(text_extents.width);
                let y_req = LReq::new_fixed_ref(font_extents.ascent, font_extents.descent);
                let shreq = Rc::new(SharedReq::new(x_req, y_req));
                v.insert(shreq).clone()
            },
            Entry::Occupied(o) => o.get().clone()
        };

        return TextElement{text: text.clone(),
                           layout: ElementLayoutWithSharedReq::new(),
                           req: req};
    }
}

impl TElement for TextElement {
    fn draw_self(&self, cairo_ctx: &Context) {
        let y = match self.layout.y_alloc.ref_point() {
            None => 0.0,
            Some(ref_point) => ref_point
        };
        cairo_ctx.move_to(0.0, y);
        cairo_ctx.show_text(self.text.as_str());
    }

    fn draw(&self, cairo_ctx: &Context) {
        self.draw_self(cairo_ctx);
    }

    fn x_req(&self) -> &LReq {
        return &self.req.x_req;
    }

    fn x_alloc(&self) -> &LAlloc {
        return &self.layout.x_alloc;
    }

    fn x_alloc_mut(&mut self) -> &mut LAlloc {
        return &mut self.layout.x_alloc;
    }

    fn y_req(&self) -> &LReq {
        return &self.req.y_req;
    }

    fn y_alloc(&self) -> &LAlloc {
        return &self.layout.y_alloc;
    }

    fn y_alloc_mut(&mut self) -> &mut LAlloc {
        return &mut self.layout.y_alloc;
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

// struct FlowElement {
//     layout: ElementLayout,
//     children: Vec<TextElement>,
// }
//
// struct PageElement {
//     layout: ElementLayout,
//     children: Vec<FlowElement>,
// }
//
// trait TElementLayout {
//     fn update_x_req(&mut self);
//     fn update_y_req(&)
// }



struct RootElement
{
    layout: ElementLayout,
    children: Vec<Box<TElement>>,
}

impl RootElement {
    fn new(child: Box<TElement>) -> RootElement {
        return RootElement{layout: ElementLayout::new(), children: vec![child]};
    }
}

impl TElement for RootElement {
    fn draw(&self, cairo_ctx: &Context) {
        self.draw_self(cairo_ctx);
        self.draw_children(cairo_ctx);
    }

    fn x_req(&self) -> &LReq {
        return &self.layout.l_box.x_req;
    }

    fn x_alloc(&self) -> &LAlloc {
        return &self.layout.l_box.x_alloc;
    }

    fn x_alloc_mut(&mut self) -> &mut LAlloc {
        return &mut self.layout.l_box.x_alloc;
    }

    fn y_req(&self) -> &LReq {
        return &self.layout.l_box.y_req;
    }

    fn y_alloc(&self) -> &LAlloc {
        return &self.layout.l_box.y_alloc;
    }

    fn y_alloc_mut(&mut self) -> &mut LAlloc {
        return &mut self.layout.l_box.y_alloc;
    }


    fn update_x_req(&mut self) {
        self.update_children_x_req();
        self.layout.l_box.x_req = self.children[0].x_req().clone();
    }

    fn allocate_x(&mut self) {
        self.layout.l_box.x_alloc = LAlloc::new_from_req(&self.layout.l_box.x_req, 0.0);
        self.children[0].x_alloc_mut().clone_from(&self.layout.l_box.x_alloc);
        self.allocate_children_x();
    }

    fn update_y_req(&mut self) {
        self.update_children_y_req();
        self.layout.l_box.y_req = self.children[0].y_req().clone();
    }

    fn allocate_y(&mut self) {
        self.layout.l_box.y_alloc = LAlloc::new_from_req(&self.layout.l_box.y_req, 0.0);
        self.children[0].y_alloc_mut().clone_from(&self.layout.l_box.y_alloc);
        self.allocate_children_y();
    }
}


impl TBranchElement for RootElement {
    fn children<'a>(&'a self) -> &'a Vec<Box<TElement>> {
        return &self.children;
    }

    fn children_mut<'a>(&'a mut self) -> &'a mut Vec<Box<TElement>> {
        return &mut self.children;
    }

}




struct RenderingAPITestWindowState {
    width: i32,
    height: i32,

    req_table: HashMap<String, Rc<SharedReq>>,

    elem: Option<RootElement>,

    initialised: bool,
    layout_required: bool
}

impl RenderingAPITestWindowState {
    fn new(width: i32, height: i32) -> RenderingAPITestWindowState {
        return RenderingAPITestWindowState{width: width, height: height,
            req_table: HashMap::new(),
            elem: None,
            initialised: false,
            layout_required: true};
    }

    fn create_document_content(&mut self, cairo_ctx: &Context) -> Box<TElement> {
        let elem = TextElement::new(
            &String::from("The only curse they could afford to put on a tomb these days was 'Bugger Off'. --PTerry"),
            &cairo_ctx, &mut self.req_table);
        return Box::new(elem);
    }

    fn new_document_in_root(&mut self, cairo_ctx: &Context) -> RootElement {
        let content = self.create_document_content(cairo_ctx);
        return RootElement::new(content);
    }


    fn initialise(&mut self, cairo_ctx: &Context) {
        if !self.initialised {
            cairo_ctx.save();
            cairo_ctx.set_font_size(18.0);

            match &self.elem {
                &None => {
                    self.elem = Some(self.new_document_in_root(cairo_ctx));
                },
                &_ => {}
            };

            cairo_ctx.restore();

            self.initialised = true;
        }
    }

    fn on_draw(&mut self, cairo_ctx: Context) {
        self.initialise(&cairo_ctx);
        cairo_ctx.save();
        cairo_ctx.set_font_size(18.0);


        // cairo_ctx.translate(50.0, (self.height as f64) * 0.5);
        self.layout();
        self.draw(&cairo_ctx);
        cairo_ctx.restore();
    }

    fn on_size_allocate(&mut self, rect: &RectangleInt) {
        self.width = rect.width as i32;
        self.height = rect.height as i32;

        self.layout_required = true;
    }

    fn layout(&mut self) {
        if self.layout_required {
            match &mut self.elem {
                &mut Some(ref mut e) => {
                    e.update_x_req();
                    e.allocate_x();
                    e.update_y_req();
                    e.allocate_y();
                },
                &mut None => {}
            }
            self.layout_required = false;
        }
    }

    fn draw(&self, cairo_ctx: &Context) {
        match &self.elem {
            &Some(ref e) => {
                e.draw(cairo_ctx);
            },
            &None => {}
        }
    }
}


struct RenderingAPITestWindow {
    window: gtk::Window,
    drawing_area: gtk::DrawingArea,
    state: Rc<RefCell<RenderingAPITestWindowState>>
}

impl RenderingAPITestWindow {
    fn new(width: i32, height: i32) -> Rc<RefCell<RenderingAPITestWindow>> {
        let window = gtk::Window::new(gtk::WindowType::Toplevel).unwrap();
        let drawing_area = gtk::DrawingArea::new().unwrap();
        drawing_area.set_size_request(width, height);
        window.set_title("Cairo API test");
        window.add(&drawing_area);

        let wrapped_state = Rc::new(RefCell::new(RenderingAPITestWindowState::new(width, height)));

        let instance = RenderingAPITestWindow{window: window,
            drawing_area: drawing_area,
            state: wrapped_state.clone()
        };
        let wrapped_instance = Rc::new(RefCell::new(instance));

        let wrapped_state_for_draw = wrapped_state.clone();
        let wrapped_instance_for_draw = wrapped_instance.clone();
        wrapped_instance.borrow().drawing_area.connect_draw(move |widget, cairo_context| {
            wrapped_state_for_draw.borrow_mut().on_draw(cairo_context);

            wrapped_instance_for_draw.borrow().drawing_area.queue_draw();
            Inhibit(true)
        });

        let wrapped_state_for_sizealloc = wrapped_state.clone();
        wrapped_instance.borrow().drawing_area.connect_size_allocate(move |widget, rect| {
            wrapped_state_for_sizealloc.borrow_mut().on_size_allocate(rect);
        });

        wrapped_instance.borrow().window.show_all();

        return wrapped_instance;
    }

    fn exit_on_close(&self) {
        self.window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(true)
        });
    }
}


fn main() {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));
    println!("Major: {}, Minor: {}", gtk::get_major_version(), gtk::get_minor_version());

    let wrapped_window = RenderingAPITestWindow::new(800, 500);
    wrapped_window.borrow().exit_on_close();
    gtk::main();
}
