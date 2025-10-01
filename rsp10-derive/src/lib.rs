use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field};

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

    let expanded = if let (Some(key_ty), Some(auth_ty)) = (key_type, auth_type) {
        // Generate with concrete types
        quote! {
            impl #name {
                pub fn auto_fill_data_impl<'a>(
                    mut ri: rsp10::RspInfo<'a, Self, #key_ty, #auth_ty>
                ) -> rsp10::RspFillDataResult<Self> {
                    let mut modified = false;
                    let mut gd = rsp10::RspDataBuilder::new();
                    #fill_data_impl
                    Self::fill_data_result(ri, gd)
                }
            }
        }
    } else {
        // Generate with generic types (fallback)
        quote! {
            impl #name {
                pub fn auto_fill_data_impl<'a, T, TA>(
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
