use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{DeriveInput, parse_quote};

pub fn derive_try_from_action_data(input: TokenStream) -> TokenStream {
    match try_from_action_data(syn::parse_macro_input!(input as DeriveInput)) {
        Ok(expr) => expr.into_token_stream().into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn try_from_action_data(input: DeriveInput) -> syn::Result<syn::ItemImpl> {
    let ident = &input.ident;

    let attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("action_data"))
        .ok_or(syn::Error::new_spanned(
            &input,
            "`action_data` attribute required",
        ))?;
    let list = attr.meta.require_list()?;
    let dim = syn::parse2::<syn::Ident>(list.tokens.clone())?;

    Ok(parse_quote! {
        impl ::core::convert::TryFrom<::bevy_pretty_nice_input::derive::ActionData> for #ident {
            type Error = ::bevy::prelude::BevyError;

            fn try_from(value: ::bevy_pretty_nice_input::derive::ActionData) -> std::result::Result<Self, Self::Error> {
                match value {
                    ::bevy_pretty_nice_input::derive::ActionData::#dim(val) => Ok(Self(val)),
                    _ => Err(::bevy::prelude::BevyError::from(format!(
                        "Expected #dim, found {}",
                        value.debug_name()
                    ))),
                }
            }
        }
    })
}
