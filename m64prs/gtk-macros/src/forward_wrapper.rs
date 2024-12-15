use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{parse::Parse, punctuated::Punctuated, spanned::Spanned};

pub(crate) mod kw {
    syn::custom_keyword!(vis);
}

pub(crate) struct Args {
    ty: syn::Type,
    comma: Option<syn::Token![,]>,
    extra_args: Punctuated<ArgsNameValue, syn::Token![,]>,
}

pub(crate) struct MetaVis {
    vis_token: kw::vis,
    equals_token: syn::Token![=],
    vis: syn::Visibility,
}

pub(crate) enum ArgsNameValue {
    Vis(MetaVis),
    NameValue(syn::MetaNameValue),
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ty = input.parse()?;

        let mut comma = None;
        let mut extra_args = Punctuated::new();
        if input.peek(syn::Token![,]) {
            comma = input.parse()?;
            extra_args = Punctuated::parse_separated_nonempty(input)?;
        }

        Ok(Self {
            ty,
            comma,
            extra_args,
        })
    }
}

impl Parse for MetaVis {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis_token = input.parse()?;
        let equals_token = input.parse()?;
        let vis = input.parse()?;
        Ok(Self {
            vis_token,
            equals_token,
            vis,
        })
    }
}

impl Parse for ArgsNameValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let token = if input.peek(kw::vis) {
            Self::Vis(input.parse()?)
        } else {
            Self::NameValue(input.parse()?)
        };
        Ok(token)
    }
}

pub(crate) fn generate(args: Args, impl_block: syn::ItemImpl) -> syn::Result<TokenStream> {
    let mut block_contents = TokenStream::new();
    let mut forward_vis: Option<syn::Visibility> = None;
    let wrapper_type = &args.ty;

    for kv_pair in &args.extra_args {
        match kv_pair {
            ArgsNameValue::Vis(meta_vis) => {
                forward_vis = Some(meta_vis.vis.clone())
            },
            ArgsNameValue::NameValue(_) => {},
        }
    }

    for item in impl_block.items {
        if let syn::ImplItem::Fn(fn_block) = item {
            let sig = &fn_block.sig;

            if !check_self_ref(sig) {
                return Err(syn::Error::new(
                    sig.inputs.span(),
                    "Forwarded functions should take &self",
                ));
            }
            let mut visit_state = ParamVisitState::default();

            for param in &sig.inputs {
                gen_param_list(&param, &mut visit_state)?;
            }
            let new_sig = transform_sig(sig, &visit_state)?;
            let call = forward_call(sig, &visit_state)?;

            block_contents.extend(quote! {
                #forward_vis #new_sig {
                    #call
                }
            });
        }
    }

    Ok(quote! {
        impl #wrapper_type {
            #block_contents
        }
    })
}

fn check_self_ref(sig: &syn::Signature) -> bool {
    let receiver = match sig.receiver() {
        Some(receiver) => receiver,
        None => return false,
    };

    match &*receiver.ty {
        syn::Type::Reference(ref_ty) => return ref_ty.mutability.is_none(),
        _ => return false,
    }
}

#[derive(Default)]
struct ParamVisitState {
    gen_counter: usize,
    fwd_args: Vec<TokenStream>,
    fwd_inputs: Vec<TokenStream>,
}

impl ParamVisitState {
    fn gen_id(&mut self) -> Ident {
        let result = format_ident!("gen{}", self.gen_counter);
        self.gen_counter += 1;
        result
    }
}

fn gen_param_list(param: &syn::FnArg, visit_state: &mut ParamVisitState) -> syn::Result<()> {
    match param {
        syn::FnArg::Receiver(receiver) => {
            let lifetime = receiver.lifetime();
            visit_state.fwd_args.push(quote! {
                &#lifetime self
            });
        },
        syn::FnArg::Typed(param) => {
            let ty = &*param.ty;
            let ident = match &*param.pat {
                syn::Pat::Ident(pat_ident) => pat_ident.ident.clone(),
                _ => visit_state.gen_id(),
            };

            visit_state.fwd_args.push(quote! {
                #ident: #ty
            });
            visit_state.fwd_inputs.push(quote! {
                #ident
            });
        },
    }

    Ok(())
}

fn transform_sig(sig: &syn::Signature, visit_state: &ParamVisitState) -> syn::Result<TokenStream> {
    let syn::Signature {
        constness,
        asyncness,
        unsafety,
        abi,
        ident,
        generics,
        output,
        ..
    } = sig;

    let args = &visit_state.fwd_args;

    Ok(quote! {
        #constness #asyncness #unsafety #abi fn #ident #generics (
            #(#args),*
        ) #output
    })
}

fn forward_call(sig: &syn::Signature, visit_state: &ParamVisitState) -> syn::Result<TokenStream> {
    let name = &sig.ident;
    let inputs = &visit_state.fwd_inputs;

    let opt_await = sig.asyncness.is_some().then_some(quote! {.await});

    Ok(quote! {
        glib::subclass::types::ObjectSubclassIsExt::imp(self)
            .#name(#(#inputs),*)#opt_await
    })
}
