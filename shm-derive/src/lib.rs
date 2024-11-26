use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, Attribute, Data, DeriveInput, Error, Fields, GenericParam,
    Generics, Result,
};

#[proc_macro_derive(ToShm)]
pub fn derive_to_shm(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let err = check_repr_c(&input.attrs, &name.span());
    if err.is_err() {
        return err.err().unwrap().into_compile_error().into();
    }

    let to_shm = to_shm_impl(&input.data, false);
    let to_shm_mut = to_shm_impl(&input.data, true);

    let expanded = quote! {
        unsafe impl #impl_generics shmoo::ToShm for #name #ty_generics #where_clause {
            fn to_shm(shm: &mut shmoo::Shm) -> &Self {
                #to_shm
            }

            fn to_shm_mut(shm: &mut shmoo::Shm) -> &mut Self {
                #to_shm_mut
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(shmoo::ToShm));
        }
    }
    generics
}

fn check_repr_c(attrs: &[Attribute], span: &Span) -> Result<()> {
    let mut has_repr = false;
    for attr in attrs {
        if attr.path().is_ident("repr") {
            has_repr = true;
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("C") {
                    Ok(())
                } else {
                    Err(meta.error("struct must be repr(C)"))
                }
            })?;
        }
    }
    if has_repr {
        Ok(())
    } else {
        Err(Error::new(*span, "struct must be repr(C)"))
    }
}

fn to_shm_impl(data: &Data, mut_spec: bool) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(_) | Fields::Unnamed(_) | Fields::Unit => {
                let mut_spec = if mut_spec {
                    quote! { mut }
                } else {
                    quote! {}
                };
                quote! {
                    let ptr = shm[..size_of::<Self>()].as_mut_ptr() as *mut Self;
                    assert!(ptr.is_aligned());
                    unsafe {
                        ptr.write(Self::default());
                        &#mut_spec *ptr
                    }
                }
            }
        },
        Data::Enum(_) => unimplemented!(),
        Data::Union(_) => unimplemented!(),
    }
}
