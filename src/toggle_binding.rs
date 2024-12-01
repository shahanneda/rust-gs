use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::{prelude::Closure, JsCast, JsValue};

use crate::{scene::Scene, web::Settings, oct_tree::OctTree};

pub struct ToggleBinding {
    element_id: &'static str,
    pub key: String,
    get_value: Rc<dyn Fn(&Settings) -> bool>,
    set_value: Rc<dyn Fn(&mut Settings, bool)>,
    pub on_toggle: Rc<dyn Fn(&Settings, &mut Scene, &OctTree)>,
}

impl ToggleBinding {
    pub fn new(
        element_id: &'static str,
        key: &str,
        get_value: impl Fn(&Settings) -> bool + 'static,
        set_value: impl Fn(&mut Settings, bool) + 'static,
        on_toggle: impl Fn(&Settings, &mut Scene, &OctTree) + 'static,
    ) -> Self {
        Self {
            element_id,
            key: key.to_string(),
            get_value: Rc::new(get_value),
            set_value: Rc::new(set_value),
            on_toggle: Rc::new(on_toggle),
        }
    }

    pub fn setup_ui_listener(
        &self,
        settings: Rc<RefCell<Settings>>,
        scene: Rc<RefCell<Scene>>,
        oct_tree: Rc<RefCell<OctTree>>,
    ) -> Result<(), JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let checkbox = document
            .get_element_by_id(self.element_id)
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap();

        let checkbox_clone = checkbox.clone();
        let get_value = self.get_value.clone();
        let set_value = self.set_value.clone();
        let on_toggle = self.on_toggle.clone();

        let checkbox_callback = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            let mut settings = settings.borrow_mut();
            let new_value = checkbox_clone.checked();
            (set_value)(&mut settings, new_value);

            let mut scene = scene.borrow_mut();
            let oct_tree = oct_tree.borrow();
            (on_toggle)(&settings, &mut scene, &oct_tree);
        }) as Box<dyn FnMut(_)>);

        checkbox.add_event_listener_with_callback(
            "change",
            checkbox_callback.as_ref().unchecked_ref(),
        )?;
        checkbox_callback.forget();
        Ok(())
    }

    pub fn update_ui(&self, settings: &Settings) {
        let document = web_sys::window().unwrap().document().unwrap();
        if let Ok(checkbox) = document
            .get_element_by_id(self.element_id)
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
        {
            checkbox.set_checked((self.get_value)(settings));
        }
    }

    pub fn handle_key_press(&self, settings: &mut Settings) {
        let current = (self.get_value)(settings);
        (self.set_value)(settings, !current);
    }
}
