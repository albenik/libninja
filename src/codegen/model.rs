use convert_case::Case;
use convert_case::Casing;
use openapiv3::{OpenAPI, ReferenceOr, Schema};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};

use crate::codegen::util::ToIdent;
use crate::codegen::util::ToToken;

pub fn generate_model_rs(spec: &OpenAPI) -> TokenStream {
    let all_struct_Schema = all_struct_Schema(spec);

    quote! {
        #all_struct_Schema
    }
}


pub fn all_struct_Schema(spec: &OpenAPI) -> TokenStream {
    let schemas = spec.components.as_ref().unwrap().schemas.iter().map(|(k, schema)| {
        let schema = schema
            .as_ref()
            .resolve(spec).unwrap();
        struct_Schema(k, schema, spec)
    });
    quote! {
        #(#schemas)*
    }
}

pub fn struct_Schema_object(name: &str, struct_schema: &Schema, spec: &OpenAPI) -> TokenStream {
    // if name != "Item" {
    //     return quote! {};
    // }

    let fields = struct_schema.properties().unwrap().iter().map(|(k, v)| {
        let mut k = k.to_string();
        let prop_schema = v
            .as_ref()
            .resolve(spec)
            .unwrap();

        let mut field_type = match v {
            ReferenceOr::Reference { ref reference } => {
                let name = reference.rsplit('/').next().unwrap();
                let field_type = syn::Ident::new(name, Span::call_site()).to_token_stream();
                if prop_schema.schema_data.nullable {
                    quote! { Option<#field_type> }
                } else {
                    field_type
                }
            }
            ReferenceOr::Item(schema) => schema.to_token(spec),
        };
        // if prop_schema.schema_data.nullable {
        //     println!("cargo:warning={}.{} nullable", name, k);
        // }
        // if name == "Item" && k == "error" {
        //     println!("cargo:warning={}.{}, requiredness struct={} prop={:?}",
        //              name, k,
        //              struct_schema.required(&k),
        //              prop_schema.schema_data
        //     );
        // }
        // if prop_schema.schema_data.nullable {
        //     field_type = quote! { Option<#field_type>};
        // }
        let serde = if k.is_restricted() {
            let serde_line = quote! {
                #[serde(rename = #k)]
            };
            k += "_";
            serde_line
        } else {
            TokenStream::new()
        };
        let z = "".to_string();
        let docstring = prop_schema.schema_data.description.as_ref().unwrap_or(&z);
        // let k = k.to_case(Case::Snake);
        let field = syn::Ident::new(&k, Span::call_site());
        quote! {
            #serde
            #[doc = #docstring]
            pub #field: #field_type,
        }
    });
    let name = syn::Ident::new(name, Span::call_site());
    quote! {
        #[derive(Debug, Serialize, Deserialize)]
        pub struct #name {
            #(#fields)*
        }
    }
}

pub fn struct_Schema_newtype(name: &str, schema: &Schema, spec: &OpenAPI) -> TokenStream {
    let field = syn::Ident::new("value", Span::call_site());
    let field_type = schema.to_token(spec);
    let name = syn::Ident::new(name, Span::call_site());
    quote! {
        #[derive(Debug, Serialize, Deserialize)]
        pub struct #name(pub #field_type);
    }
}

pub fn struct_Schema(name: &str, schema: &Schema, spec: &OpenAPI) -> TokenStream {
    match schema.properties() {
        Some(properties) => struct_Schema_object(name, schema, spec),
        None => struct_Schema_newtype(name, schema, spec),
    }
}
