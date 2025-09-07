// use proc_macro::TokenStream;
// use quote::{format_ident, quote};
// use serde::Deserialize;
// use serde::Serialize;
// use syn::DeriveInput;
// use syn::{parse_macro_input, parse_quote, ItemStruct};

// #[proc_macro_attribute]
// pub fn global_events(_attr: TokenStream, item: TokenStream) -> TokenStream {
//     let mut input = parse_macro_input!(item as DeriveInput);
//     let name = &input.ident;

//     // Append derives to the struct
//     // input.attrs.push(parse_quote!(#[derive(Default)]));

//     input.attrs.push(parse_quote!(#[derive(Clone)]));
//     input.attrs.push(parse_quote!(#[derive(serde::Serialize)]));
//     input
//         .attrs
//         .push(parse_quote!(#[derive(serde::Deserialize)]));

//     #[used]
//     let register_fn = format_ident!("global_event_{}", name);

//     let expanded = quote! {
//         #input

//         #[ctor::ctor]
//         #[allow(non_snake_case)]
//         fn #register_fn() {

//             println! ( "FOUND MACRO");
//             // core::collections::event_queue::EventQueue::register_global_events::<#name>( );
//         }
//     };

//     TokenStream::from(expanded)
// }
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, DeriveInput};

#[proc_macro_attribute]
pub fn global_events(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    // Append derives to the enum
    input.attrs.push(parse_quote!(#[derive(Clone)]));
    input.attrs.push(parse_quote!(#[derive(serde::Serialize)]));
    input
        .attrs
        .push(parse_quote!(#[derive(serde::Deserialize)]));

    let register_fn = format_ident!("global_event_{}", name);

    let expanded = quote! {
        #input

        #[ctor::ctor]
        // #[used] // ensure function isn’t discarded
        #[allow(non_snake_case)]
        fn #register_fn() {
            println!("FOUND MACRO for {}", stringify!(#name));
            core::collections::event_queue::EventQueue::register_global_events::<#name>( );
        }
    };

    TokenStream::from(expanded)
}
