// use proc_macro::TokenStream;
// use quote::{format_ident, quote};
// use syn::{parse_macro_input, DeriveInput};

// #[proc_macro_derive(RegisterType)]
// pub fn register_type_derive(input: TokenStream) -> TokenStream {
//     let ast = parse_macro_input!(input as DeriveInput);
//     let name = &ast.ident;
//     let fn_ident = format_ident!("__auto_register_type_id_{}", name);

//     let expanded = quote! {
//         impl #name {
//             fn __register_type_id() {
//                 println!("registed");
//                 crate::dumpster_engine::DumpsterEngine::reg_type (std::any::TypeId::of::<Self>() );
//                 // let mut registry = TYPE_REGISTRY.lock().unwrap();
//                 // registry.insert(std::any::TypeId::of::<Self>());
//             }
//         }

//         #[ctor::ctor]
//         fn #fn_ident() {
//             #name::__register_type_id();
//         }
//     };

//     TokenStream::from(expanded)
// }
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(RegisterComponent)]
pub fn register_component_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let register_fn = format_ident!("__register_component_{}", name);

    let expanded = quote! {
        #[derive(serde::Serialize, Clone)]
        #input

        #[ctor::ctor]
        fn #register_fn() {
            crate::register_component::<#name>(stringify!(#name));
        }
    };

    TokenStream::from(expanded)
}

// #[proc_macro_derive(RegisterComponent2)]
// pub fn register_component_derive2(input: TokenStream) -> TokenStream {
//     let expanded = quote! {
//         #[derive(serde::Serialize, Clone)]
//         #input

//         #[ctor::ctor]
//         fn #register_fn() {
//             crate::register_component::<#name>(stringify!(#name));
//         }
//     };

//     TokenStream::from(expanded)
// }
