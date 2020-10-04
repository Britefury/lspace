#![cfg_attr(not(feature = "gtk_3_10"), allow(unused_variables, unused_mut))]

extern crate time;
extern crate gtk;
extern crate cairo;
extern crate lspace;
extern crate regex;

use std::io::prelude::*;
use std::io::{self, BufReader, BufWriter};
use std::fs::File;
use std::path::Path;
use std::string::String;
use std::rc::Rc;
use regex::Regex;

use gtk::traits::*;
use gtk::prelude::Inhibit;

use lspace::geom::colour::Colour;
use lspace::elements::text_element::{TextStyleParams};
use lspace::pres::pres::Pres;
use lspace::pres::primitive::{Column, Row, Text};
use lspace::pres::richtext::paragraph;
use lspace::lspace_widget::LSpaceWidget;

const FILENAME: &'static str = "examples/render_rust.rs";



struct TokenDefinition {
    re: Regex,
    style: Rc<TextStyleParams>,
    valid_strings: Option<Vec<String>>
}

impl TokenDefinition {
    pub fn new(re: &str, style: Rc<TextStyleParams>) -> TokenDefinition {
        return TokenDefinition{re: Regex::new(re).unwrap(), style: style, valid_strings: None};
    }

    pub fn keywords(words: Vec<&'static str>, style: Rc<TextStyleParams>) -> TokenDefinition {
        return TokenDefinition{re: Regex::new(r"\w+").unwrap(), style: style,
                               valid_strings: Some(words.iter().map(
                                   |x| String::from(*x)).collect())};
    }

    pub fn first(&self, text: &str) -> Option<(usize, usize)> {
        match self.re.find(text) {
            None => None,
            Some(mat) => {
                let start = mat.start();
                let end = mat.end();
                match self.valid_strings {
                    None => Some((start, end)),
                    Some(ref valids) if valids.contains(&String::from(&text[start..end])) =>
                        Some((start, end)),
                    _ => None
                }
            }
        }
    }
}

struct Tokeniser {
    tokens: Vec<TokenDefinition>,
    default_style: Rc<TextStyleParams>,
}

impl Tokeniser {
    pub fn new(tokens: Vec<TokenDefinition>, default_style: Rc<TextStyleParams>) -> Tokeniser {
        return Tokeniser{tokens: tokens, default_style: default_style};
    }

    pub fn tokenise(&self, text: &String) -> Vec<Pres> {
        if text.trim().len() == 0 {
            return vec![Text::new(String::from(" "), self.default_style.clone())];
        }
        let mut result: Vec<Pres> = Vec::new();

        let mut pos = 0;
        while pos < text.len() {
            let mut best: Option<(usize, usize, usize)> = None;
            for (tdef_index, tdef) in self.tokens.iter().enumerate() {
                let re_pos = tdef.first(&text[pos..]);
                best = match re_pos {
                    None => best,
                    Some((match_start, match_end)) => {
                        let token_start = pos + match_start;
                        let token_end = pos + match_end;
                        match best {
                            None => Some((token_start, token_end, tdef_index)),
                            Some((best_start, best_end, best_tdef_index))
                                if token_start < best_start => {
                                    Some((token_start, token_end, tdef_index))
                            },
                            _ => best
                        }
                    }
                }
            }

            match best {
                None => {
                    result.push(Text::new(String::from(&text[pos..]), self.default_style.clone()));
                    pos = text.len();
                },
                Some((tok_start, tok_end, tdef_index)) => {
                    if tok_start > pos {
                        result.push(Text::new(String::from(&text[pos..tok_start]),
                                              self.default_style.clone()));
                    }
                    result.push(Text::new(String::from(&text[tok_start..tok_end]),
                                          self.tokens[tdef_index].style.clone()));
                    pos = tok_end;
                }
            }
        }

        return result;
    }
}





fn main() {
    // Initialise GTK
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));
    println!("Major: {}, Minor: {}", gtk::get_major_version(), gtk::get_minor_version());


    // Build the tokeniser
    let tokeniser = Tokeniser::new(
        vec![
            TokenDefinition::new("[\\[\\]\\(\\)\\{\\}<>:\\.&'\"]",
                Rc::new(TextStyleParams::with_family_and_colour(String::from("Courier New"),
                                                                &Colour::new(0.0, 0.5, 1.0, 1.0)))),
            TokenDefinition::keywords(vec!["let", "mut", "for", "while", "struct", "enum", "trait",
                                           "in", "as", "match", "fn", "return", "use", "const",
                                           "extern", "impl", "pub", "self"],
                Rc::new(TextStyleParams::with_family_and_colour(String::from("Courier New"),
                                                                &Colour::new(0.7, 0.0, 0.0, 1.0)))),
            TokenDefinition::new(r"\w+",
                Rc::new(TextStyleParams::with_family_and_colour(String::from("Courier New"),
                                                                &Colour::new(0.0, 0.5, 0.0, 1.0)))),
        ],

        Rc::new(TextStyleParams::with_family(String::from("Courier New")))
    );


    println!("Some Rust code....");

    // Load some rust code
    let mut lines: Vec<String> = Vec::new();
    let mut f = File::open(FILENAME).unwrap();
    let mut reader = BufReader::new(f);
    for line in reader.lines() {
        match line {
            Ok(line_string) => {
                // Remove leading and trailing whitespace
                lines.push(line_string);
            },
            Err(..) => {panic!();}
        }
    }

    println!("Loaded Rust code ({} lines); creating presentation...", lines.len());

    // Create a presentation of the text using the `lspace.pres` API
    let mut pres_paragraphs: Vec<Pres> = Vec::new();
    for ref line_text in lines {
        let tokens = tokeniser.tokenise(line_text);
        let row = Row::new(tokens);
        pres_paragraphs.push(row);
        //pres_paragraphs.push(paragraph(line_text, &style));
    }
    let content = Column::new(pres_paragraphs);

    println!("Presentation built; displaying....");

    // Create the LSpace widget, showing our content
    let lspace = LSpaceWidget::new(content);
    let widget = lspace.gtk_widget();

    // Create a GTK window in which to place it
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Render Rust code");
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
