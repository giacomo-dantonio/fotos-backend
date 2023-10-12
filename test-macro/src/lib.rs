// use std::panic;

use proc_macro::TokenStream;
use syn::parse_macro_input;

type FA<R> = fn(i32) -> R;
struct AttributeInput {
    setup: FA<()>,
    teardown: FA<()>
}

// FIXME: look at this https://blog.logrocket.com/procedural-macros-in-rust/
// https://doc.rust-lang.org/reference/procedural-macros.html
#[proc_macro_attribute]
pub fn run_test(attr: TokenStream, test: TokenStream) -> TokenStream {
    let AttributeInput{ setup, teardown } = parse_macro_input!(attr);
    match data {

    }



    // setup();

    // let result = panic::catch_unwind(|| {
    //     test()
    // });

    // teardown();

    test
}