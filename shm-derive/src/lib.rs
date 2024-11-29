use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, Attribute, Data, DeriveInput, Error, Fields, GenericParam,
    Generics, Result,
};

#[proc_macro_derive(ShmInit)]
pub fn derive_to_shm(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let err = check_repr_c(&input.attrs, &name.span(), "ShmInit");
    if err.is_err() {
        return err.err().unwrap().into_compile_error().into();
    }

    let to_shm = to_shm_impl(&input.data, false);
    let to_shm_mut = to_shm_impl(&input.data, true);

    let expanded = quote! {
        unsafe impl #impl_generics shmoo::ShmInit for #name #ty_generics #where_clause {
            fn shm_init(shm: &mut shmoo::Shm) -> shmoo::error::Result<&Self> {
                #to_shm
            }

            fn shm_init_mut(shm: &mut shmoo::Shm) -> shmoo::error::Result<&mut Self> {
                #to_shm_mut
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn to_shm_impl(data: &Data, is_mut: bool) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(_) | Fields::Unnamed(_) | Fields::Unit => {
                let mut_tok = if is_mut {
                    quote! { mut }
                } else {
                    quote! {}
                };
                quote! {
                    let ptr = shm[..size_of::<Self>()].as_mut_ptr() as *mut Self;
                    assert!(ptr.is_aligned());
                    unsafe {
                        ptr.write(Self::default());
                        Ok(&#mut_tok *ptr)
                    }
                }
            }
        },
        Data::Enum(_) => unimplemented!(),
        Data::Union(_) => unimplemented!(),
    }
}

#[proc_macro_derive(FromShm)]
pub fn derive_from_shm(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let err = check_repr_c(&input.attrs, &name.span(), "FromShm");
    if err.is_err() {
        return err.err().unwrap().into_compile_error().into();
    }

    let from_shm = from_shm_impl(&input.data, false);
    let from_shm_mut = from_shm_impl(&input.data, true);

    let expanded = quote! {
        unsafe impl #impl_generics shmoo::FromShm for #name #ty_generics #where_clause {
            fn from_shm(shm: &shmoo::Shm) -> shmoo::error::Result<&Self> {
                #from_shm
            }

            fn from_shm_mut(shm: &mut shmoo::Shm) -> shmoo::error::Result<&mut Self> {
                #from_shm_mut
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn from_shm_impl(data: &Data, is_mut: bool) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(_) | Fields::Unnamed(_) | Fields::Unit => {
                let (mut_tok, const_tok, method) = if is_mut {
                    (quote! { mut }, quote! { mut }, quote! {as_mut_ptr})
                } else {
                    (quote! {}, quote! { const }, quote! {as_ptr})
                };
                quote! {
                    let size = size_of::<Self>();
                    if shm.len() < size {
                        return Err(shmoo::error::Error::new(shmoo::error::ErrorKind::SizeError(shm.len())));
                    }
                    let ptr = shm[..size].#method() as *#const_tok Self;
                    if !ptr.is_aligned() {
                        return Err(shmoo::error::Error::new(shmoo::error::ErrorKind::AlignmentError(align_of::<Self>())));
                    }
                    unsafe { Ok(&#mut_tok *ptr) }
                }
            }
        },
        Data::Enum(_) => unimplemented!(),
        Data::Union(_) => unimplemented!(),
    }
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(shmoo::ToShm));
        }
    }
    generics
}

fn check_repr_c(attrs: &[Attribute], span: &Span, trait_name: &str) -> Result<()> {
    let mut has_repr = false;
    let err_msg = &format!("{}: struct must be repr(C)", trait_name);
    for attr in attrs {
        if attr.path().is_ident("repr") {
            has_repr = true;
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("C") {
                    Ok(())
                } else {
                    Err(meta.error(err_msg))
                }
            })?;
        }
    }
    if has_repr {
        Ok(())
    } else {
        Err(Error::new(*span, err_msg))
    }
}
