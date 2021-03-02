use darling::FromMeta;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::*;

#[proc_macro_attribute]
pub fn call_dispatch(args: TokenStream1, input: TokenStream1) -> TokenStream1 {
    let attr_args = parse_macro_input!(args as AttributeArgs);
    let args = match CallDispatchArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };
    let mut input = parse_macro_input!(input as ItemImpl);
    call_dispatch2(args, &mut input)
        .unwrap_or_else(|e| {
            let ce = e.into_compile_error();
            quote! { #input #ce }
        })
        .into()
}

fn call_dispatch2(args: CallDispatchArgs, input: &mut ItemImpl) -> Result<TokenStream> {
    let calls = take_call_attributes(input);
    gen_dispatcher(input, &calls);
    Ok(quote! { #input })
}

fn take_call_attributes(input: &mut ItemImpl) -> Vec<CallFn> {
    let mut calls = Vec::new();
    for item in &mut input.items {
        let method = match item {
            ImplItem::Method(m) => m,
            _ => continue,
        };
        let call_meta = match take_attribute(&mut method.attrs, "call") {
            Some(v) => v.parse_meta().unwrap(),
            _ => continue,
        };
        // let call_args = CallArgs::from_meta(&call_meta).unwrap();
        let fn_info = CallFn::from(&method.sig);
        calls.push(fn_info);
    }
    calls
}

/// Find and generate dispatcher function.
fn gen_dispatcher(input: &mut ItemImpl, calls: &[CallFn]) {
    for item in &mut input.items {
        let method = match item {
            ImplItem::Method(m) => m,
            _ => continue,
        };
        let dispatcher_meta = match take_attribute(&mut method.attrs, "dispatcher") {
            Some(v) => v.parse_meta().unwrap(),
            _ => continue,
        };
        let num_ident = match &method.sig.inputs[1] {
            FnArg::Typed(typed) => &typed.pat,
            _ => panic!(""),
        };
        let args_ident = match &method.sig.inputs[2] {
            FnArg::Typed(typed) => &typed.pat,
            _ => panic!(""),
        };
        // TODO: check fn signature
        let match_body = calls.iter().map(|f| {
            let name = format_ident!("{}", &f.name);
            let pattern = format_ident!("{}", f.name.to_uppercase());
            let args = (0..f.arg_num).map(|i| quote!(#args_ident[#i]));
            let await_suffix = if f.is_async { quote!(.await) } else { quote!() };
            quote! {
                #pattern => self.#name(#(#args as _),*)#await_suffix,
            }
        });
        let dispatcher_body = quote! {{
            let ret = match #num_ident {
                #(#match_body)*
                _ => return core::option::Option::None,
            };
            core::option::Option::Some(ret)
        }};
        method.block = syn::parse2::<Block>(dispatcher_body).unwrap();
    }
}

/// Find and remove attribute with specific `path`.
fn take_attribute(attrs: &mut Vec<Attribute>, path: &str) -> Option<Attribute> {
    attrs
        .iter()
        .position(|attr| {
            attr.path
                .get_ident()
                .map(|ident| ident == path)
                .unwrap_or(false)
        })
        .map(|idx| attrs.remove(idx))
}

#[derive(Debug, FromMeta)]
struct CallDispatchArgs {
    // attr_name: String,
// fn_name: String,
}

#[derive(Debug, FromMeta)]
struct CallArgs {}

#[derive(Debug, FromMeta)]
struct DispatcherArgs {}

/// Useful information of a call function.
#[derive(Debug)]
struct CallFn {
    name: String,
    is_async: bool,
    arg_num: usize,
}

impl From<&Signature> for CallFn {
    fn from(sig: &Signature) -> Self {
        CallFn {
            name: sig.ident.to_string(),
            is_async: sig.asyncness.is_some(),
            arg_num: sig.inputs.len() - 1,
        }
    }
}
