use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, ItemImpl, Type};

#[proc_macro_attribute]
pub fn ECSEvent(attr: TokenStream, item: TokenStream) -> TokenStream {
    let event_type = parse_macro_input!(attr as Type);
    let mut input = parse_macro_input!(item as ItemImpl);

    let self_ty = &input.self_ty;

    // Add #[intertrait::cast_to]
    input.attrs.push(parse_quote!(#[intertrait::cast_to]));

    let expanded = quote! {
        #input

        // This enforces the trait bound at compile time
        struct _EnforceEventReceiver_ where #self_ty: crate::gameplay::ecs::traits::ecs_event_reciever::EventReciever<#event_type> { }
        // struct _EnforceSystem where #self_ty: crate::system::system_components::gameplay_components::gameplay_component_default::ECSSystemEventless { }
    };

    TokenStream::from(expanded)
}
