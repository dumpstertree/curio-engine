use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, ItemStruct};

#[proc_macro_attribute]
pub fn record_serializable(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut ownership_val = None;
    let mut serializable = false;

    {
        let parser = syn::meta::parser(|meta| {
            if meta.path.is_ident("ownership") {
                ownership_val = Some(meta.value()?.parse::<syn::Path>()?);
                Ok(())
            } else if meta.path.is_ident("serializable") {
                serializable = true;
                Ok(())
            } else {
                Err(meta.error("unsupported argument — expected `ownership` or `serializable`"))
            }
        });
        parse_macro_input!(attr with parser);
    }

    let ownership = match ownership_val {
        Some(path) => quote! { #path },
        None => quote! { curio_core::StateOwnerships::Instance },
    };

    let mut input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    // Always derived
    input.attrs.push(parse_quote!(#[derive(Default)]));
    input.attrs.push(parse_quote!(#[derive(Clone)]));

    // Only derived when serializable
    if serializable {
        input.attrs.push(parse_quote!(#[derive(serde::Serialize)]));
        input
            .attrs
            .push(parse_quote!(#[derive(serde::Deserialize)]));
    }

    let register_fn = format_ident!("__global_state_register_{}", name);
    let static_id = format_ident!("__RECORD_ID_{}", name);

    // Pick the correct registration call
    let register_call = if serializable {
        quote! { curio_core::static_data::global_states::register_global_state_serializable::<#name>(); }
    } else {
        quote! { curio_core::static_data::global_states::register_global_state::<#name>(); }
    };

    let expanded = quote! {
        #input

        static #static_id: std::sync::OnceLock<i32> = std::sync::OnceLock::new();

        impl curio_core::RecordCommon for #name {
            fn id() -> i32 where Self: Sized + 'static {
                *#static_id.get_or_init(|| {
                    curio_core::system::record_id::RecordId::of::<#name>()
                })
            }

            fn ownership() -> curio_core::StateOwnerships
            where
                Self: Sized + 'static,
            {
                #ownership
            }
        }

        #[ctor::ctor]
        #[allow(non_snake_case)]
        fn #register_fn() {
            #register_call
        }
    };

    TokenStream::from(expanded)
}
