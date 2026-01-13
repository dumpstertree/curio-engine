// use proc_macro::TokenStream;
// use quote::{format_ident, quote};
// use serde::Deserialize;
// use syn::parse_macro_input;
// use syn::parse_quote;
// use syn::ItemStruct;

// // #[proc_macro_attribute]
// // pub fn global_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
// //     let mut input = parse_macro_input!(item as ItemStruct);
// //     let name = &input.ident;

// //     let register_fn = format_ident!("global_state_{}", name);

// //     // input
// //     //     .attrs
// //     //     .push(parse_quote!(#[derive(serde::Deserialize)]));
// //     let expanded = quote! {
// //         #input

// //         unsafe impl Send for #name {}
// //         unsafe impl Sync for #name {}

// //         #[ctor::ctor]
// //         #[allow(non_snake_case)]
// //         fn #register_fn() {
// //             system_component_default_gameplay::static_data::global_components::register_global_component::<#name>();
// //         }

// //     };

// //     TokenStream::from(expanded)
// // }
// #[proc_macro_attribute]
// pub fn global_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
//     let mut input = parse_macro_input!(item as syn::ItemStruct);
//     let name = &input.ident;

//     let register_fn = format_ident!("global_state_{}", name);

//     // Inject `owner: Option<Form>` field
//     match &mut input.fields {
//         syn::Fields::Named(fields) => {
//             fields.named.push(syn::parse_quote! {
//                 owner: Option<system_component_default_gameplay::form::Form>
//             });
//         }
//         _ => {
//             return syn::Error::new_spanned(input, "global_component only supports structs with named fields")
//                 .to_compile_error()
//                 .into();
//         }
//     }

//     let expanded = quote! {
//         #input

//         unsafe impl Send for #name {}
//         unsafe impl Sync for #name {}

//         impl system_component_default_gameplay::form::FacetCommon for #name {
//             fn set_ownership(&mut self, owner: system_component_default_gameplay::form::Form) {
//                 self.owner = Some(owner);
//             }

//             fn form(&self) -> system_component_default_gameplay::form::Form {
//                 self.owner.clone().expect("Facet owner not set")
//             }
//         }

//         #[ctor::ctor]
//         #[allow(non_snake_case)]
//         fn #register_fn() {
//             system_component_default_gameplay::static_data::global_components
//                 ::register_global_component::<#name>();
//         }
//     };

//     TokenStream::from(expanded)
// }
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Fields, ItemStruct};

#[proc_macro_attribute]
pub fn global_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);

    let struct_name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Ensure named fields
    let fields_named = match &mut input.fields {
        Fields::Named(fields) => fields,
        _ => {
            return syn::Error::new_spanned(&input.ident, "#[global_component] can only be applied to structs with named fields")
                .to_compile_error()
                .into();
        }
    };

    // Inject owner field FIRST
    fields_named.named.push(syn::parse_quote! {
        owner: Option<system_component_default_gameplay::form::Form>
    });

    let fields = &fields_named.named;

    // Clone impl
    let clone_fields = fields.iter().map(|field| {
        let name = field.ident.as_ref().unwrap();
        quote! { #name: self.#name.clone() }
    });

    let clone_impl = quote! {
        impl #impl_generics Clone for #struct_name #ty_generics #where_clause {
            fn clone(&self) -> Self {
                Self {
                    #(#clone_fields),*
                }
            }
        }
    };

    // Default impl
    let default_fields = fields.iter().map(|field| {
        let name = field.ident.as_ref().unwrap();

        if name.to_string() == "owner" {
            quote! { #name: None }
        } else {
            quote! { #name: Default::default() }
        }
    });

    let default_impl = quote! {
        impl #impl_generics Default for #struct_name #ty_generics #where_clause {
            fn default() -> Self {
                Self {
                    #(#default_fields),*
                }
            }
        }
    };

    let register_fn = format_ident!("global_state_{}", struct_name);

    let expanded = quote! {
        #input

        unsafe impl #impl_generics Send for #struct_name #ty_generics #where_clause {}
        unsafe impl #impl_generics Sync for #struct_name #ty_generics #where_clause {}

        impl #impl_generics system_component_default_gameplay::form::FacetCommon
            for #struct_name #ty_generics #where_clause
        {
            fn set_ownership(
                &mut self,
                owner: system_component_default_gameplay::form::Form,
            ) {
                self.owner = Some(owner);
            }

            fn form(&self) -> system_component_default_gameplay::form::Form {
                self.owner.clone().expect("Facet owner not set")
            }
        }

        #[ctor::ctor]
        #[allow(non_snake_case)]
        fn #register_fn() {
            system_component_default_gameplay::static_data::global_components
                ::register_global_component::<#struct_name #ty_generics>();
        }

        #clone_impl
        #default_impl
    };

    expanded.into()
}
