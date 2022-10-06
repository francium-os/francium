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
        if let TraitItem::Method(method) = item {
            let mut method_without_attrs = method.clone();
            method_without_attrs.attrs.clear();
            let sig = &method_without_attrs.sig;

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

            let inputs_with_handles: Vec<_> = sig.inputs.iter().filter_map(|x| if let syn::FnArg::Typed(arg) = x { Some(arg) } else { None }).collect();
            let input_names_with_handles: Vec<_> = inputs_with_handles.iter().map(|x| x.pat.clone()).collect();

            let mut handles: Vec<_> = Vec::new();
            let inputs: Vec<_> = inputs_with_handles.iter().filter_map(|inp| {
                // ..
                match &*inp.ty {
                    syn::Type::Path(path) => {
                        if path.path.segments[0].ident.to_string() == "TranslateHandle" {
                            handles.push(inp);
                            None
                        } else {
                            Some(inp)
                        }
                    },
                    _ => { 
                        println!("{:?}", inp);
                        Some(inp)
                    }
                }
            }).collect();

            //let input_types: Vec<_> = inputs.iter().map(|x| x.ty.clone()).collect();
            let handle_names: Vec<_> = handles.iter().map(|x| x.pat.clone()).collect();
            let input_names: Vec<_> = inputs.iter().map(|x| x.pat.clone()).collect();

            let output = &sig.output;

            let dispatch_output = match &sig.output {
                ReturnType::Default => quote!(),
                ReturnType::Type(_, ty) => {
                    // if translate type
                    quote! {
                        let out: #ty = msg.read();
                        out
                    }
                }
            };

            let method_name = &sig.ident;
            server_dispatch.push(quote! {
                #method_id => {
                    #(let #inputs = msg.read();)*
                    #(msg.read::<u32>(); let #handles = msg.read();)*

                    let res = self.#method_name (#(#input_names_with_handles),*);
                    msg.write(res);
                    crate::syscalls::ipc_reply(h).unwrap();
                }
            });

            // this also calls write() for everything
            let msg_length_maybe = if input_names.len() == 0 {
                quote!{ let msg_length: usize = 0; }
            } else {
                quote! { #(msg.write(#input_names));*; let msg_length: usize = msg.write_offset; }
            };

            let translate_count: usize = handles.len();

            client_methods.push(quote! {
                pub fn #method_name ( #(#inputs_with_handles),* ) #output {
                    let h = #ipc_handle_accessor();
                    let mut msg: crate::ipc::message::IPCMessage = crate::ipc::message::IPCMessage::new();
                    let mut header = crate::ipc::message::IPCHeader { id: #method_id, size: 0, translate_count: #translate_count };

                    // Write normal parameters and record their length
                    #msg_length_maybe
                    // Write translate handles
                    #(msg.write(#handle_names.0.0));* ;

                    header.size = msg_length;
                    assert!(header.size % 4 == 0);
                    msg.write_header(&header);

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
            #server_dispatch_method

            #(#server_methods)*
        }
            

        #(#client_methods)*
    };

    out.into()
}