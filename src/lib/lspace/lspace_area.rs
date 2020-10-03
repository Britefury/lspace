#![cfg_attr(not(feature = "gtk_3_10"), allow(unused_variables, unused_mut))]

extern crate time;

use std::rc::Rc;
use std::cell::{Ref, RefCell};

use cairo::Context;

use gdk::keys::Key;

use geom::vector2::Vector2;
use geom::point2::Point2;
use geom::bbox2::BBox2;
use input::inputmodifier::InputModifierState;
use input::keyboard::Keyboard;
use input::pointer::{Pointer, PointerPosition};
use elements::element_ctx::{ElementContext, ElementLayoutContext};
use elements::element::{ElementRef, elem_as_ref};
use elements::{root_element};
use pres::pres::{Pres, TPres, PresBuildCtx};


pub trait TLSpaceListener {
    fn notify_queue_redraw(&self, rect: &BBox2);
}

pub struct LSpaceAreaMut {
    width: i32,
    height: i32,

    input_mods: InputModifierState,
    input_pointer: Pointer,

    elem_ctx: ElementContext,

    root_element: ElementRef,

    layout_required: bool,
}

impl LSpaceAreaMut {
    pub fn new() -> LSpaceAreaMut {
        let root_elem = elem_as_ref(root_element::RootElement::new());

        return LSpaceAreaMut{width: 100, height: 100,
            input_mods: InputModifierState::new(),
            input_pointer: Pointer::new(),
            elem_ctx: ElementContext::new(),
            root_element: root_elem,
            layout_required: true};
    }

    pub fn set_content_element(&mut self, content: ElementRef) {
        self.root_element.as_bin().unwrap().set_child(&self.root_element, content);
    }

    pub fn set_content_pres(&mut self, p: Pres) {
        let pres_ctx = PresBuildCtx::new(&self.elem_ctx);
        let child = p.build(&pres_ctx);
        self.root_element.as_bin().unwrap().set_child(&self.root_element, child);
    }

    pub fn on_realize(&mut self) {
    }

    pub fn on_unrealize(&mut self) {
    }

    pub fn on_size_allocate(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;

        self.layout_required = true;
    }

    pub fn on_button_press(&mut self, mod_state: InputModifierState, pos: Point2, button: u32) {
        self.input_mods = mod_state;
        self.input_pointer.set_position(PointerPosition::at_position(pos));
    }

    pub fn on_button_release(&mut self, mod_state: InputModifierState, pos: Point2, button: u32) {
        self.input_mods = mod_state;
        self.input_pointer.set_position(PointerPosition::at_position(pos));
    }

    pub fn on_enter(&mut self, mod_state: InputModifierState, pos: Point2) {
        self.input_mods = mod_state;
        self.input_pointer.set_position(PointerPosition::at_position(pos));
    }

    pub fn on_leave(&mut self, mod_state: InputModifierState, pos: Point2) {
        self.input_mods = mod_state;
        self.input_pointer.set_position(PointerPosition::out_of_bounds());
    }

    pub fn on_motion(&mut self, mod_state: InputModifierState, pos: Point2) {
        self.input_mods = mod_state;
        self.input_pointer.set_position(PointerPosition::at_position(pos));
    }

    pub fn on_scroll(&mut self, mod_state: InputModifierState, pos: Point2,
                 scroll_x: f64, scroll_y: f64) {
        self.input_mods = mod_state;
        self.input_pointer.set_position(PointerPosition::at_position(pos));
    }

    pub fn on_key_press(&mut self, mod_state: InputModifierState, key_val: Key, key_string: Option<String>) {
        self.input_mods = mod_state;
    }

    pub fn on_key_release(&mut self, mod_state: InputModifierState, key_val: Key, key_string: Option<String>) {
        self.input_mods = mod_state;
    }

    pub fn on_draw(&mut self, cairo_ctx: &Context) {
        self.layout(cairo_ctx);
        self.draw(cairo_ctx);
    }

    fn layout(&mut self, cairo_ctx: &Context) {
        if self.layout_required || true {
            let e = self.root_element.as_root_element().unwrap();
            let layout_ctx = ElementLayoutContext::new(&self.elem_ctx, cairo_ctx);
            let rx = e.root_requisition_x(&layout_ctx);
            e.root_allocate_x(self.width as f64);
            let ry = e.root_requisition_y();
            e.root_allocate_y(ry);
            self.layout_required = false;
        }
    }

    fn draw(&self, cairo_ctx: &Context) {
        let e = self.root_element.as_root_element().unwrap();
        let t1 = time::precise_time_ns();
        e.draw(cairo_ctx, &BBox2::from_lower_size(Point2::origin(),
                Vector2::new(self.width as f64, self.height as f64)));
        let t2 = time::precise_time_ns();
    }
}


pub struct LSpaceArea {
    m: RefCell<LSpaceAreaMut>,
    input_keyboard: Keyboard,
}

impl LSpaceArea {
    pub fn new() -> LSpaceArea {
        LSpaceArea{m: RefCell::new(LSpaceAreaMut::new()),
            input_keyboard: Keyboard::new(),
        }
    }

    pub fn element_context(&self) -> Ref<ElementContext> {
        let mm = self.m.borrow();
        Ref::map(mm, |x| &x.elem_ctx)
    }

    pub fn keyboard(&self) -> &Keyboard {
        &self.input_keyboard
    }

    pub fn set_content_element(&self, content: ElementRef) {
        let mut mm = self.m.borrow_mut();
        mm.set_content_element(content)
    }

    pub fn set_content_pres(&self, p: Pres) {
        let mut mm = self.m.borrow_mut();
        mm.set_content_pres(p)
    }


    pub fn set_lspace_listener(&self, listener: Option<&Rc<TLSpaceListener>>) {
        self.m.borrow().root_element.as_root_element().unwrap().root_set_lspace_listener(listener);
    }


    pub fn on_realize(&self) {
        self.m.borrow_mut().on_realize();
    }

    pub fn on_unrealize(&self) {
        self.m.borrow_mut().on_unrealize();
    }

    pub fn on_size_allocate(&self, width: i32, height: i32) {
        self.m.borrow_mut().on_size_allocate(width, height);
    }

    pub fn on_button_press(&self, mod_state: InputModifierState, pos: Point2, button: u32) {
        self.m.borrow_mut().on_button_press(mod_state, pos, button);
    }

    pub fn on_button_release(&self, mod_state: InputModifierState, pos: Point2, button: u32) {
        self.m.borrow_mut().on_button_release(mod_state, pos, button);
    }

    pub fn on_enter(&self, mod_state: InputModifierState, pos: Point2) {
        self.m.borrow_mut().on_enter(mod_state, pos);
    }

    pub fn on_leave(&self, mod_state: InputModifierState, pos: Point2) {
        self.m.borrow_mut().on_leave(mod_state, pos);
    }

    pub fn on_motion(&self, mod_state: InputModifierState, pos: Point2) {
        self.m.borrow_mut().on_motion(mod_state, pos);
    }

    pub fn on_scroll(&self, mod_state: InputModifierState, pos: Point2,
                     scroll_x: f64, scroll_y: f64) {
        self.m.borrow_mut().on_scroll(mod_state, pos, scroll_x, scroll_y);
    }

    pub fn on_key_press(&self, mod_state: InputModifierState, key_val: Key, key_string: Option<String>) {
        self.m.borrow_mut().on_key_press(mod_state, key_val.clone(), key_string.clone());
        self.input_keyboard.on_key_press(mod_state, key_val, key_string);
    }

    pub fn on_key_release(&self, mod_state: InputModifierState, key_val: Key, key_string: Option<String>) {
        self.m.borrow_mut().on_key_release(mod_state, key_val.clone(), key_string.clone());
        self.input_keyboard.on_key_release(mod_state, key_val, key_string);
    }

    pub fn on_draw(&self, cairo_ctx: &Context) {
        self.m.borrow_mut().on_draw(cairo_ctx);
    }
}

