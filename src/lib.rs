/// simple helper for adding Dom to a slot
#[macro_export]
macro_rules! html_at_slot {
    ($name:expr, $slot:expr, { $($rest:tt)* }) => {
        html!($name, {
            .attribute("slot", $slot)
            $($rest)*
        })
    };
}
/// takes a literal and an ident and does the following:
/// 1. impls what's needed for dominator
/// 2. calls make_custom_event_serde! to enable the .data() helper (also for dominator)
/// 3. creates a function for testing the round-tripping from typescript (when ts_test feature is enabled)
/// NOTE: currently depends on using dakom/dominator fork
/// See: https://github.com/Pauan/rust-dominator/pull/25

#[macro_export]
macro_rules! make_event {
    ($literal:literal, $data:ident) => {
        paste::item! {
            dominator::make_custom_event_serde!($literal, [<$data Event>], $data);
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