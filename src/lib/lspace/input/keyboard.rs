use std::cell::RefCell;
use std::rc::Rc;

use input::inputmodifier::InputModifierState;

use gdk::keys::Key;


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum KeyEventType {
    Press,
    Release
}

#[derive(Debug)]
pub struct KeyEvent {
    mod_state: InputModifierState,
    key_val: Key,
    key_string: Option<String>,
    event_type: KeyEventType
}

impl KeyEvent {
    pub fn new_press(mod_state: InputModifierState, key_val: Key, key_string: Option<String>) -> KeyEvent {
        KeyEvent{mod_state: mod_state, key_val: key_val, key_string: key_string,
            event_type: KeyEventType::Press}
    }

    pub fn new_release(mod_state: InputModifierState, key_val: Key, key_string: Option<String>) -> KeyEvent {
        KeyEvent{mod_state: mod_state, key_val: key_val, key_string: key_string,
            event_type: KeyEventType::Release}
    }

    pub fn mod_state(&self) -> InputModifierState {
        self.mod_state
    }

    pub fn key_val(&self) -> Key {
        self.key_val.clone()
    }

    pub fn key_string(&self) -> &Option<String> {
        &self.key_string
    }

    pub fn event_type(&self) -> KeyEventType {
        self.event_type
    }
}


pub trait TKeyboardInteractor {
    fn on_key_event(&self, event: &KeyEvent);
}

fn are_same(a: &Rc<TKeyboardInteractor>, b: &Rc<TKeyboardInteractor>) -> bool {
    let aptr: &TKeyboardInteractor = &**a;
    let bptr: &TKeyboardInteractor = &**b;
    return (aptr as *const TKeyboardInteractor) == (bptr as *const TKeyboardInteractor);
}


struct KeyboardMut {
    interactors: Vec<Rc<TKeyboardInteractor>>
}

impl KeyboardMut {
    fn new() -> KeyboardMut {
        KeyboardMut{interactors: Vec::new()}
    }

    fn on_key_press(&self, mod_state: InputModifierState, key_val: Key, key_string: Option<String>) {
        let event = KeyEvent::new_press(mod_state, key_val, key_string);
        for ref interactor in self.interactors.iter() {
            interactor.on_key_event(&event);
        }
    }

    fn on_key_release(&self, mod_state: InputModifierState, key_val: Key, key_string: Option<String>) {
        let event = KeyEvent::new_release(mod_state, key_val, key_string);
        for ref interactor in self.interactors.iter() {
            interactor.on_key_event(&event);
        }
    }

    fn add_interactor(&mut self, interactor: &Rc<TKeyboardInteractor>) {
        self.interactors.push(interactor.clone());
    }

    fn remove_interactor(&mut self, interactor: &Rc<TKeyboardInteractor>) {
        let mut index_to_remove: Option<usize> = None;
        for ndx_item in self.interactors.iter().enumerate() {
            if are_same(&ndx_item.1, interactor) {
                index_to_remove = Some(ndx_item.0);
                break;
            }
        }
        match index_to_remove {
            Some(i) => {self.interactors.remove(i);},
            None => {}
        }
    }
}


pub struct Keyboard {
    m: RefCell<KeyboardMut>
}

impl Keyboard {
    pub fn new() -> Keyboard {
        return Keyboard{
            m: RefCell::new(KeyboardMut::new())
        };
    }

    pub fn on_key_press(&self, mod_state: InputModifierState, key_val: Key, key_string: Option<String>) {
        self.m.borrow().on_key_press(mod_state, key_val, key_string);
    }

    pub fn on_key_release(&self, mod_state: InputModifierState, key_val: Key, key_string: Option<String>) {
        self.m.borrow().on_key_release(mod_state, key_val, key_string);
    }

    pub fn add_interactor(&self, interactor: &Rc<TKeyboardInteractor>) {
        self.m.borrow_mut().add_interactor(interactor);
    }

    pub fn remove_interactor(&self, interactor: &Rc<TKeyboardInteractor>) {
        self.m.borrow_mut().remove_interactor(interactor);
    }
}
