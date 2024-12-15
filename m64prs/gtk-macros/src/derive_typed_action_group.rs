use proc_macro2::{Literal, TokenStream};
use quote::quote;
use syn::{meta::ParseNestedMeta, punctuated::Punctuated, spanned::Spanned, Token};

pub(crate) fn generate(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let data = match &input.data {
        syn::Data::Struct(data_struct) => data_struct,
        syn::Data::Enum(data_enum) => {
            return Err(syn::Error::new_spanned(
                data_enum.enum_token,
                "derive(TypedActionGroup) can only be applied to structs",
            ))
        }
        syn::Data::Union(data_union) => {
            return Err(syn::Error::new_spanned(
                data_union.union_token,
                "derive(TypedActionGroup) can only be applied to structs",
            ))
        }
    };

    match &data.fields {
        syn::Fields::Named(fields_named) => generate_named(&input.ident, &fields_named),
        syn::Fields::Unnamed(fields_unnamed) => generate_unnamed(&input.ident, &fields_unnamed),
        syn::Fields::Unit => {
            return Err(syn::Error::new_spanned(
                data.struct_token,
                "derive(TypedActionGroup) should not be applied to unit structs",
            ))
        }
    }
}

fn generate_named(id_struct: &syn::Ident, fields: &syn::FieldsNamed) -> syn::Result<TokenStream> {
    let mut init_fields = Vec::<TokenStream>::new();
    let mut register_stmts = Vec::<TokenStream>::new();

    for field in &fields.named {
        let action_attr = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("action"))
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    field,
                    "fields should be annotated with #[action(name = \"name\")]",
                )
            })?;
        let info = ActionInfo::from_attribute(action_attr)?;

        let ActionInfo { name, default } = &info;

        let ident = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(field, "No identifier for field"))?;

        // generate init field:
        init_fields.push(match default {
            Some(_) => quote! {
                #ident: ::m64prs_gtk_utils::actions::TypedAction::with_state(#name, #default)
            },
            None => quote! {
                #ident: ::m64prs_gtk_utils::actions::TypedAction::new(#name)
            },
        });

        register_stmts.push(quote! {
            ::m64prs_gtk_utils::actions::ActionMapTypedExt::register_action(map, &self.#ident);
        });
    }

    Ok(quote! {
        impl ::m64prs_gtk_utils::actions::TypedActionGroup for #id_struct {
            fn new_default() -> Self {
                Self {
                    #(#init_fields),*
                }
            }

            fn register_to(&self, map: &impl glib::object::IsA<gio::ActionMap>) {
                #(#register_stmts)*
            }
        }
    })
}

fn generate_unnamed(
    id_struct: &syn::Ident,
    fields: &syn::FieldsUnnamed,
) -> syn::Result<TokenStream> {
    let mut init_fields = Vec::<TokenStream>::new();
    let mut register_stmts = Vec::<TokenStream>::new();

    for (index, field) in fields.unnamed.iter().enumerate() {
        let action_attr = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("action"))
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    field,
                    "fields should be annotated with #[action(name = \"name\")]",
                )
            })?;
        let info = ActionInfo::from_attribute(action_attr)?;

        let ActionInfo { name, default } = &info;

        let ident = Literal::usize_unsuffixed(index);

        // generate init field:
        init_fields.push(match default {
            Some(_) => quote! {
                ::m64prs_gtk_utils::actions::TypedAction::with_state(#name, #default)
            },
            None => quote! {
                ::m64prs_gtk_utils::actions::TypedAction::new(#name)
            },
        });

        register_stmts.push(quote! {
            ::m64prs_gtk_utils::actions::ActionMapTypedExt::register_action(map, &self.#ident);
        });
    }

    Ok(quote! {
        #[automatically_derived]
        impl ::m64prs_gtk_utils::actions::TypedActionGroup for #id_struct {
            fn new_default() -> Self {
                Self(#(#init_fields),*)
            }

            fn register_to(&self, map: &impl glib::object::IsA<gio::ActionMap>) {
                #(#register_stmts)*
            }
        }
    })
}

struct ActionInfo {
    name: syn::LitStr,
    default: Option<syn::Expr>,
}

impl ActionInfo {
    fn from_attribute(attr: &syn::Attribute) -> syn::Result<Self> {
        let mut name: Option<syn::LitStr> = None;
        let mut default: Option<syn::Expr> = None;

        let kv_pairs: Punctuated<syn::MetaNameValue, Token![,]> =
            attr.parse_args_with(Punctuated::parse_separated_nonempty)?;

        for pair in &kv_pairs {
            if pair.path.is_ident("name") {
                match &pair.value {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
                        ..
                    }) => name = Some(lit_str.clone()),
                    _ => return Err(syn::Error::new_spanned(pair, "name should be a string literal")),
                }
            }
            else if pair.path.is_ident("default") {
                default = Some(pair.value.clone());
            }
        }

        let name = name
            .ok_or_else(|| syn::Error::new_spanned(attr, "name must be provided in #[action]"))?;

        Ok(Self { name, default })
    }
}
