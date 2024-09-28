use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Field, Fields, parse_macro_input, punctuated::Iter};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;
    let builder_name = Ident::new(&format!("{}Builder", struct_name), Span::call_site());
    let setters = builder_setter(&input.data);

    let expanded = quote! {
        pub struct #builder_name {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }

        impl Command {
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
        }

    };
    proc_macro::TokenStream::from(expanded)
}

fn extract_fields(data: &Data) -> Iter<Field> {
   match data {
       Data::Struct(structure) => match &structure.fields {
           Fields::Named(fields_named) => {
               fields_named.named.iter()
           }
           _ => unimplemented!()
       }
       _ => {
           unimplemented!()
       }
   }
}

fn builder_setter(data: &Data) -> TokenStream {
    let fields = extract_fields(data);
    let setters = fields.map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;

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
