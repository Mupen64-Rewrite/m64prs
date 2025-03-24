use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote, ToTokens};

pub(crate) fn generate(closure: syn::ExprClosure) -> syn::Result<TokenStream> {
    let mut params: Vec<TokenStream> = Vec::new();
    let mut decls: Vec<TokenStream> = Vec::new();

    for (index, param) in closure.inputs.iter().enumerate() {
        let pat_type = match param {
            syn::Pat::Type(pat_type) => pat_type,
            bad_pattern => {
                return Err(syn::Error::new_spanned(
                    bad_pattern,
                    "All parameters must be typed",
                ))
            }
        };

        let param_n = format_ident!("param{}", index);
        let ty = &pat_type.ty;
        let n = Literal::usize_suffixed(index);
        let fail_msg = Literal::string(&format!(
            "param {} is not a {}",
            index,
            ty.into_token_stream()
        ));

        decls.push(quote! {
            let #param_n: #ty = __values[#n].get().expect(#fail_msg);
        });
        params.push(param_n.into_token_stream());
    }

    let closure_call = if rt_is_unit(&closure.output) {
        quote! {
            __base_closure(#(#params),*);
            None
        }
    } else {
        quote! {
            Some(glib::prelude::ToValue::to_value(__base_closure(#(#params),*)))
        }
    };

    let n_params = Literal::usize_suffixed(params.len());

    Ok(quote! {
        {
            let __base_closure = #closure;
            move |__values: &[glib::Value]| -> Option<glib::Value> {
                assert!(__values.len() == #n_params);
                #(#decls)*
                #closure_call
            }
        }
    })
}

fn rt_is_unit(rt: &syn::ReturnType) -> bool {
    match rt {
        syn::ReturnType::Default => true,
        syn::ReturnType::Type(_, ty) => match &**ty {
            syn::Type::Tuple(type_tuple) => type_tuple.elems.empty_or_trailing(),
            _ => false,
        },
    }
}
