use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field};

/// Derive macro for RspKey trait implementation
///
/// Automatically generates from_query_args() implementation based on struct fields:
/// - Each field is extracted from query parameters by name
/// - Optional fields are handled gracefully
/// - Supports basic Rust types (i32, String, etc.)
#[proc_macro_derive(RspKey)]
pub fn derive_rsp_key(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    // Parse the struct fields
    let fields = match &input.data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => &fields.named,
                _ => panic!("RspKey can only be derived for structs with named fields"),
            }
        }
        _ => panic!("RspKey can only be derived for structs"),
    };

      // Generate the from_query_args implementation
    let from_query_args_impl = generate_from_query_args(fields, name);

    let expanded = quote! {
        impl rsp10::core::RspKey for #name {
            #from_query_args_impl
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for RspState trait implementation
///
/// Automatically generates fill_data() and related methods based on field naming conventions:
/// - txtXXX: Text input field
/// - ddXXX: Dropdown/select element
/// - cbXXX: Checkbox
/// - rbXXX: Radio button group
/// - Other: Plain data
#[proc_macro_derive(RspState, attributes(rsp_source, rsp_key, rsp_auth, rsp_template))]
pub fn derive_rsp_state(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    // Extract key and auth types from attributes
    let (key_type, auth_type) = extract_types_from_attrs(&input.attrs);

    // Parse the struct fields
    let fields = match &input.data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => &fields.named,
                _ => panic!("RspState can only be derived for structs with named fields"),
            }
        }
        _ => panic!("RspState can only be derived for structs"),
    };

    // Generate fill_data implementation
    let fill_data_impl = generate_fill_data(fields);

    // TODO: Extract these from attributes
    // For now, we'll leave them as associated types/generics

    // Generate a private "MyPageAuth" type alias
    let auth_alias_ident = syn::Ident::new("MyPageAuth", name.span());

    // Always generate as a standalone impl - this avoids conflicts with manual trait impls
    let expanded = if let (Some(key_ty), Some(auth_ty)) = (key_type, auth_type) {
        // Generate with concrete types
        quote! {
            pub type #auth_alias_ident = #auth_ty;

            // Iron handler (when iron feature is enabled)
            #[cfg(feature = "iron")]
            pub fn handler() -> impl iron::Handler {
                rsp10::make_iron_handler::<#name, #key_ty, #auth_ty>()
            }

            // Axum handler (when axum feature is enabled) - State must come first for Handler trait
            #[cfg(feature = "axum")]
            pub async fn axum_handler(
                state: axum::extract::State<std::sync::Arc<tokio::sync::Mutex<rsp10::axum_adapter::SessionData>>>,
                query: axum::extract::Query<std::collections::HashMap<String, String>>,
                form: Option<axum::extract::Form<std::collections::HashMap<String, String>>>,
            ) -> axum::response::Response {
                rsp10::axum_adapter::axum_handler_fn::<#name, #key_ty, #auth_ty>((query, form, state)).await
            }

            impl #name {
                pub fn derive_auto_fill_data_impl<'a>(
                    mut ri: rsp10::RspInfo<'a, Self, #key_ty, #auth_ty>
                ) -> rsp10::RspFillDataResult<Self> {
                    println!("DEBUG: derive_auto_fill_data_impl called for {}", stringify!(#name));
                    let mut modified = false;
                    let mut gd = rsp10::RspDataBuilder::new();
                    #fill_data_impl
                    <Self as rsp10::RspState<#key_ty, #auth_ty>>::fill_data_result(ri, gd)
                }
            }
        }
    } else {
        // Generate with generic types (fallback)
        quote! {
            impl #name {
                pub fn derive_auto_fill_data_impl<'a, T, TA>(
                    mut ri: rsp10::RspInfo<'a, Self, T, TA>
                ) -> rsp10::RspFillDataResult<Self>
                where
                    T: serde::Serialize + std::fmt::Debug + Clone + Default + serde::de::DeserializeOwned,
                    TA: rsp10::RspUserAuth + serde::Serialize,
                {
                    let mut modified = false;
                    let mut gd = rsp10::RspDataBuilder::new();
                    #fill_data_impl
                    Self::fill_data_result(ri, gd)
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn generate_fill_data(fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>) -> proc_macro2::TokenStream {
    let mut field_generations = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        // Determine field type based on prefix
        let field_gen = if field_name_str.starts_with("txt") {
            generate_text_field(field_name)
        } else if field_name_str.starts_with("dd") {
            generate_dropdown_field(field_name, field)
        } else if field_name_str.starts_with("cb") {
            generate_checkbox_field(field_name)
        } else if field_name_str.starts_with("rb") {
            generate_radio_field(field_name)
        } else if field_name_str.starts_with("btn") {
            generate_button_field(field_name)
        } else {
            // Plain data field
            continue;
        };

        field_generations.push(field_gen);
    }

    quote! {
        #(#field_generations)*
        rsp10_data!(modified => gd);
    }
}

fn generate_text_field(field_name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        rsp10_text!(#field_name, ri => gd, modified);
    }
}

fn generate_checkbox_field(field_name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        rsp10_check!(#field_name, ri => gd, modified);
    }
}

fn generate_button_field(field_name: &syn::Ident) -> proc_macro2::TokenStream {
    let field_name_str = field_name.to_string();
    let label = field_name_str
        .strip_prefix("btn")
        .unwrap_or(&field_name_str)
        .to_string();

    quote! {
        rsp10_button!(#field_name, #label => gd);
    }
}

fn generate_radio_field(_field_name: &syn::Ident) -> proc_macro2::TokenStream {
    // TODO: Implement radio button generation
    quote! {
        // Radio button not yet implemented
    }
}

fn generate_dropdown_field(field_name: &syn::Ident, field: &Field) -> proc_macro2::TokenStream {
    let source_fn = get_dropdown_source(field_name, field);

    quote! {
        rsp10_select!(#field_name, #source_fn(ri.state.#field_name), ri => gd, modified);
    }
}

fn extract_types_from_attrs(attrs: &[syn::Attribute]) -> (Option<syn::Type>, Option<syn::Type>) {
    let mut key_type = None;
    let mut auth_type = None;

    for attr in attrs {
        if attr.path().is_ident("rsp_key") {
            if let Ok(ty) = attr.parse_args::<syn::Type>() {
                key_type = Some(ty);
            }
        } else if attr.path().is_ident("rsp_auth") {
            if let Ok(ty) = attr.parse_args::<syn::Type>() {
                auth_type = Some(ty);
            }
        }
    }

    (key_type, auth_type)
}

fn get_dropdown_source(field_name: &syn::Ident, field: &Field) -> proc_macro2::TokenStream {
    // Check for explicit #[rsp_source(func_name)] attribute
    for attr in &field.attrs {
        if attr.path().is_ident("rsp_source") {
            if let Ok(source) = attr.parse_args::<syn::Ident>() {
                return quote! { #source };
            }
            // Try parsing as path (e.g., common::dropdowns::get_list)
            if let Ok(source) = attr.parse_args::<syn::Path>() {
                return quote! { #source };
            }
        }
    }

    // Convention 1: Try get_{full_field_name}
    let full_name = syn::Ident::new(
        &format!("get_{}", field_name),
        field_name.span()
    );

    // Convention 2: Try get_{name_without_dd_prefix}
    // TODO: Add fallback logic if function doesn't exist
    // let field_str = field_name.to_string();
    // let _stripped = field_str.strip_prefix("dd_").unwrap_or(&field_str);

    // For now, just use the full name convention
    quote! { #full_name }
}

fn generate_from_query_args(fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>, struct_name: &syn::Ident) -> proc_macro2::TokenStream {
    let mut field_extractions = Vec::new();
    let mut field_names = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        field_names.push(field_name);

        // Generate extraction code based on field type
        let extraction = match &field.ty {
            syn::Type::Path(syn::TypePath { path, .. }) => {
                if let Some(segment) = path.segments.last() {
                    match segment.ident.to_string().as_str() {
                        "Option" => {
                            // For Option<T> fields, make them optional
                            quote! {
                                let #field_name = args.get(#field_name_str)
                                    .and_then(|vals| vals.first())
                                    .and_then(|s| s.parse().ok());
                            }
                        }
                        "String" => {
                            quote! {
                                let #field_name = args.get(#field_name_str)
                                    .and_then(|vals| vals.first())
                                    .cloned()
                                    .unwrap_or_default();
                            }
                        }
                        "i32" | "i64" | "u32" | "u64" => {
                            quote! {
                                let #field_name = args.get(#field_name_str)
                                    .and_then(|vals| vals.first())
                                    .and_then(|s| s.parse().ok())
                                    .unwrap_or_default();
                            }
                        }
                        "bool" => {
                            quote! {
                                let #field_name = args.get(#field_name_str)
                                    .and_then(|vals| vals.first())
                                    .map(|s| s == "true" || s == "1")
                                    .unwrap_or(false);
                            }
                        }
                        _ => {
                            // Default for other types - try to parse
                            quote! {
                                let #field_name = args.get(#field_name_str)
                                    .and_then(|vals| vals.first())
                                    .and_then(|s| s.parse().ok());
                            }
                        }
                    }
                } else {
                    quote! {
                        let #field_name = None;
                    }
                }
            }
            _ => {
                quote! {
                    let #field_name = None;
                }
            }
        };

        field_extractions.push(extraction);
    }

    quote! {
        fn from_query_args(args: &std::collections::HashMap<String, Vec<String>>) -> Option<Self> {
            #(#field_extractions)*

            Some(#struct_name {
                #(#field_names),*
            })
        }
    }
}
