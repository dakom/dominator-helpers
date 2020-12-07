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
                let error_msg = &format!("unable to get element for {}", $query);
                let child = element.query_selector($query).expect_throw(error_msg).expect_throw(error_msg);
                let child: $t = wasm_bindgen::JsCast::dyn_into(child).expect_throw(error_msg);
                dominator::apply_methods!(dominator::DomBuilder::new(child), { $($methods)* })
            })
        })
    };
    ($this:ident, $query:expr, { $($methods:tt)* }) => {
        $crate::with_query!($this, $query => web_sys::HtmlElement, { $($methods)* })
    };
}

/// Method to get a child by data-id
/// example:
/// .with_data_id!("status", {
///     .event(...)
///
#[macro_export]
macro_rules! with_data_id {
    ($this:ident, $id:expr => $t:ty, { $($methods:tt)* }) => {
        $crate::with_query!($this, &format!("[data-id='{}']", $id) => $t, { $($methods)* })
    };
    ($this:ident, $id:expr, { $($methods:tt)* }) => {
        $crate::with_data_id!($this, $id => web_sys::HtmlElement, { $($methods)* })
    };
}

/// Placeholder for https://github.com/Pauan/rust-dominator/issues/44
#[macro_export]
macro_rules! dynamic_class_signal {
    ($this:ident, $signal:expr) => {
        dominator::with_node!($this, element => {
            .__internal_transfer_callbacks({
                dominator::apply_methods!(dominator::DomBuilder::new(element.clone()), {
                    .future({
                        let mut old = None;
                        $signal.for_each(move |class| {
                            if let Some(old) = old.as_deref() {
                                element.class_list().remove_1(old).unwrap();
                            }

                            if let Some(name) = class.as_deref() {
                                element.class_list().add_1(&name).unwrap();
                            }
                            old = class;

                            async {}
                        })
                    })
                })
            })
        })
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
