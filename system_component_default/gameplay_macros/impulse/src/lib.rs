// use proc_macro::TokenStream;
// use quote::quote;
// use syn::{parse_macro_input, parse_quote, ItemImpl, Type};

// #[proc_macro_attribute]
// pub fn global_ecs_system_event_reciever(attr: TokenStream, item: TokenStream) -> TokenStream {
//     let event_type = parse_macro_input!(attr as Type);
//     let mut input = parse_macro_input!(item as ItemImpl);

//     let self_ty = &input.self_ty;

//     // Add #[intertrait::cast_to]
//     input.attrs.push(parse_quote!(#[intertrait::cast_to]));

//     let expanded = quote! {
//         #input

//         global_ecs_system_event_reciever::Register

//         // // This enforces the trait bound at compile time
//         // struct _EnforceEventReceiver_ where #self_ty: core::gameplay::ecs::traits::ecs_event_reciever::EventReciever<#event_type> { }
//         // struct _EnforceSystem where #self_ty: crate::system::system_components::gameplay_components::gameplay_component_default::ECSSystemEventless { }
//     };

//     TokenStream::from(expanded)
// }

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
use syn::{parse_macro_input, DeriveInput, Path};

#[proc_macro_attribute]
pub fn impulse(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input struct
    let input = parse_macro_input!(item as DeriveInput);

    // Parse the attribute as a Path (e.g. `MyEvent` or `my::events::Event`)
    let event_path = {
        let parsed = parse_macro_input!(attr as Path);
        // ensure attribute wasn't empty by checking last segment exists
        if parsed.segments.is_empty() {
            return syn::Error::new_spanned(input.ident.clone(), "Expected event type: #[global_ecs_system_event_reciever(EventType)]")
                .to_compile_error()
                .into();
        }
        parsed
    };

    let receiver_name = &input.ident;

    // Append derives to the struct
    // input.attrs.push(parse_quote!(#[derive(Default)]));
    // input.attrs.push(parse_quote!(#[derive(Clone)]));
    // input.attrs.push(parse_quote!(#[derive(serde::Serialize)]));
    // input
    //     .attrs
    //     .push(parse_quote!(#[derive(serde::Deserialize)]));

    // Create a unique function name for registration.
    // Use the last segment of the event path for readability.
    let event_ident = &event_path.segments.last().unwrap().ident;
    let register_fn = format_ident!("register_event_reciever_{}_for_{}", receiver_name, event_ident);

    // Generate the expanded tokens
    let expanded = quote! {
        #input

        #[ctor::ctor]
        #[allow(non_snake_case)]
        fn #register_fn() {
            gameplay::static_data::global_event_recievers::register_global_event_receiver::<#event_path,#receiver_name >();
        }
    };

    TokenStream::from(expanded)
}
