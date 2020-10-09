#![feature(trace_macros)]
trace_macros!(true);

mod macro_helpers;
mod async_helpers;

pub use macro_helpers::*;
pub use async_helpers::*;


#[cfg(test)]
mod tests {
    use dominator::{DomBuilder, Dom, html, events, clone, apply_methods};
    use super::*; 

    #[test]
    fn it_works() {
        let elem = elem!("<div>hello</div>", { 
            .with_data_id!("img" => HtmlElement, {
                .property("src", "foo")
            })
        });
    }
}
