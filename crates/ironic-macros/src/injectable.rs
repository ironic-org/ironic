use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Fields, GenericArgument, LitStr, PathArguments, Type, parse2,
    spanned::Spanned,
};

pub(crate) fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let input = parse2::<DeriveInput>(input)?;
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "`Injectable` does not support generic provider types",
        ));
    }

    let mut scope = quote!(::ironic::Scope::Singleton);
    let mut eager = false;
    for attribute in input
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("injectable"))
    {
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("eager") {
                eager = true;
                return Ok(());
            }
            if meta.path.is_ident("scope") {
                let value = meta.value()?.parse::<LitStr>()?;
                scope = match value.value().as_str() {
                    "singleton" => quote!(::ironic::Scope::Singleton),
                    "transient" => quote!(::ironic::Scope::Transient),
                    "request" => quote!(::ironic::Scope::Request),
                    _ => {
                        return Err(syn::Error::new(
                            value.span(),
                            "scope must be `singleton`, `transient`, or `request`",
                        ));
                    }
                };
                return Ok(());
            }
            Err(meta.error(
                "supported options are `scope = \"singleton|transient|request\"` and `eager`",
            ))
        })?;
    }

    let name = &input.ident;
    let (dependencies, initializers) = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let mut dependencies = Vec::new();
                let mut initializers = Vec::new();
                for field in &fields.named {
                    let field_name = field.ident.as_ref().expect("named field");
                    let dependency = arc_inner(&field.ty)?;
                    dependencies.push(quote!(::ironic::Dependency::required::<#dependency>()));
                    initializers
                        .push(quote!(#field_name: resolver.resolve::<#dependency>().await?));
                }
                (dependencies, quote!(Self { #(#initializers),* }))
            }
            Fields::Unit => (Vec::new(), quote!(Self)),
            Fields::Unnamed(fields) => {
                return Err(syn::Error::new(
                    fields.span(),
                    "`Injectable` requires named fields or a unit struct",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new(
                input.span(),
                "`Injectable` can only be derived for structs",
            ));
        }
    };

    let eager_call = eager.then(|| quote!(.eager()));
    Ok(quote! {
        impl #name {
            /// Returns the dependency-injection registration generated for this type.
            pub fn provider_definition() -> ::ironic::ProviderDefinition {
                ::ironic::ProviderDefinition::factory(
                    #scope,
                    ::std::vec![#(#dependencies),*],
                    |resolver| async move { ::std::result::Result::Ok(#initializers) },
                )
                #eager_call
            }
        }
    })
}

fn arc_inner(ty: &Type) -> syn::Result<&Type> {
    let Type::Path(path) = ty else {
        return Err(syn::Error::new_spanned(
            ty,
            "injectable fields must have type `Arc<T>`",
        ));
    };
    let Some(segment) = path.path.segments.last() else {
        return Err(syn::Error::new_spanned(
            ty,
            "injectable fields must have type `Arc<T>`",
        ));
    };
    if segment.ident != "Arc" {
        return Err(syn::Error::new_spanned(
            ty,
            "injectable fields must have type `Arc<T>`",
        ));
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return Err(syn::Error::new_spanned(
            ty,
            "injectable fields must have type `Arc<T>`",
        ));
    };
    match arguments.args.first() {
        Some(GenericArgument::Type(inner)) if arguments.args.len() == 1 => Ok(inner),
        _ => Err(syn::Error::new_spanned(
            ty,
            "injectable fields must have type `Arc<T>`",
        )),
    }
}
