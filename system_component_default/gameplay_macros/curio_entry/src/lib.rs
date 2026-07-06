// use curio_core::built_in::record::sys_record_debug::SysRecordDebug;
// use curio_core::built_in::record::sys_record_debug_gui::SysRecordDebugGui;
// use curio_core::built_in::record::sys_record_gui::SysRecordGui;
// use curio_core::built_in::record::sys_record_input::SysRecordInput;
// use curio_core::built_in::record::sys_record_network::SysRecordNetwork;
// use curio_core::built_in::record::sys_record_screen::SysRecordScreen;
// use curio_core::built_in::record::sys_record_time::SysRecordTime;
// use curio_core::GlobalRecords;
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
            services: *const curio_core::Services,
        ) -> *mut Curio {
            curio_core::Services::set(services);

            register_built_in_ecs();
            // register_built_in_records();
            register_built_in_component();

            let curio = #fn_name();

            Box::into_raw(Box::new(curio))
        }
    };

    TokenStream::from(expanded)
}
