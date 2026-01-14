use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemStruct};

#[proc_macro_attribute]
pub fn global_state(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;
    let register_fn = format_ident!("render_feature_driver{}", name);

    let expanded = quote! {
        #input

        #[ctor::ctor]
        #[allow(non_snake_case)]
        fn #register_fn() {
           curio_core::collections::game_state::GameState::register_global_states::<#name>();
        }
    };

    TokenStream::from(expanded)
}
