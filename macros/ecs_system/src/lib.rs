use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(ECSSystem)]
pub fn ecs_system_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let register_fn = format_ident!("__ecs_systemt_{}", name);

    let expanded = quote! {
        #[ctor::ctor]
        fn #register_fn() {

            fn _check_system<T: crate::system::system_components::gameplay_components::gameplay_component_default::ECSSystemEventless>() {}
            _check_system::<#name>();

            //
            fn _check_default<T: Default>() {}
            _check_default::<#name>();

            // create instance
            crate::dumpster_engine::DumpsterEngine::register_ecs_system::<#name>();

        }
    };

    TokenStream::from(expanded)
}
