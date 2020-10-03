use std::rc::Rc;
use std::cell::{RefCell, Ref, RefMut};
use std::string::String;
use std::mem::transmute;

use cairo::Context;
use cairo::{FontSlant, FontWeight};

use layout::lreq::LReq;
use layout::lalloc::LAlloc;
use geom::bbox2::BBox2;
use geom::colour::{Colour, BLACK};
use elements::element_layout::{ElementReq, ElementAlloc};
use elements::element_ctx::{ElementContext, ElementLayoutContext};
use elements::element::{TElement, ElementRef, ElementParentMut, queue_resize};
use elements::container::{TContainerElement};
use elements::bin::{TBinElement};
use elements::container_sequence::{TContainerSequenceElement};
use elements::root_element::{TRootElement};
use lspace_area::LSpaceArea;


pub trait TTextElement : TElement {
    fn get_text(&self) -> Ref<String>;
    fn set_text(&self, text: String);
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TextWeight {
    Normal,
    Bold,
}

impl TextWeight {
    pub fn as_cairo(&self) -> FontWeight {
        match self {
            &TextWeight::Normal => FontWeight::Normal,
            &TextWeight::Bold => FontWeight::Bold,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TextSlant {
    Normal,
    Italic,
}

impl TextSlant {
    pub fn as_cairo(&self) -> FontSlant {
        match self {
            &TextSlant::Normal => FontSlant::Normal,
            &TextSlant::Italic => FontSlant::Italic,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextStyleParams {
    pub font_family: String,
    pub weight: TextWeight,
    pub slant: TextSlant,
    pub size: f64,
    pub colour: Colour,
}

impl TextStyleParams {
    pub fn new(font_family: String, weight: TextWeight, slant: TextSlant,
               size: f64, colour: &Colour) -> TextStyleParams {
        return TextStyleParams{font_family: font_family, weight: weight, slant: slant, size: size,
                               colour: colour.clone()};
    }

    pub fn default() -> TextStyleParams {
        return TextStyleParams{font_family: String::from("Sans serif"),
                               weight: TextWeight::Normal, slant: TextSlant::Normal,
                               size: 14.0, colour: BLACK};
    }

    pub fn with_family(font_family: String) -> TextStyleParams {
        return TextStyleParams{font_family: font_family,
                               weight: TextWeight::Normal, slant: TextSlant::Normal,
                               size: 14.0, colour: BLACK};
    }

    pub fn with_family_and_colour(font_family: String, colour: &Colour) -> TextStyleParams {
        return TextStyleParams{font_family: font_family,
                               weight: TextWeight::Normal, slant: TextSlant::Normal,
                               size: 14.0, colour: colour.clone()};
    }

    pub fn text_req_key(&self, text: String) -> TextReqKey {
        // In all the cases we care about the font size will not be an invalid FP number.
        // As a consequence of Rust's stick-up-its-arse handling of floats we have to use
        // some nasty unsafe code to get something that we can get a hash code from...
        let hashable_size: i64 = unsafe{transmute(self.size)};
        return (self.font_family.clone(), self.weight, self.slant, hashable_size, text);
    }

    pub fn apply(&self, cairo_ctx: &Context) {
        cairo_ctx.select_font_face(self.font_family.as_str(), self.slant.as_cairo(),
                                   self.weight.as_cairo());
        cairo_ctx.set_font_size(self.size);
        cairo_ctx.set_source_rgba(self.colour.r as f64, self.colour.g as f64, self.colour.b as f64,
                                  self.colour.a as f64);
    }
}

pub type TextReqKey = (String, TextWeight, TextSlant, i64, String);


struct TextElementMut {
    parent: ElementParentMut,
    req: Rc<ElementReq>,
    alloc: ElementAlloc,
    text: String,
    req_up_to_date: bool
}

pub struct TextElement {
    style: Rc<TextStyleParams>,
    m: RefCell<TextElementMut>,
}

impl TextElement {
    pub fn new(text: String, style: Rc<TextStyleParams>,
               elem_ctx: &ElementContext) -> TextElement {
        return TextElement{style: style,
                           m: RefCell::new(TextElementMut{
                                parent: ElementParentMut::new(),
                                req: elem_ctx.empty_shared_req(),
                                alloc: ElementAlloc::new(),
                                text: text,
                                req_up_to_date: false}),
                           };
    }

    pub fn new_in_area(text: String, style: Rc<TextStyleParams>,
                       area: &Rc<LSpaceArea>) -> TextElement {
        TextElement::new(text, style, &*area.element_context())
    }
}

impl TElement for TextElement {
    /// Interface acquisition
    fn as_text_element(&self) -> Option<&TTextElement> {
        Some(self)
    }

    fn as_container(&self) -> Option<&TContainerElement> {
        return None;
    }

    fn as_bin(&self) -> Option<&TBinElement> {
        return None
    }

    fn as_container_sequence(&self) -> Option<&TContainerSequenceElement> {
        return None
    }

    fn as_root_element(&self) -> Option<&TRootElement> {
        return None;
    }
    
    /// Parent get and set methods
    fn get_parent(&self) -> Option<ElementRef> {
        return self.m.borrow().parent.get().clone();
    }

    fn set_parent(&self, p: Option<&ElementRef>) {
        self.m.borrow_mut().parent.set(p);
    }

    // Element structure acquisition
    fn element_req(&self) -> Ref<ElementReq> {
        return Ref::map(self.m.borrow(), |m| &(*m.req));
    }

    fn element_alloc(&self) -> Ref<ElementAlloc> {
        return Ref::map(self.m.borrow(), |m| &m.alloc);
    }

    fn element_alloc_mut(&self) -> RefMut<ElementAlloc> {
        return RefMut::map(self.m.borrow_mut(), |m| &mut m.alloc);
    }

    /// Update element X requisition
    fn element_update_x_req(&self, x_req: &LReq) -> bool {
        return false;
    }

    /// Update element Y requisition
    fn element_update_y_req(&self, y_req: &LReq) -> bool {
        return false;
    }

    // Draw
    fn draw_self(&self, cairo_ctx: &Context, visible_region: &BBox2) {
        let mm = self.m.borrow();
        let y = match mm.alloc.y_alloc.ref_point() {
            None => 0.0,
            Some(ref_point) => ref_point
        };
        cairo_ctx.move_to(0.0, y);
        self.style.apply(cairo_ctx);
        cairo_ctx.show_text(mm.text.as_str());
    }

    fn draw(&self, cairo_ctx: &Context, visible_region: &BBox2) {
        self.draw_self(cairo_ctx, visible_region);
    }

    // Update layout
    fn update_x_req(&self, layout_ctx: &ElementLayoutContext) -> bool {
        // Nothing to do; requisition is shared
        let mut mm = self.m.borrow_mut();

        if !mm.req_up_to_date {
            let elem_ctx = layout_ctx.elem_ctx();
            let req = elem_ctx.text_shared_req(self.style.clone(), mm.text.clone(),
                                               layout_ctx.cairo_ctx());
            mm.req = req;
            mm.req_up_to_date = true;
        }

        let updated = mm.alloc.is_x_req_update_required();
        mm.alloc.x_req_updated();
        return updated;
    }

    fn allocate_x(&self, x_alloc: &LAlloc) -> bool {
        // Nothing to do; no children
        let mut mm = self.m.borrow_mut();
        mm.alloc.update_x_alloc(x_alloc);
        mm.alloc.x_alloc_updated();
        return false;
    }

    fn update_y_req(&self) -> bool {
        // Nothing to do; requisition is shared
        let mut mm = self.m.borrow_mut();
        let updated = mm.alloc.is_y_req_update_required();
        mm.alloc.y_req_updated();
        return updated;
    }

    fn allocate_y(&self, y_alloc: &LAlloc) {
        // Nothing to do; no children
        let mut mm = self.m.borrow_mut();
        mm.alloc.update_y_alloc(y_alloc);
        mm.alloc.y_alloc_updated();
    }
}


impl TTextElement for TextElement {
    fn get_text(&self) -> Ref<String> {
        let mm = self.m.borrow();
        return Ref::map(mm, |m| &m.text);
    }

    fn set_text(&self, text: String) {
        {
            let mut mm = self.m.borrow_mut();
            mm.text = text;
            mm.req_up_to_date = false;
        }
        queue_resize(self);
    }
}
