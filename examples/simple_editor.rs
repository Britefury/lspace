#![cfg_attr(not(feature = "gtk_3_10"), allow(unused_variables, unused_mut))]

extern crate time;
extern crate gtk;
extern crate gdk;
extern crate cairo;
extern crate lspace;

use std::string::String;
use std::rc::Rc;

use gtk::traits::*;
use gtk::prelude::Inhibit;

use gdk::keys::constants as keys;
use gdk::keys::Key;

use lspace::geom::colour::Colour;
use lspace::elements::element::{ElementRef, elem_as_ref};
use lspace::elements::container_sequence::TContainerSequenceElement;
use lspace::elements::{text_element, row, column};
use lspace::input::keyboard::{KeyEventType, KeyEvent, TKeyboardInteractor};
use lspace::lspace_widget::LSpaceWidget;
use lspace::lspace_area::LSpaceArea;


struct LineEditor {
    text_elem: ElementRef
}

impl LineEditor {
    pub fn new(elem: &ElementRef) -> Rc<LineEditor> {
        Rc::new(LineEditor{text_elem: elem.clone()})
    }
}

impl TKeyboardInteractor for LineEditor {
    fn on_key_event(&self, event: &KeyEvent) {
        if event.event_type() == KeyEventType::Press {
            let mut text = self.text_elem.as_text_element().unwrap().get_text().clone();
            println!("Key val = {}", event.key_val());
            match event.key_val() as Key {
                keys::BackSpace => {
                    println!("Deleting");
                    let n = text.len();
                    let new_text = String::from(&text[0..n-1]);
                    self.text_elem.as_text_element().unwrap().set_text(new_text);
                },
                keys::Shift_L | keys::Shift_R | keys::Control_L | keys::Control_R |
                keys::Meta_L | keys::Meta_R | keys::Alt_L | keys::Alt_R |
                keys::Super_L | keys::Super_R | keys::Hyper_L | keys::Hyper_R |
                keys::Caps_Lock | keys::Shift_Lock => {
                    // Ignore
                },
                _ => {
                    match event.key_string() {
                        None => {
                        },
                        Some(key_string) => {
                            println!("Inserting {:?}", key_string);
                            let new_text = text + key_string;
                            self.text_elem.as_text_element().unwrap().set_text(new_text);
                        }
                    }
                }
            }
        }
    }
}


fn main() {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));

    // Styles for text element and end-of-line element
    let text_style = Rc::new(text_element::TextStyleParams::default());
    let eol_style = Rc::new(text_element::TextStyleParams::new(String::from("Helvetica"),
                                                               text_element::TextWeight::Normal,
                                                               text_element::TextSlant::Normal,
                                                               11.0,
                                                               &Colour::new(0.0, 0.5, 1.0, 1.0)));

    let lspace = Rc::new(LSpaceArea::new());

    // The text that will be edited
    let text = elem_as_ref(text_element::TextElement::new_in_area(String::from("Hello world"),
                                                                  text_style, &lspace));

    // End of line marker (in blue)
    let end_of_line = elem_as_ref(text_element::TextElement::new_in_area(String::from("}{"),
                                                                         eol_style, &lspace));

    // Editor that will mutate the text element
    let editor_as_interactor: Rc<TKeyboardInteractor> = LineEditor::new(&text);

    // Place the editable text and EOL marker in a row
    let line = elem_as_ref(row::RowElement::new(0.0));
    line.as_container_sequence().unwrap().set_children(&line, &vec![text, end_of_line]);

    let content = elem_as_ref(column::ColumnElement::new(0.0));
    content.as_container_sequence().unwrap().set_children(&content, &vec![line]);

    lspace.set_content_element(content);

    // Add the editor as a listener
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
