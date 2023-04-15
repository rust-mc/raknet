#![crate_type = "proc-macro"]

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Fields, FieldsNamed};

#[proc_macro_derive(ServerPacket)]
pub fn server_packet(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let output: proc_macro2::TokenStream = match data {
        Data::Struct(s) => match s.fields {
            Fields::Named(FieldsNamed { named, .. }) => {
                let expr = named.iter().map(|f| {
                    let f_name = &f.ident;
                    let ty = f.ty.to_token_stream().to_string();
                    match ty.as_str() {
                        "String" => quote! {
                            writer.write_u16(self.#f_name.len() as u16)?;
                            writer.write(&self.#f_name.as_bytes())?;
                        },
                        "async_std :: net :: SocketAddr" => quote! {
                            use async_std::net::SocketAddr;
                            let version = if self.#f_name.is_ipv4() { 4u8 } else { 6u8 };
                            let ip = self.#f_name.ip().to_string().split(".").map(|x| {
                                x.parse::<u8>().unwrap()
                            }).collect::<Vec<_>>();
                            let port = self.#f_name.port();
                            writer.write_u8(version)?;
                            writer.write(&ip)?;
                            writer.write_u16(port)?;
                        },
                        "[u8 ; 16]" => quote!(writer.write(&crate::protocol::MAGIC)?;),
                        "bool" => quote!(writer.write_u8(u8::from(self.#f_name.clone()))?;),
                        "u8" => quote! {
                            writer.write_u8(self.#f_name)?;
                        },
                        "i8" => quote! {
                            writer.write_i8(self.#f_name)?;
                        },
                        "u16" => quote! {
                            writer.write_u16(self.#f_name)?;
                        },
                        "i16" => quote! {
                            writer.write_i16(self.#f_name)?;
                        },
                        "raknet :: internal :: u24" => quote! {
                            writer.write(self.#f_name.to_le_bytes())?;
                        },
                        "u32" => quote! {
                            writer.write_u32(self.#f_name)?;
                        },
                        "i32" => quote! {
                            writer.write_i32(self.#f_name)?;
                        },
                        "u64" => quote! {
                            writer.write_u64(self.#f_name)?;
                        },
                        "i64" => quote! {
                            writer.write_i64(self.#f_name)?;
                        },
                        ty => panic!("Incompatible types: {}", ty),
                    }
                });

                let new_expr_args = named.iter().filter_map(|f| {
                    let f_name = &f.ident;
                    let ty = &f.ty;
                    let sty = ty.to_token_stream().to_string();
                    let sty = sty.as_str();
                    match sty {
                        "[u8 ; 16]" => None,
                        _ => Some(quote!(#f_name: #ty)),
                    }
                });

                let new_expr = named.iter().map(|f| {
                    let f_name = &f.ident;
                    let ty = f.ty.to_token_stream().to_string();
                    let ty = ty.as_str();
                    match ty {
                        "[u8 ; 16]" => quote!(#f_name: [0u8 ; 16],),
                        _ => quote!(#f_name: #f_name,),
                    }
                });

                let doc_id = format!(
                    "Returned [`crate::protocol::PacketID::{}`] as `u8`",
                    ident.to_string()
                );

                quote! {
                    impl #ident {

                        pub fn new(#(#new_expr_args),*) -> Self {
                            #ident {
                                #(#new_expr)
                                *
                            }
                        }
                    }

                    impl crate::protocol::ServerPacket for #ident {

                        #[doc = #doc_id]
                        fn id() -> u8 {
                            crate::protocol::PacketID::#ident as u8
                        }

                        #[doc = "Collects all the fields of the structure by bytes into a container."]
                        #[doc = "# Failures"]
                        #[doc = "See [`byte_order::NumberWriter`] by more info."]
                        fn compose(&self) -> std::io::Result<byte_order::NumberReader<std::io::Cursor<Vec<u8>>>> {
                            use std::io::{Cursor, Write};
                            use byte_order::{NumberReader, NumberWriter, ByteOrder};
                            let cursor = Cursor::new(vec![]);
                            let mut writer = NumberWriter::with_order(ByteOrder::BE, cursor);
                            writer.write_u8(Self::id())?;
                            #(#expr)
                            *
                            Ok(NumberReader::with_order(ByteOrder::BE, writer.into_inner()))
                        }
                    }
                }
            }
            _ => panic!("#[derive(ServerPacket)] can only be used with structs"),
        },
        _ => panic!("#[derive(ServerPacket)] can only be used with structs"),
    };

    output.into()
}

#[proc_macro_derive(ClientPacket)]
pub fn client_packet(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let output: proc_macro2::TokenStream = match data {
        Data::Struct(s) => match s.fields {
            Fields::Named(FieldsNamed { named, .. }) => {
                let expr = named.iter().map(|f| {
					let f_name = &f.ident;
					let ty = f.ty.to_token_stream().to_string();
					let ty = ty.as_str();
					match ty {
						"String" => quote! {
							#f_name: {
								use std::io::Read;
								let len = reader.read_u16()?;
								let mut temp = String::new();
								let mut handle = reader.take(len as u64);
								let bytes = handle.read_to_string(&mut temp)?;
								assert_eq!(len as usize, bytes,
									"Parsing the string failed. \
									source len: {}; output len: {}; string: {}", len, bytes, temp);
								reader = handle.into_inner();
								temp
							},
						},
						"async_std :: net :: SocketAddr" => quote! {
	                        #f_name: {
								use async_std::net::SocketAddr;
		                        let version = reader.read_u8()?;
								let ip = reader.read_u32()?;
								let ip = ip.to_be_bytes();
								let port = reader.read_u16()?;
								format!("{}.{}.{}.{}:{}", ip[0], ip[1], ip[2], ip[3], port).parse().unwrap()
	                        },
	                    },
						"[u8 ; 16]" => quote! {
							#f_name: {
								use std::io::Read;
								let mut buf = [0u8; 16];
								reader.read_exact(&mut buf)?;
								buf
							},
						},
						"MTU" => quote! {
							#f_name: {
								let size = reader.into_inner().into_inner().len();
								MTU {0: size as u16}
							},
						},
						"bool" => quote! {
							#f_name: if reader.read_u8()? == 1 {true} else {false},
						},
						"u8" => quote! {
							#f_name: reader.read_u8()?,
						},
						"i8" => quote! {
							#f_name: reader.read_i8()?,
						},
						"u16" => quote! {
							#f_name: reader.read_u16()?,
						},
						"i16" => quote! {
							#f_name: reader.read_i16()?,
						},
						"raknet :: internal :: u24" => quote! {
							#f_name: {
								let buf = [0u8; 3];
								reader.read_exact(&mut buf)?;
								u24::from_le_bytes(buf)
							}
						},
						"u32" => quote! {
							#f_name: reader.read_u32()?,
						},
						"i32" => quote! {
							#f_name: reader.read_i32()?,
						},
						"u64" => quote! {
							#f_name: reader.read_u64()?,
						},
						"i64" => quote! {
							#f_name: reader.read_i64()?,
						},
						ty => panic!("Incompatible types: {}", ty)
					}
				});

                let doc_id = format!(
                    "Returned [`crate::protocol::PacketID::{}`] as `u8`",
                    ident.to_string()
                );

                quote! {
                    impl crate::protocol::ClientPacket for #ident {

                        #[doc = #doc_id]
                        fn id() -> u8 {
                            crate::protocol::PacketID::#ident as u8
                        }

                        #[doc = "Parsing an array of bytes into structure fields."]
                        #[doc = "# Failures"]
                        #[doc = "See [`byte_order::NumberReader`] by more info."]
                        fn parse(mut reader: byte_order::NumberReader<std::io::Cursor<Vec<u8>>>) -> std::io::Result<Self> {
                            use std::io::Cursor;
                            use byte_order::{NumberReader, ByteOrder};
                            Ok(#ident {
                                #(#expr)
                                *
                            })
                        }
                    }
                }
            }
            _ => panic!("#[derive(ClientPacket)] can only be used with structs"),
        },
        _ => panic!("#[derive(ClientPacket)] can only be used with structs"),
    };

    output.into()
}
