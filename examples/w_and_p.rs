#![cfg_attr(not(feature = "gtk_3_10"), allow(unused_variables, unused_mut))]
// #![feature(path_ext)]
// #![feature(convert)]

extern crate time;
extern crate gtk;
extern crate cairo;
extern crate lspace;

use std::io::prelude::*;
use std::io::{self, BufReader, BufWriter};
use std::fs::File;
use std::path::Path;
use std::string::String;
use std::rc::Rc;
use std::process::Command;

use gtk::traits::*;
use gtk::prelude::Inhibit;

use lspace::elements::text_element::{TextStyleParams};
use lspace::pres::pres::Pres;
use lspace::pres::primitive::Column;
use lspace::pres::richtext::paragraph;
use lspace::lspace_widget::LSpaceWidget;

const DOWNLOAD_URL: &'static str = "http://www.gutenberg.org/files/2600/2600.txt";
const FORMATTED_FILENAME: &'static str = "war_and_peace_formatted.txt";

fn main() {
    // Initialise GTK
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));
    println!("Major: {}, Minor: {}", gtk::get_major_version(), gtk::get_minor_version());

    // Download War and Peace if necessary
    if !Path::new(FORMATTED_FILENAME).exists() {
        println!("Using Python to download War and Peace text...");

        let cmd_name = match Command::new("python").args(&["--version"]).output() {
            Ok(_) => Option::Some("python"),
            Err(_) => match Command::new("python3").args(&["--version"]).output() {
                Ok(_) => Option::Some("python3"),
                Err(_) => Option::None
            }
        };
        let version_out = Command::new(cmd_name.unwrap()).args(&["--version"]).output().unwrap();

        let output = String::from_utf8_lossy(&version_out.stdout);
        println!("Output: {}", output);

        let import_code = if output.starts_with("Python 2.") {
            "from urllib import urlretrieve"
        } else if output.starts_with("Python 3.") {
            "from urllib.request import urlretrieve"
        } else {
            panic!();
        };

        Command::new(cmd_name.unwrap()).args(&["-c",
            &format!("{}; urlretrieve('{}', '{}')",
                import_code, DOWNLOAD_URL, FORMATTED_FILENAME)]).status().unwrap();
    }

    println!("Loading War and Peace....");

    // Load the text of War and Peace
    let mut paragraphs: Vec<String> = Vec::new();
    let mut current_paragraph_lines: Vec<String> = Vec::new();
    let mut f = File::open(FORMATTED_FILENAME).unwrap();
    let mut reader = BufReader::new(f);
    for line in reader.lines() {
        match line {
            Ok(line_string) => {
                // Remove leading and trailing whitespace
                let l = line_string.trim();

                if l.len() == 0 {
                    // Empty line; finish previous paragraph
                    {
                        let xs: Vec<&str> = current_paragraph_lines.iter().map(|x| x.as_str()).collect();
                        paragraphs.push(xs.join(" "));
                    }
                    current_paragraph_lines.clear();
                } else {
                    current_paragraph_lines.push(String::from(l));
                }
            },
            Err(..) => {panic!();}
        }
    }

    if current_paragraph_lines.len() > 0 {
        // Last paragraph
        let xs: Vec<&str> = current_paragraph_lines.iter().map(|x| x.as_str()).collect();
        paragraphs.push(xs.join(" "));
    }

    println!("Loaded War and Peace ({} paragraphs); creating presentation...", paragraphs.len());

    // Create a presentation of the text using the `lspace.pres` API
    let style = Rc::new(TextStyleParams::default());
    let mut pres_paragraphs: Vec<Pres> = Vec::new();
    for ref para_text in paragraphs {
        pres_paragraphs.push(paragraph(para_text, &style));
    }
    let content = Column::new(pres_paragraphs);

    println!("Loaded War and Peace; displaying....");

    // Create the LSpace widget, showing our content
    let lspace = LSpaceWidget::new(content);
    let widget = lspace.gtk_widget();

    // Create a GTK window in which to place it
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("War and Peace");
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
