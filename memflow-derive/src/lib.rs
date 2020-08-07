use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, Data, DeriveInput, Fields, GenericParam, ItemFn};

#[derive(Debug, FromMeta)]
struct ConnectorFactoryArgs {
    name: String,
    #[darling(default)]
    version: Option<String>,
}

// We should add conditional compilation for the crate-type here
// so our rust libraries who use a connector wont export those functions
// again by themselves (e.g. the ffi).
//
// This would also lead to possible duplicated symbols if
// multiple connectors are imported.
//
// See https://github.com/rust-lang/rust/issues/20267 for the tracking issue.
//
// #[cfg(crate_type = "cdylib")]
#[proc_macro_attribute]
pub fn connector(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);
    let args = match ConnectorFactoryArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let connector_name = args.name;

    let func = parse_macro_input!(input as ItemFn);
    let func_name = &func.sig.ident;

    let gen = quote! {
        #[cfg(feature = "plugin")]
        #[doc(hidden)]
        pub static CONNECTOR_NAME: &str = #connector_name;

        #[cfg(feature = "plugin")]
        #[doc(hidden)]
        #[no_mangle]
        pub static MEMFLOW_CONNECTOR: memflow_core::connector::ConnectorDescriptor = memflow_core::connector::ConnectorDescriptor {
            connector_version: memflow_core::connector::MEMFLOW_CONNECTOR_VERSION,
            name: CONNECTOR_NAME,
            factory: connector_factory,
        };

        #[cfg(feature = "plugin")]
        pub extern "C" fn connector_factory(args: &memflow_core::connector::ConnectorArgs) -> memflow_core::error::Result<memflow_core::connector::PluginConnector> {
            let connector = #func_name(args)?;
            Ok(Box::new(connector))
        }

        pub fn static_connector_factory(args: &memflow_core::connector::ConnectorArgs) -> memflow_core::error::Result<impl memflow_core::mem::PhysicalMemory> {
            #func_name(args)
        }

        #func
    };
    gen.into()
}

#[proc_macro_derive(ByteSwap)]
pub fn byteswap_derive(input: TokenStream) -> TokenStream {
    // TODO: parse struct fields
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics.params;

    let mut impl_generics = quote!();
    let mut type_generics = quote!();
    let mut where_generics = quote!();
    for param in generics.iter() {
        match param {
            GenericParam::Type(ty) => {
                let id = &ty.ident;
                impl_generics.extend(quote!(#ty));
                type_generics.extend(quote!(#id));
                where_generics.extend(quote!(#id: ByteSwap));
            }
            GenericParam::Lifetime(lt) => {
                impl_generics.extend(quote!(#lt));
                type_generics.extend(quote!(#lt));
            }
            GenericParam::Const(_cnst) => {}
        }
    }

    let mut gen_inner = quote!();
    match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(named) => {
                for field in named.named.iter() {
                    let name = field.ident.as_ref().unwrap();
                    gen_inner.extend(quote!(
                        self.#name.byte_swap();
                    ));
                }
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    let gen = quote!(
        impl<#impl_generics> ByteSwap for #name<#type_generics> where #where_generics {
            fn byte_swap(&mut self) {
                #gen_inner
            }
        }
    );

    gen.into()
}