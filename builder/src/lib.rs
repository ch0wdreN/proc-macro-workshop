use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, FieldsNamed, Path, PathSegment, Type, TypePath,
};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;
    let builder_name = Ident::new(&format!("{}Builder", &struct_name), Span::call_site());
    let setters = builder_setter(&input.data);
    let build = builder_build(&input.data, &struct_name);

    let expanded = quote! {
        pub struct #builder_name {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }

        impl #struct_name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None
                }
            }
        }

        impl #builder_name {
            #setters

            #build
        }

    };
    proc_macro::TokenStream::from(expanded)
}

fn extract_fields(data: &Data) -> &FieldsNamed {
    match data {
        Data::Struct(structure) => match &structure.fields {
            Fields::Named(fields_named) => fields_named,
            _ => unimplemented!(),
        },
        _ => {
            unimplemented!()
        }
    }
}

fn builder_setter(data: &Data) -> TokenStream {
    let fields = extract_fields(data);
    let setters = fields.named.iter().map(|f| {
        let field_name = &f.ident;
        let field_type = &f.ty;

        quote! {
            fn #field_name(&mut self, #field_name: #field_type) -> &mut Self {
                self.#field_name = Some(#field_name);
                self
            }
        }
    });

    quote! {
        #(#setters)*
    }
}

fn builder_build(data: &Data, ident: &Ident) -> TokenStream {
    let fields = extract_fields(data);
    let fields_check = fields.named.iter().filter_map(|f| {
        let field_name = &f.ident;
        let field_type = &f.ty;

        if is_option(field_type) || is_vector(field_type) {
            return None;
        }

        let err = format!("field {} is not set", field_name.as_ref().unwrap());
        Some(quote! {
            if self.#field_name.is_none() {
                return Err(#err.into());
            }
        })
    });

    let values = fields.named.iter().map(|f| {
        let field_name = &f.ident;
        let field_type = &f.ty;

        if is_option(field_type) {
            quote! {
                #field_name: self.#field_name.clone()
            }
        } else if is_vector(field_type) {
            quote! {
                #field_name: self.#field_name.clone().unwrap_or_else(std::vec::Vec::new)
            }
        } else {
            quote! {
                #field_name: self.#field_name.clone().unwrap()
            }
        }
    });

    quote! {
        pub fn build(&mut self) -> std::result::Result<#ident, std::boxed::Box<dyn std::error::Error>> {
            #(#fields_check)*

            Ok(#ident {
                #(#values),*
            })
        }
    }
}

fn is_option(ty: &Type) -> bool {
    extract_type(ty)
        .map(|seg| seg.ident == "Option")
        .unwrap_or(false)
}

fn is_vector(ty: &Type) -> bool {
    extract_type(ty)
        .map(|seg| seg.ident == "Vec")
        .unwrap_or(false)
}

fn extract_type(ty: &Type) -> Option<&PathSegment> {
    match ty {
        Type::Path(TypePath {
            qself: _,
            path:
                Path {
                    segments: seg,
                    leading_colon: _,
                },
        }) => seg.last(),
        _ => None,
    }
}
