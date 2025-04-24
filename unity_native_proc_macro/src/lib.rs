use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn unity_plugin_load(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let fn_ident = input.sig.ident.clone();

    let loader = quote! {
        #input

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        extern "stdcall" fn UnityPluginLoad(
            interfaces: *mut unity_native::RawUnityInterfaces,
        ) {
            let interfaces = unsafe { unity_native::UnityInterfaces::new(interfaces).unwrap() };

            #fn_ident(interfaces);
        }
    };

    TokenStream::from(loader)
}

#[proc_macro_attribute]
pub fn unity_plugin_unload(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let fn_ident = input.sig.ident.clone();

    let loader = quote! {
        #input

        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        extern "stdcall" fn UnityPluginUnload() {
            #fn_ident();
        }
    };

    TokenStream::from(loader)
}
