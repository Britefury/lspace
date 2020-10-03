#![cfg_attr(not(feature = "gtk_3_10"), allow(unused_variables, unused_mut))]

extern crate time;

use std::rc::Rc;
use std::cell::RefCell;
use std::ffi::CStr;
use libc::c_char;

// use gdk::ffi as gdk_ffi;
use gdk::ModifierType;
use gtk;
use gtk::traits::*;
use gtk::prelude::Inhibit;

use input::inputmodifier::{self, InputModifierState};

use geom::point2::Point2;
use geom::bbox2::BBox2;
use pres::pres::Pres;
use lspace_area::{LSpaceArea, TLSpaceListener};


fn gdk_modifier_to_input_mod_state(gdk_state: ModifierType) -> InputModifierState {
    let mut value = 0;
    value = value | if gdk_state.contains(ModifierType::BUTTON1_MASK) {inputmodifier::BUTTON1}
                    else {0};
    value = value | if gdk_state.contains(ModifierType::BUTTON2_MASK) {inputmodifier::BUTTON2}
                    else {0};
    value = value | if gdk_state.contains(ModifierType::BUTTON3_MASK) {inputmodifier::BUTTON3}
                    else {0};
    value = value | if gdk_state.contains(ModifierType::BUTTON4_MASK) {inputmodifier::BUTTON4}
                    else {0};
    value = value | if gdk_state.contains(ModifierType::BUTTON5_MASK) {inputmodifier::BUTTON5}
                    else {0};
    value = value | if gdk_state.contains(ModifierType::CONTROL_MASK) {inputmodifier::KEY_CTRL}
                    else {0};
    value = value | if gdk_state.contains(ModifierType::SHIFT_MASK) {inputmodifier::KEY_SHIFT}
                    else {0};
    value = value | if gdk_state.contains(ModifierType::MOD1_MASK) {inputmodifier::KEY_ALT}
                    else {0};
    value = value | if gdk_state.contains(ModifierType::SUPER_MASK) {inputmodifier::KEY_SUPER}
                    else {0};
    InputModifierState::from_values(value)
}


struct LSpaceWidgetMut {
    drawing_area: Rc<gtk::DrawingArea>,
    area: Rc<LSpaceArea>
}

impl LSpaceWidgetMut {
    pub fn new_with_area(area: Rc<LSpaceArea>) -> LSpaceWidgetMut {
        let drawing_area = Rc::new(gtk::DrawingArea::new());
        drawing_area.set_can_focus(true);

        let instance = LSpaceWidgetMut{drawing_area: drawing_area.clone(),
            area: area.clone()
        };

        {
            let state_clone = area.clone();
            drawing_area.connect_realize(move |widget| {
                state_clone.on_realize();
            });
        }

        {
            let state_clone = area.clone();
            drawing_area.connect_unrealize(move |widget| {
                state_clone.on_unrealize();
            });
        }

        {
            let state_clone = area.clone();
            drawing_area.connect_size_allocate(move |widget, rect| {
                state_clone.on_size_allocate(rect.width as i32, rect.height as i32);
            });
        }

        {
            let state_clone = area.clone();
            drawing_area.connect_button_press_event(move |widget, event_button| {
                let mod_state = gdk_modifier_to_input_mod_state(event_button.get_state());
                let (pos_x, pos_y) = event_button.get_position();
                let pos = Point2::new(pos_x, pos_y);
                state_clone.on_button_press(mod_state, pos, event_button.get_button());
                Inhibit(true)
            });
        }

        {
            let state_clone = area.clone();
            drawing_area.connect_button_release_event(move |widget, event_button| {
                let mod_state = gdk_modifier_to_input_mod_state(event_button.get_state());
                let (pos_x, pos_y) = event_button.get_position();
                let pos = Point2::new(pos_x, pos_y);
                state_clone.on_button_release(mod_state, pos, event_button.get_button());
                Inhibit(true)
            });
        }

        {
            let state_clone = area.clone();
            drawing_area.connect_enter_notify_event(move |widget, event_crossing| {
                let mod_state = gdk_modifier_to_input_mod_state(event_crossing.get_state());
                let (pos_x, pos_y) = event_crossing.get_position();
                let pos = Point2::new(pos_x, pos_y);
                state_clone.on_enter(mod_state, pos);
                Inhibit(true)
            });
        }

        {
            let state_clone = area.clone();
            drawing_area.connect_leave_notify_event(move |widget, event_crossing| {
                let mod_state = gdk_modifier_to_input_mod_state(event_crossing.get_state());
                let (pos_x, pos_y) = event_crossing.get_position();
                let pos = Point2::new(pos_x, pos_y);
                state_clone.on_leave(mod_state, pos);
                Inhibit(true)
            });
        }

        {
            let state_clone = area.clone();
            drawing_area.connect_motion_notify_event(move |widget, event_motion| {
                let mod_state = gdk_modifier_to_input_mod_state(event_motion.get_state());
                let (pos_x, pos_y) = event_motion.get_position();
                let pos = Point2::new(pos_x, pos_y);
                state_clone.on_motion(mod_state, pos);
                Inhibit(true)
            });
        }

        {
            let state_clone = area.clone();
            drawing_area.connect_scroll_event(move |widget, event_scroll| {
                let mod_state = gdk_modifier_to_input_mod_state(event_scroll.get_state());
                let (pos_x, pos_y) = event_scroll.get_position();
                let pos = Point2::new(pos_x, pos_y);
                let (dx, dy) = event_scroll.get_delta();
                state_clone.on_scroll(mod_state, pos, dx, dy);
                Inhibit(true)
            });
        }

        {
            let state_clone = area.clone();
            drawing_area.connect_key_press_event(move |widget, event_key| {
                let mod_state = gdk_modifier_to_input_mod_state(event_key.get_state());
                let k_str = event_key.get_keyval().to_unicode().map(|s| s.to_string());
                state_clone.on_key_press(mod_state, event_key.get_keyval(), k_str);
                Inhibit(true)
            });
        }

        {
            let state_clone = area.clone();
            drawing_area.connect_key_release_event(move |widget, event_key| {
                let mod_state = gdk_modifier_to_input_mod_state(event_key.get_state());
                let k_str = event_key.get_keyval().to_unicode().map(|s| s.to_string());
                state_clone.on_key_release(mod_state, event_key.get_keyval(), k_str);
                Inhibit(true)
            });
        }

        {
            let state_clone = area.clone();
            drawing_area.connect_draw(move |widget, cairo_context| {
                state_clone.on_draw(&cairo_context);
                Inhibit(true)
            });
        }

        LSpaceWidgetMut{drawing_area: drawing_area.clone(), area: area.clone()}
    }

    pub fn new(content: Pres) -> LSpaceWidgetMut {
        let area = Rc::new(LSpaceArea::new());
        area.set_content_pres(content);
        return LSpaceWidgetMut::new_with_area(area);
    }

    pub fn gtk_widget(&self) -> Rc<gtk::DrawingArea> {
        return self.drawing_area.clone();
    }
}


pub struct LSpaceWidget {
    m: RefCell<LSpaceWidgetMut>
}

impl LSpaceWidget {
    pub fn new_with_area(area: Rc<LSpaceArea>) -> Rc<LSpaceWidget> {
        let inst = Rc::new(LSpaceWidget{m: RefCell::new(LSpaceWidgetMut::new_with_area(area))});
        let listener: Rc<TLSpaceListener> = inst.clone();
        inst.m.borrow().area.set_lspace_listener(Some(&listener));
        inst
    }

    pub fn new(content: Pres) -> Rc<LSpaceWidget> {
        let inst = Rc::new(LSpaceWidget{m: RefCell::new(LSpaceWidgetMut::new(content))});
        let listener: Rc<TLSpaceListener> = inst.clone();
        inst.m.borrow().area.set_lspace_listener(Some(&listener));
        inst
    }

    pub fn gtk_widget(&self) -> Rc<gtk::DrawingArea> {
        return self.m.borrow().gtk_widget();
    }
}

impl TLSpaceListener for LSpaceWidget {
    fn notify_queue_redraw(&self, rect: &BBox2) {
        // When the Cairo bindings finally implement the necessary types to allow the
        // `queue_draw_region()` method to be called, use it instead, as it allows you to only
        // redraw the part of the widget that has changed, rather than the whole thing.
        self.m.borrow().drawing_area.queue_draw();
    }
}

