use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, ItemStruct};

#[proc_macro_attribute]
pub fn global_ecs_system(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    // Append derives to the struct
    input.attrs.push(parse_quote!(#[derive(Default)]));
    // input.attrs.push(parse_quote!(#[derive(Clone)]));

    let register_fn = format_ident!("global_ecs_system_{}", name);

    let expanded = quote! {
        #input

        #[ctor::ctor]
        #[allow(non_snake_case)]
        fn #register_fn() {
            fn _check_system<T: system_component_default_gameplay::ecs_system::ECSSystemEventless>() {}
            _check_system::<#name>();

           system_component_default_gameplay::static_data::global_ecs::register_global_ecs::<#name>();
        }
    };

    TokenStream::from(expanded)
}
