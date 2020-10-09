mod macro_helpers;
mod async_helpers;

pub use macro_helpers::*;
pub use async_helpers::*;


#[cfg(test)]
mod tests {
    use dominator::{DomBuilder, Dom, html, events, clone, apply_methods};
    use wasm_bindgen::prelude::*;
    use web_sys::HtmlImageElement;
    use super::*; 

    #[test]
    fn it_works() {
        let elem = html!("<div>hello</div>", { 
            .with_data_id!("img" => HtmlImageElement, {
                .property("src", "foo")
            })
        });
    }
}
