#![feature(proc_macro_quote)]

use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemTrait, TraitItem, Meta, Lit, ReturnType};

#[proc_macro_attribute]
pub fn ipc_server(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ipc_handle_accessor = format_ident!("{}", attr.to_string());

    let input = parse_macro_input!(item as ItemTrait);

    let mut server_methods: Vec<syn::TraitItemMethod> = Vec::new();
    let mut client_methods: Vec<_> = Vec::new();
    let mut server_dispatch: Vec<_> = Vec::new();

    let server_trait_name = input.ident.clone();

    for item in input.items {
        if let TraitItem::Method(method) = item  {
            let mut method_without_attrs = method.clone();
            method_without_attrs.attrs.clear();
            let sig = &method_without_attrs.sig;

            let inputs: Vec<_> = sig.inputs.iter().filter_map(|x| if let syn::FnArg::Typed(arg) = x { Some(arg) } else { None }).collect();
            let input_names: Vec<_> = inputs.iter().map(|x| x.pat.clone()).collect();
            let output = &sig.output;

            let dispatch_output = match &sig.output {
                ReturnType::Default => quote!(""),
                ReturnType::Type(_, ty) => quote! {
                    let out: #ty = msg.read();
                    out
                }
            };

            let mut method_id: u32 = 0;
            //let mut copy_handles = [];

            for attr in method.attrs {
                let meta = attr.parse_meta().unwrap();
                // Either Path or List
                match meta {
                    Meta::Path(_) => panic!("got a path?"),
                    Meta::List(list) => {
                        if list.path.segments[0].ident.to_string() == "copy_handles" {
                            panic!("Got copy handles");
                        }
                    },
                    Meta::NameValue(nv) => {
                        if nv.path.segments[0].ident.to_string() == "ipc_method_id" {
                            // Rejoice
                            if let Lit::Int(int_value) = nv.lit {
                                method_id = int_value.base10_parse().unwrap();
                            } else {
                                panic!("angy");
                            }
                        }
                    }
                }
            }

            let method_name = &sig.ident;
            server_dispatch.push(quote! {
                #method_id => {
                    #(let #inputs = msg.read();)*
                    let res = self.#method_name (#(#input_names),*);
                    msg.write(res);
                    crate::syscalls::ipc_reply(h).unwrap();
                }
            });

            client_methods.push(quote! {
                pub fn #method_name ( #(#inputs),* ) #output {
                    let h = #ipc_handle_accessor();
                    let mut msg: crate::ipc::message::IPCMessage = crate::ipc::message::IPCMessage::new();
                    let header = crate::ipc::message::IPCHeader { id: #method_id };
                    msg.write_header(&header);
                    #(msg.write(#input_names);)*
                    crate::syscalls::ipc_request(h).unwrap();
                    #dispatch_output
                }
            });

            server_methods.push(method_without_attrs);
        }
    }

    let server_dispatch_method = quote! {
        fn process(&self, h: Handle) {
            let mut msg: crate::ipc::message::IPCMessage = crate::ipc::message::IPCMessage::new();
            let message_header = msg.read_header();

            match message_header.id {
                #(#server_dispatch),*
                _ => { panic!("Unexpected IPC message ID!") }
            }
        }
    };

    let out = quote! {
        pub trait #server_trait_name { 
            #(#server_methods);*
        }

        impl crate::ipc_server::IPCServer for alloc::boxed::Box<dyn #server_trait_name> {
            #server_dispatch_method
        }

        #(#client_methods)*
    };

    out.into()
}
