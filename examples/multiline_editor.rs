#![cfg_attr(not(feature = "gtk_3_10"), allow(unused_variables, unused_mut))]

extern crate time;
extern crate gtk;
extern crate gdk;
extern crate cairo;
extern crate lspace;
extern crate regex;

use std::string::String;
use std::rc::Rc;
use std::cell::{RefCell};
use std::cmp::min;

use gtk::traits::*;
use gtk::prelude::Inhibit;

// for a list of key names see: http://gtk-rs.org/docs/gdk/enums/key/index.html
use gdk::keys::constants as keys;
use gdk::keys::Key;

use lspace::geom::colour::Colour;
use lspace::elements::element::{ElementRef, elem_as_ref};
use lspace::elements::container_sequence::TContainerSequenceElement;
use lspace::elements::{text_element, row, column};
use lspace::input::keyboard::{KeyEventType, KeyEvent, TKeyboardInteractor};
use lspace::lspace_widget::LSpaceWidget;
use lspace::lspace_area::LSpaceArea;


struct MultilineEditorMut {
    text_elements: Vec<ElementRef>,
    current_line: usize,
    column: usize,
}

struct MultiLineEditor {
    m: RefCell<MultilineEditorMut>,
    area: Rc<LSpaceArea>,
    text_style: Rc<text_element::TextStyleParams>,
    content: ElementRef
}


impl MultiLineEditor {
    pub fn new(area: &Rc<LSpaceArea>) -> Rc<MultiLineEditor> {
        let text_style = Rc::new(text_element::TextStyleParams::default());

        let m = MultilineEditorMut{
            text_elements: Vec::new(),
            current_line: 0,
            column: 0
        };

        let _self = Rc::new(MultiLineEditor{
            area: area.clone(),
            m: RefCell::new(m),
            text_style: text_style,
            content: elem_as_ref(column::ColumnElement::new(0.0))
        });
        _self.new_line();
        _self.area.set_content_element(_self.content.clone());
        _self
    }

    fn get_content(&self) -> &TContainerSequenceElement {
        self.content.as_container_sequence().unwrap()
    }

    fn new_line(&self) -> ElementRef {
        let line = elem_as_ref(text_element::TextElement::new_in_area(
            String::from(""), self.text_style.clone(), &self.area));
        let mut mm = self.m.borrow_mut();
        let mut index = mm.current_line;
        if index < mm.text_elements.len() {
            index = index + 1;
        }
        if mm.text_elements.len() > 0 {
            mm.current_line += 1;
        }
        mm.column = 0;
        mm.text_elements.insert(index, line.clone());
        self.get_content().set_children(&self.content, &mm.text_elements);

        line
    }

    fn cursor_left(&self) {
        let mut mm = self.m.borrow_mut();
        if mm.column > 0 {
            mm.column -= 1;
        }
    }

    fn cursor_right(&self) {
        let mut mm = self.m.borrow_mut();
        let mut text = mm.text_elements[mm.current_line].as_text_element().unwrap().get_text().clone();

        if mm.column < text.len() {
            mm.column += 1;
        }
    }

    fn cursor_up(&self) {
        let mut mm = self.m.borrow_mut();
        if mm.current_line == 0 {
            mm.column = 0;
        } else {
            mm.current_line -= 1;
            let text = mm.text_elements[mm.current_line].as_text_element().unwrap().get_text().clone();
            mm.column = min(mm.column, text.len());
        }
    }

    fn cursor_down(&self) {
        let mut mm = self.m.borrow_mut();
        // last line
        if mm.current_line + 1 >= mm.text_elements.len() {
            let text = mm.text_elements[mm.current_line].as_text_element().unwrap().get_text().clone();
            mm.column = text.len();
        } else {
            mm.current_line += 1;
            let text = mm.text_elements[mm.current_line].as_text_element().unwrap().get_text().clone();
            mm.column = min(mm.column, text.len());
        }
    }
}

impl TKeyboardInteractor for MultiLineEditor {
    fn on_key_event(&self, event: &KeyEvent) {
        if event.event_type() == KeyEventType::Press {
            match event.key_val() as Key {
                keys::BackSpace => {
                    let mut mm = self.m.borrow_mut();
                    let pos = mm.column;
                    let mut text = mm.text_elements[mm.current_line].as_text_element().unwrap().get_text().clone();

                    let n = text.len();
                    if n > 0 {
                        println!("{:?}", vec![n, pos]);
                        let new_text = String::from(text[0..pos-1].to_string() + &text[pos..n]);
                        mm.text_elements[mm.current_line].as_text_element().unwrap().set_text(new_text);
                        mm.column -= 1;
                    }
                },

                keys::Left => {
                    self.cursor_left();
                },
                keys::Right => {
                    self.cursor_right();
                },
                keys::Up => {
                    self.cursor_up();
                },
                keys::Down => {
                    self.cursor_down();
                },

                keys::Return => {
                    self.new_line();
                },

                // we can ignore shift
                keys::Shift_L => {},
                keys::Shift_R => {},

                _ => {
                    let mut mm = self.m.borrow_mut();
                    let mut text = mm.text_elements[mm.current_line].as_text_element().unwrap().get_text().clone();
                    let n = text.len();

                    println!("Inserting {:?}", event.key_string().as_ref().unwrap());
                    let new_text = text[0..mm.column].to_string() + event.key_string().as_ref().unwrap().as_str() + &text[mm.column..n];
                    mm.column += 1;
                    mm.text_elements[mm.current_line].as_text_element().unwrap().set_text(new_text);
                }
            }
        }
    }
}


fn main() {
    // Initialise GTK
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));

    let eol_style = Rc::new(text_element::TextStyleParams::new(String::from("Helvetica"),
                                                               text_element::TextWeight::Normal, text_element::TextSlant::Normal, 11.0, &Colour::new(0.0, 0.5, 1.0, 1.0)));


    let lspace = Rc::new(LSpaceArea::new());

    let editor_as_interactor: Rc<TKeyboardInteractor> = MultiLineEditor::new(&lspace);

    lspace.keyboard().add_interactor(&editor_as_interactor);


    // Create the LSpace widget, showing our content
    let lsw = LSpaceWidget::new_with_area(lspace);
    let widget = lsw.gtk_widget();
    widget.grab_focus();

    // Create a GTK window in which to place it
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Very Simple Editor");
    window.add(&*widget);
    window.set_default_size(800, 500);

    // Quit the GTK main loop when the window is closed
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(true)
    });

    // Show everything
    window.show_all();

    // Enter GTK main loop
    gtk::main();
}
