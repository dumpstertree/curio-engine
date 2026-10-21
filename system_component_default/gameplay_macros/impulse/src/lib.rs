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
//         // struct _EnforceSystem where #self_ty: crate::system::system_components::gameplay_components::gameplay_component_default::ECsystemEventless { }
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

/// Implements `curio_core::ImpulseCommon` for a game's impulse enum with zero boilerplate.
///
/// Generates `Clone` and a process-unique `ImpulseCommon::id()` (no more hardcoded `0`s that
/// collide across impulse types). `ownership()` defaults to `ImpulseScope::Instance` and can be
/// overridden with an attribute argument:
///
/// ```ignore
/// #[curio_impulse]
/// pub enum GameEvents {}
///
/// #[curio_impulse(scope = "connected_peers")]
/// pub enum NetEvents {
///     FireCannon { at: u64 },
/// }
/// ```
#[proc_macro_attribute]
pub fn curio_impulse(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    // resolve the optional scope override: scope = "connected_peers"
    let mut scope = quote! { curio_core::ImpulseScope::Instance };
    if !attr.is_empty() {
        let raw = attr.to_string().trim().to_owned();
        let key = match raw.split_once('=') {
            Some((lhs, rhs)) => {
                if lhs.trim() != "scope" {
                    return syn::Error::new_spanned(input.ident.clone(), format!("curio_impulse: unknown argument '{lhs}' — expected scope = \"...\"")).to_compile_error().into();
                }
                rhs.trim().trim_matches('"').to_owned()
            }
            None => raw.trim().trim_matches('"').to_owned(),
        };
        match key.as_str() {
            "instance" => scope = quote! { curio_core::ImpulseScope::Instance },
            "all" => scope = quote! { curio_core::ImpulseScope::All },
            "connected_host" => scope = quote! { curio_core::ImpulseScope::ConnectedHost },
            "connected_peers" => scope = quote! { curio_core::ImpulseScope::ConnectedPeers },
            other => {
                return syn::Error::new_spanned(input.ident.clone(), format!("curio_impulse: unknown scope '{other}' — expected one of instance, all, connected_host, connected_peers"))
                    .to_compile_error()
                    .into();
            }
        }
    }

    let expanded = quote! {
        #[derive(Clone)]
        #input

        impl curio_core::ImpulseCommon for #name {
            fn id() -> i32
            where
                Self: Sized + 'static,
            {
                // one lazily assigned unique id per impulse type
                static IMPULSE_ID: std::sync::LazyLock<i32> = std::sync::LazyLock::new(curio_core::next_impulse_id);
                *IMPULSE_ID
            }

            fn ownership(&self) -> curio_core::ImpulseScope
            where
                Self: Sized + 'static,
            {
                #scope
            }
        }
    };
    TokenStream::from(expanded)
}

/// Implements `gameplay::traits::ui_events::IUIEvent` for a game's UI event enum with zero
/// boilerplate. Generates `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash` and a `Display` that prints
/// the variant name. `new_instance` is unreachable for empty enums and panics with a clear
/// message for enums that have no panels implemented yet.
///
/// ```ignore
/// #[curio_ui_event]
/// pub enum UIEvents {}
/// ```
#[proc_macro_attribute]
pub fn curio_ui_event(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    let variants = match &input.data {
        syn::Data::Enum(data) => data.variants.iter().map(|v| &v.ident).collect::<Vec<_>>(),
        _ => {
            return syn::Error::new_spanned(input.ident.clone(), "curio_ui_event: can only be applied to enums")
                .to_compile_error()
                .into();
        }
    };

    let display_arms = variants.iter().map(|v| quote! { Self::#v { .. } => write!(f, stringify!(#v)), });
    let new_instance_arms = variants.iter().map(|v| {
        let msg = format!("curio_ui_event: UIEvents::{} has no UIPanel implementation", v);
        quote! { Self::#v { .. } => unimplemented!(#msg), }
    });

    let expanded = quote! {
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        #input

        impl core::fmt::Display for #name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match *self {
                    #(#display_arms)*
                }
            }
        }

        impl gameplay::traits::ui_events::IUIEvent for #name {
            fn new_instance(&self) -> Box<dyn gameplay::traits::ui_panel::UIPanel> {
                match *self {
                    #(#new_instance_arms)*
                }
            }
        }
    };
    TokenStream::from(expanded)
}
