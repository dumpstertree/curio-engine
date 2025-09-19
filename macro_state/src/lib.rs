use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, ItemStruct};

#[proc_macro_attribute]
pub fn global_state(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    // Append derives to the struct
    input.attrs.push(parse_quote!(#[derive(Default)]));
    input.attrs.push(parse_quote!(#[derive(Clone)]));
    // input.attrs.push(parse_quote!(#[derive(serde::Serialize)]));
    // input
    // .attrs
    // .push(parse_quote!(#[derive(serde::Deserialize)]));

    let register_fn = format_ident!("global_state_{}", name);

    let expanded = quote! {
        #input

        #[ctor::ctor]
        #[allow(non_snake_case)]
        fn #register_fn() {
           core::collections::game_state::GameState::register_global_states::<#name>();
            // core::dumpster_engine::DumpsterEngine::register_global_ecs_system::<#name>( );
        }
    };

    TokenStream::from(expanded)
}
