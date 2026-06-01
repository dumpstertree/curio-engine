use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn curio_entry(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);

    let fn_name = &func.sig.ident;

    let expanded = quote! {
        #func

        #[unsafe(no_mangle)]
        pub extern "C" fn curio_init(
            services: *const EngineServices,
        ) -> *mut Curio {
            set_services(services);

            register_built_in_ecs();
            register_built_in_records();
            register_built_in_component();

            let curio = #fn_name();

            Box::into_raw(Box::new(curio))
        }
    };

    TokenStream::from(expanded)
}
