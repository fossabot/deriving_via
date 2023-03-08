use proc_macro2::TokenStream;
use quote::quote;
use syn::GenericParam;

use crate::utils::extract_single_field;

pub(crate) fn extract(input: &syn::DeriveInput, via: Option<&syn::Type>) -> TokenStream {
    let struct_name = &input.ident;
    let generics = {
        let lt = &input.generics.lt_token;
        let params = &input.generics.params;
        let gt = &input.generics.gt_token;

        quote! { #lt #params #gt }
    };
    let generic_params = {
        let lt = &input.generics.lt_token;
        let params = input.generics.params.iter().filter_map(|p| match p {
            GenericParam::Type(ty) => Some(&ty.ident),
            _ => None,
        });
        let gt = &input.generics.gt_token;

        quote! { #lt #(#params),* #gt }
    };
    let where_clause = &input.generics.where_clause;
    let field = extract_single_field(input);

    let field_ident = &field.ident;
    let field_ty = &field.ty;

    via.map_or_else(
        || {
            field_ident.as_ref().map_or_else(
                || {
                    quote! {
                        impl #generics std::convert::TryFrom<#field_ty> for #struct_name #generic_params #where_clause {
                            type Error = <#field_ty as std::str::TryFrom>::Error;

                            fn try_from(__: #field_ty) -> std::result::Result<Self, Self::Error> {
                                Ok(Self(__.try_into()?))
                            }
                        }
                    }
                },
                |field_name| {
                    quote! {
                        impl #generics std::convert::TryFrom<#field_ty> for #struct_name #generic_params #where_clause {
                            type Error = <#field_ty as std::str::TryFrom>::Error;

                            fn try_from(__: #field_ty) -> std::result::Result<Self, Self::Error> {
                                Ok(Self { #field_name: __.try_into()? })
                            }
                        }
                    }
                },
            )
        },
        |via| {
            quote! {
                impl #generics std::convert::TryFrom<#field_ty> for #struct_name #generic_params #where_clause {
                    type Error = <#via as std::str::TryFrom>::Error;

                    fn try_from(__: #field_ty) -> std::result::Result<Self, Self::Error> {
                        let intermediate: #via = __.try_into()?;
                        Ok(intermediate.into())
                    }
                }
            }
        },
    )
}
