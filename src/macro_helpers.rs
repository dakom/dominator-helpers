/// pass in a HtmlElement and get a Dom
#[macro_export]
macro_rules! elem {
    ($elem:expr, { $($methods:tt)* }) => {
        dominator::apply_methods!(dominator::DomBuilder::new($elem), { $($methods)* }).into_dom()
    };
}

/// Method to get a child by query 
/// example:
/// .with_query!("[data-id='status']", {
///     .event(...)
///
#[macro_export]
macro_rules! with_query {
    ($this:ident, $query:expr => $t:ty, { $($methods:tt)* }) => {
        dominator::with_node!($this, element => {
            .__internal_transfer_callbacks({
                let child = element.query_selector($query).unwrap_throw().unwrap_throw();
                let child: $t = wasm_bindgen::JsCast::dyn_into(child).unwrap_throw();
                dominator::apply_methods!(dominator::DomBuilder::new(child), { $($methods)* })
            })
        })
    };
    ($this:ident, $query:expr, { $($methods:tt)* }) => {
        $crate:with_query!($this, $query => web_sys::HtmlElement, { $($methods)* })
    };
}

/// Method to get a child by data-id
/// example:
/// .with_data_id!("status", {
///     .event(...)
#[macro_export]
macro_rules! with_data_id {
    ($this:ident, $id:expr => $t:ty, { $($methods:tt)* }) => {
        $crate::with_query($this, &format!("[data-id='{}']", $id) => $t, { $($methods)* })
    };
    ($this:ident, $id:expr, { $($methods:tt)* }) => {
        $crate:with_data_id!($this, $id => web_sys::HtmlElement, { $($methods)* })
    };
}

/// Create an element type at a slot (useful for web components) 
/// e.g. this will create the "todo-input" element with its "slot" attribute set to "input"
/// html_at_slot!("todo-input", "input", { ... }
#[macro_export]
macro_rules! html_at_slot {
    ($name:expr, $slot:expr, { $($rest:tt)* }) => {
        dominator::html!($name, {
            .attribute("slot", $slot)
            $($rest)*
        })
    };
}


/// TODO - relies on dominator exporting make_event!
/// takes a literal and an ident and does the following:
/// 1. impls what's needed for dominator
/// 2. calls make_custom_event_serde! to enable the .data() helper (also for dominator)
/// 3. creates a function for testing the round-tripping from typescript (when ts_test feature is enabled)
#[macro_export]
macro_rules! make_ts_event {
    ($literal:literal, $data:ident) => {
        paste::item! {
            make_custom_event_serde!($literal, [<$data Event>], $data);
            cfg_if::cfg_if! {
                if #[cfg(feature = "ts_test")] {

                    use web_sys::CustomEvent;
                    use wasm_bindgen::prelude::*;

                    #[wasm_bindgen]
                    pub fn [<check_rust_event_ $data>](event:CustomEvent) -> Result<JsValue, JsValue> {
                        let literal = event.type_();
                        let event:[<$data Event>] = unsafe {
                            std::mem::transmute::<CustomEvent, [<$data Event>]>(event)
                        };

                        if literal == $literal {
                            let data:$data = event.data(); 
                            let expected = serde_json::to_string(&$data::default()).unwrap();
                            let got = serde_json::to_string(&data).unwrap();
                            if expected != got {
                                Err(JsValue::from_str(&format!("did not match default! should be {} but is {}", expected, got)))
                            } else {
                                Ok(JsValue::from_str(&got))
                            }
                        } else {
                            Err(JsValue::from_str(&format!("wrong type! should be {} but is {}", $literal, literal)))
                        }
                    }
                }
            }
        }
    }
}

/// TODO - relies on dominator exporting make_event!
#[macro_export]
macro_rules! make_custom_event {
    ($name:ident, $type:literal) => {
        dominator::make_event!($name, $type => web_sys::CustomEvent);
        impl $name {
            pub fn detail(&self) -> JsValue { self.event.detail() }
        }
    }
}

/// TODO - relies on dominator exporting make_event!
/// first arg is name of the new struct to create
/// second arg is literal name of the event
/// third arg is the data structure. 
/// 
/// the data structure needs to already be defined and derive `Deserialize`
/// 
/// requires that the serde_wasm_bindgen crate be installed 
/// however, since this is only a macro, there's no need to feature-gate it here
/// Example:
/// 
/// JS (e.g. from a CustomElement)
/// 
/// ```javascript
/// this.dispatchEvent(new CustomEvent('todo-input', {
///     detail: {label: value}
/// }));
/// ```
/// 
/// Rust - first register the event
/// 
/// ```rust
/// 
/// #[derive(Deserialize)]
/// pub struct TodoInputEventData {
///     pub label: String 
/// }
/// make_custom_event_serde!("todo-input", TodoInputEvent, TodoInputEventData);
/// ``` 
/// 
/// then use it
///
/// ```
/// html!("todo-custom", {
///     .event(|event:TodoInputEvent| {
///         //event.data() is a TodoInputEventData
///         let label:&str = event.data().label;
///     })
/// })
/// ```
#[macro_export]
macro_rules! make_custom_event_serde {
    ($type:literal, $name:ident, $data:ident) => {
        make_custom_event!($name, $type);
        impl $name {
            pub fn data(&self) -> $data { 
                serde_wasm_bindgen::from_value(self.detail()).unwrap()
            }
        }
    }
}
