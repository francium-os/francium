use quote::{format_ident, quote};
use serde_derive::Deserialize;
use std::env;
use std::fs;
use std::path::Path;
use syn::{Path as SynPath, Type};

#[derive(Debug, Deserialize)]
struct Ty {
    name: String,
    ty: String,
}

#[derive(Debug, Deserialize)]
struct Method {
    name: String,
    id: u32,
    inputs: Vec<Ty>,
    output: String,
    is_async: Option<bool>,
}

impl Method {
    fn server(&self) -> syn::__private::TokenStream2 {
        let input_names: Vec<_> = self
            .inputs
            .iter()
            .map(|x| format_ident!("{}", x.name))
            .collect();
        let inputs: Vec<_> = self
            .inputs
            .iter()
            .map(|x| {
                let name = format_ident!("{}", x.name);
                let ty_: Type = syn::parse_str(&x.ty).unwrap();
                quote!(#name: #ty_)
            })
            .collect();

        let method_name = format_ident!("{}", self.name);
        let method_id: u32 = self.id;

        let output_type: Type = syn::parse_str(&self.output).unwrap();

        let maybe_await = if self.is_async.unwrap_or(false) {
            quote!(.await)
        } else {
            quote!()
        };

        let spawn_or_block = if self.is_async.unwrap_or(false) {
            quote!(tokio::spawn)
        } else {
            quote!(tokio::task::block_in_place)
        };

        let closure_or_async_block = if self.is_async.unwrap_or(false) {
            quote!(async move)
        } else {
            quote!(||)
        };

        quote! {
            #method_id => {
                request_msg.read_translates();

                #(let #inputs = request_msg.read();)*

                #spawn_or_block(#closure_or_async_block {
                    let res: #output_type = self.#method_name (server, #(#input_names),*) #maybe_await;
                    let mut reply_msg = unsafe { process::ipc::message::IPCMessage::new(&mut IPC_BUFFER) };
                    reply_msg.write(res);
                    reply_msg.write_translates();
                    reply_msg.write_header_for(0);

                    unsafe { crate::syscalls::ipc_reply(h, &mut IPC_BUFFER).unwrap(); }
                });
            }
        }
    }

    fn client(&self, handle_accessor: &str) -> syn::__private::TokenStream2 {
        let ipc_handle_accessor: SynPath = syn::parse_str(handle_accessor).unwrap();

        let input_names: Vec<_> = self
            .inputs
            .iter()
            .map(|x| format_ident!("{}", x.name))
            .collect();
        let inputs: Vec<_> = self
            .inputs
            .iter()
            .map(|x| {
                let name = format_ident!("{}", x.name);
                let ty_: Type = syn::parse_str(&x.ty).unwrap();
                quote!(#name: #ty_)
            })
            .collect();

        let output_type: Type = syn::parse_str(&self.output).unwrap();
        let dispatch_output = if self.output == "()" {
            quote! {}
        } else {
            quote! {
                let out: #output_type = reply_msg.read();
                out
            }
        };

        let write_inputs = if input_names.len() == 0 {
            quote! {}
        } else {
            quote! { #(request_msg.write(#input_names));*; }
        };

        let method_name = format_ident!("{}", self.name);
        let method_id: u32 = self.id;

        quote! {
            pub fn #method_name ( #(#inputs),* ) -> #output_type {
                let h = #ipc_handle_accessor();
                let mut request_msg = unsafe { crate::ipc::message::IPCMessage::new(&mut IPC_BUFFER) };

                #write_inputs

                request_msg.write_header_for(#method_id);
                request_msg.write_translates();

                unsafe { crate::syscalls::ipc_request(h, &mut IPC_BUFFER).unwrap(); }

                let mut reply_msg = unsafe { crate::ipc::message::IPCMessage::new(&mut IPC_BUFFER) };
                reply_msg.read_header();
                reply_msg.read_translates();

                #dispatch_output
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    name: String,
    struct_name: String,
    #[serde(default)]
    lifetimes: String,
    handle_accessor: String,
    main_interface: Interface,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    sub_interfaces: Vec<Interface>,
}

#[derive(Debug, Deserialize)]
struct Interface {
    name: Option<String>,
    session_name: String,
    #[serde(default)]
    lifetimes: String,
    methods: Vec<Method>,
}

fn generate_server_ipcserver_impl(server: &ServerConfig) -> String {
    let server_struct_name = format_ident!("{}", server.struct_name);
    let server_lifetimes = syn::parse_str::<syn::Generics>(&server.lifetimes).unwrap();

    let mut all_subinterfaces: Vec<&Interface> = Vec::new();
    all_subinterfaces.push(&server.main_interface);
    all_subinterfaces.extend(server.sub_interfaces.iter());

    let main_interface_ident = format_ident!("{}", server.main_interface.session_name);

    let mut all_subinterface_idents: Vec<_> = all_subinterfaces
        .iter()
        .map(|x| format_ident!("{}", x.session_name))
        .collect();

    let mut all_subinterface_lifetimes: Vec<_> = all_subinterfaces
        .iter()
        .map(|x| syn::parse_str::<syn::Generics>(&x.lifetimes).unwrap())
        .collect(); 

    let server_impl = quote!(
        impl<'a> IPCServer<'a> for #server_struct_name #server_lifetimes {
            // etc
            fn get_server_impl<'m, 'r>(self: &'r Arc<Self>) -> MutexGuard<'m, ServerImpl<'a, Self>> where 'r: 'm {
                self.__server_impl.lock().unwrap()
            }

            fn accept_main_session_in_trait(self: &Arc<Self>) -> Box<dyn IPCSession<'a, Server = Self>> {
                self.accept_main_session()
            }

            fn process_server(self: Arc<Self>, h: process::Handle, ipc_buffer: &mut [u8]) {
                let server_impl = self.get_server_impl();
                let session = &server_impl.sessions[&h];
                drop(server_impl);

                session.process_session(self, h, ipc_buffer);
            }
        }
    );
    server_impl.to_string()
}

fn generate_server_interface(server: &ServerConfig, interface: &Interface) -> String {
    let server_struct_name = format_ident!("{}", server.struct_name);
    let server_methods: Vec<_> = interface.methods.iter().map(|x| x.server()).collect();
    let session_name = format_ident!("{}", interface.session_name);

    let session_lifetime = syn::parse_str::<syn::Generics>(&interface.lifetimes).unwrap();
    let lifetime = if interface.lifetimes == "" {
        syn::parse_str::<syn::Generics>(&server.lifetimes).unwrap()
    } else {
        session_lifetime.clone()
    };

    let server_impl = quote!(
        impl #lifetime IPCSession<'a> for #session_name #session_lifetime {
            type Server = #server_struct_name #lifetime;
            fn process_session(&self, server: Arc<Self::Server>, h: process::Handle, ipc_buffer: &mut [u8]) {
                let mut request_msg = process::ipc::message::IPCMessage::new(ipc_buffer);
                request_msg.read_header();

                match request_msg.header.id {
                    #(#server_methods),*,
                    _ => { panic!("Unexpected IPC message ID!") }
                }
            }
        }
    );
    server_impl.to_string()
}

pub fn generate_server(path: &str) {
    let spec = toml::from_str::<ServerConfig>(&fs::read_to_string(path).unwrap()).unwrap();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(spec.name.clone() + "_server_impl.rs");

    let header = "use process::ipc::message::IPC_BUFFER;\n
    use process::ipc_server::*;
    use std::sync::MutexGuard;"
        .to_string();

    let server_impl = generate_server_ipcserver_impl(&spec);
    let server_main_impl = generate_server_interface(&spec, &spec.main_interface);
    let server_sub_impl = spec
        .sub_interfaces
        .iter()
        .map(|x| generate_server_interface(&spec, x))
        .collect::<Vec<String>>()
        .join("\n");
    fs::write(
        &dest_path,
        header + &server_impl + &server_main_impl + &server_sub_impl,
    )
    .unwrap();

    // lol. lmao
    std::process::Command::new("rustfmt").arg("--edition").arg("2021").arg(&dest_path).output().unwrap();

    println!("cargo:rerun-if-changed={}", path);
}

pub fn generate_client(path: &str) {
    let spec = toml::from_str::<ServerConfig>(&fs::read_to_string(path).unwrap()).unwrap();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(spec.name + "_client_impl.rs");

    let client_methods: Vec<_> = spec
        .main_interface
        .methods
        .iter()
        .map(|x| x.client(&spec.handle_accessor))
        .collect();

    let client_impl = quote! {
        use crate::ipc::message::IPC_BUFFER;

        #(#client_methods)*
    };

    fs::write(&dest_path, client_impl.to_string()).unwrap();
    
    // lol. lmao
    std::process::Command::new("rustfmt").arg("--edition").arg("2021").arg(&dest_path).output().unwrap();

    println!("cargo:rerun-if-changed={}", path);
}
