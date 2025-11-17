use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, ItemStruct};

#[proc_macro_attribute]
pub fn global_state(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    // Append derives if not already present
    input
        .attrs
        .push(parse_quote!(#[derive(Default, Clone, PartialEq)]));

    // Implement Send + Sync + IState for the struct automatically
    let register_fn = format_ident!("global_state_{}", name);

    let expanded = quote! {
        #input

        // Automatically implement IState for this type
        // impl core::collections::game_state::IState for #name {
        //     fn clone_box(&self) -> Box<dyn core::collections::game_state::IState> {
        //         Box::new(self.clone())
        //     }
        // }

        unsafe impl Send for #name {}
        unsafe impl Sync for #name {}

        #[ctor::ctor]
        #[allow(non_snake_case)]
        fn #register_fn() {
            core::collections::game_state::GameState::register_global_states::<#name>();
        }
    };

    TokenStream::from(expanded)
}
