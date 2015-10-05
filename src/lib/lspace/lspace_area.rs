#![cfg_attr(not(feature = "gtk_3_10"), allow(unused_variables, unused_mut))]

extern crate time;

use std::rc::Rc;
use std::cell::RefCell;

use gdk::{
    EventAny,
    EventButton,
    EventConfigure,
    EventCrossing,
    EventExpose,
    EventFocus,
    EventGrabBroken,
    EventKey,
    EventMotion,
    EventProperty,
    EventProximity,
    EventScroll,
    EventWindowState,
    Screen,
};
use gdk::ffi::GdkModifierType;
use gtk;
use gtk::traits::*;
use gtk::signal::Inhibit;
use cairo::{Context, RectangleInt};

use geom::vector2::Vector2;
use geom::point2::Point2;
use geom::bbox2::BBox2;
use input::input_system::InputSystem;
use input::pointer::PointerPosition;
use elements::element_ctx::ElementContext;
use elements::element::{TElement};
use elements::root_element::RootElement;
use pres::pres::{Pres, PresBuildCtx};
use pres::primitive::root_containing;


struct LSpaceAreaState {
    width: i32,
    height: i32,

    input_system: InputSystem,

    elem_ctx: RefCell<ElementContext>,

    content: Pres,

    elem: Option<RootElement>,

    initialised: bool,
    layout_required: bool
}

impl LSpaceAreaState {
    fn new(content: Pres) -> LSpaceAreaState {
        return LSpaceAreaState{width: 100, height: 100,
            input_system: InputSystem::new(),
            content: content,
            elem_ctx: RefCell::new(ElementContext::new()),
            elem: None,
            initialised: false,
            layout_required: true};
    }

    fn new_document_in_root(&mut self, cairo_ctx: &Context) -> RootElement {
        let pres_ctx = PresBuildCtx::new(&self.elem_ctx, cairo_ctx);
        let root_elem = root_containing(&self.content, &pres_ctx);
        return root_elem;
    }

    fn initialise(&mut self, cairo_ctx: &Context) {
        if !self.initialised {
            cairo_ctx.save();
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

    fn on_realize(&mut self) {
    }

    fn on_unrealize(&mut self) {
    }

    fn on_size_allocate(&mut self, rect: &RectangleInt) {
        self.width = rect.width as i32;
        self.height = rect.height as i32;

        self.layout_required = true;
    }

    fn on_button_press(&mut self, event_button: &EventButton) {
        self.input_system.mod_state_mut().update_from_gdk_mod(event_button.state);
        let pos = Point2::new(event_button.x, event_button.y);
        self.input_system.mouse_mut().set_position(PointerPosition::at_position(pos));
    }

    fn on_button_release(&mut self, event_button: &EventButton) {
        self.input_system.mod_state_mut().update_from_gdk_mod(event_button.state);
        let pos = Point2::new(event_button.x, event_button.y);
        self.input_system.mouse_mut().set_position(PointerPosition::at_position(pos));
    }

    fn on_enter(&mut self, event_crossing: &EventCrossing) {
        self.input_system.mod_state_mut().update_from_gdk_mod(event_crossing.state);
        let pos = Point2::new(event_crossing.x, event_crossing.y);
        self.input_system.mouse_mut().set_position(PointerPosition::at_position(pos));
    }

    fn on_leave(&mut self, event_crossing: &EventCrossing) {
        self.input_system.mod_state_mut().update_from_gdk_mod(event_crossing.state);
        self.input_system.mouse_mut().set_position(PointerPosition::out_of_bounds());
    }

    fn on_motion(&mut self, event_motion: &EventMotion) {
        self.input_system.mod_state_mut().update_from_gdk_mod(event_motion.state);
        let pos = Point2::new(event_motion.x, event_motion.y);
        self.input_system.mouse_mut().set_position(PointerPosition::at_position(pos));
    }

    fn on_scroll(&mut self, event_scroll: &EventScroll) {
        self.input_system.mod_state_mut().update_from_gdk_mod(event_scroll.state);
        let pos = Point2::new(event_scroll.x, event_scroll.y);
        self.input_system.mouse_mut().set_position(PointerPosition::at_position(pos));
    }

    fn on_key_press(&mut self, event_key: &EventKey) {
        self.input_system.mod_state_mut().update_from_gdk_mod(event_key.state);
    }

    fn on_key_release(&mut self, event_key: &EventKey) {
        self.input_system.mod_state_mut().update_from_gdk_mod(event_key.state);
    }

    fn on_draw(&mut self, cairo_ctx: Context) {
        self.initialise(&cairo_ctx);
        self.layout();
        self.draw(&cairo_ctx);
    }

    fn layout(&mut self) {
        if self.layout_required {
            match &mut self.elem {
                &mut Some(ref mut e) => {
                    let t1 = time::precise_time_ns();
                    let rx = e.root_requisition_x();
                    e.root_allocate_x(self.width as f64);
                    let ry = e.root_requisition_y();
                    e.root_allocate_y(ry);
                    let t2 = time::precise_time_ns();
                    println!("Layout time: {}", (t2-t1) as f64 * 1.0e-9);
                },
                &mut None => {}
            }
            self.layout_required = false;
        }
    }

    fn draw(&self, cairo_ctx: &Context) {
        match &self.elem {
            &Some(ref e) => {
                let t1 = time::precise_time_ns();
                e.draw(cairo_ctx, &BBox2::from_lower_size(Point2::origin(),
                        Vector2::new(self.width as f64, self.height as f64)));
                let t2 = time::precise_time_ns();
                println!("Render time: {}", (t2-t1) as f64 * 1.0e-9);
            },
            &None => {}
        }
    }
}


pub struct LSpaceArea {
    drawing_area: gtk::DrawingArea,
    state: Rc<RefCell<LSpaceAreaState>>
}

impl LSpaceArea {
    pub fn new(content: Pres) -> Rc<RefCell<LSpaceArea>> {
        let drawing_area = gtk::DrawingArea::new().unwrap();
        let wrapped_state = Rc::new(RefCell::new(LSpaceAreaState::new(content)));

        let instance = LSpaceArea{drawing_area: drawing_area,
            state: wrapped_state.clone()
        };
        let wrapped_instance = Rc::new(RefCell::new(instance));

        {
            let state_clone = wrapped_state.clone();
            wrapped_instance.borrow().drawing_area.connect_realize(move |widget| {
                state_clone.borrow_mut().on_realize();
            });
        }

        {
            let state_clone = wrapped_state.clone();
            wrapped_instance.borrow().drawing_area.connect_unrealize(move |widget| {
                state_clone.borrow_mut().on_unrealize();
            });
        }

        {
            let state_clone = wrapped_state.clone();
            wrapped_instance.borrow().drawing_area.connect_size_allocate(move |widget, rect| {
                state_clone.borrow_mut().on_size_allocate(rect);
            });
        }

        {
            let state_clone = wrapped_state.clone();
            wrapped_instance.borrow().drawing_area.connect_button_press_event(move |widget, event_button| {
                state_clone.borrow_mut().on_button_press(event_button);
                return Inhibit(true);
            });
        }

        {
            let state_clone = wrapped_state.clone();
            wrapped_instance.borrow().drawing_area.connect_button_release_event(move |widget, event_button| {
                state_clone.borrow_mut().on_button_release(event_button);
                return Inhibit(true);
            });
        }

        {
            let state_clone = wrapped_state.clone();
            wrapped_instance.borrow().drawing_area.connect_enter_notify_event(move |widget, event_crossing| {
                state_clone.borrow_mut().on_enter(event_crossing);
                return Inhibit(true);
            });
        }

        {
            let state_clone = wrapped_state.clone();
            wrapped_instance.borrow().drawing_area.connect_leave_notify_event(move |widget, event_crossing| {
                state_clone.borrow_mut().on_leave(event_crossing);
                return Inhibit(true);
            });
        }

        {
            let state_clone = wrapped_state.clone();
            wrapped_instance.borrow().drawing_area.connect_motion_notify_event(move |widget, event_motion| {
                state_clone.borrow_mut().on_motion(event_motion);
                return Inhibit(true);
            });
        }

        {
            let state_clone = wrapped_state.clone();
            wrapped_instance.borrow().drawing_area.connect_scroll_event(move |widget, event_scroll| {
                state_clone.borrow_mut().on_scroll(event_scroll);
                return Inhibit(true);
            });
        }

        {
            let state_clone = wrapped_state.clone();
            wrapped_instance.borrow().drawing_area.connect_key_press_event(move |widget, event_key| {
                state_clone.borrow_mut().on_key_press(event_key);
                return Inhibit(true);
            });
        }

        {
            let state_clone = wrapped_state.clone();
            wrapped_instance.borrow().drawing_area.connect_key_release_event(move |widget, event_key| {
                state_clone.borrow_mut().on_key_release(event_key);
                return Inhibit(true);
            });
        }

        {
            let state_clone = wrapped_state.clone();
            let inst_clone = wrapped_instance.clone();
            wrapped_instance.borrow().drawing_area.connect_draw(move |widget, cairo_context| {
                state_clone.borrow_mut().on_draw(cairo_context);

                inst_clone.borrow().drawing_area.queue_draw();
                return Inhibit(true);
            });
        }

        return wrapped_instance;
    }

    pub fn gtk_widget(&self) -> &gtk::DrawingArea {
        return &self.drawing_area;
    }
}
