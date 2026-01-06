use proc_macro::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use syn::parse_macro_input;
use syn::parse_quote;
use syn::ItemStruct;

#[proc_macro_attribute]
pub fn global_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    let register_fn = format_ident!("global_state_{}", name);

    // input
    //     .attrs
    //     .push(parse_quote!(#[derive(serde::Deserialize)]));
    let expanded = quote! {
        #input

        unsafe impl Send for #name {}
        unsafe impl Sync for #name {}

        #[ctor::ctor]
        #[allow(non_snake_case)]
        fn #register_fn() {
            system_component_default_gameplay::static_data::global_components::register_global_component::<#name>();
        }
    };

    TokenStream::from(expanded)
}
// use proc_macro::TokenStream;
// use quote::quote;
// use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, Meta};

// #[proc_macro_derive(PrefabComponent, attributes(prefab))]
// pub fn derive_prefab_component(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as DeriveInput);
//     let name = input.ident;

//     let fields = match input.data {
//         Data::Struct(data) => data.fields,
//         _ => panic!("PrefabComponent can only be derived for structs"),
//     };

//     let mut override_match_arms = Vec::new();

//     if let Fields::Named(fields) = fields {
//         for field in fields.named {
//             let ident = field.ident.unwrap();
//             let field_name = ident.to_string();

//             if is_overridable(&field.attrs) {
//                 override_match_arms.push(quote! {
//                     #field_name => {
//                         self.#ident = value.parse()
//                             .expect(concat!(
//                                 "Failed to parse prefab override for field: ",
//                                 #field_name
//                             ));
//                     }
//                 });
//             }
//         }
//     } else {
//         panic!("PrefabComponent requires named fields");
//     }

//     let expanded = quote! {
//         impl ::core::PrefabOverridable for #name {
//             fn apply_override(&mut self, key: &str, value: &str) {
//                 match key {
//                     #(#override_match_arms,)*
//                     _ => {}
//                 }
//             }
//         }
//     };

//     TokenStream::from(expanded)
// }

// fn is_overridable(attrs: &[Attribute]) -> bool {
//     attrs.iter().any(|attr| {
//         if !attr.path().is_ident("prefab") {
//             return false;
//         }

//         matches!(&attr.meta, Meta::List(list) if list.tokens.to_string().contains("overridable"))
//     })
// }
