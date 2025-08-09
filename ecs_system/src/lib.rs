use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, ItemStruct};

#[proc_macro_attribute]
pub fn ECSSystem(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    // Append derives to the struct
    input.attrs.push(parse_quote!(#[derive(Default)]));
    input.attrs.push(parse_quote!(#[derive(Clone)]));

    let register_fn = format_ident!("_ecs_system_{}", name);

    let expanded = quote! {
        #input

        #[ctor::ctor]
        fn #register_fn() {
            fn _check_system<T: core::gameplay::ecs::traits::ecs_system::ECSSystemEventless>() {}
            _check_system::<#name>();

            core::dumpster_engine::DumpsterEngine::register_ecs_system::<#name>( );
        }
    };

    TokenStream::from(expanded)
}
