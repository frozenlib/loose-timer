use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Expr, GenericArgument, ItemFn, Lit, LitFloat, LitStr, PathArguments, Result, ReturnType, Type,
    parse::{Parse, ParseStream},
    parse_quote, parse2,
};

pub fn timeout(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let attr: TimeoutArgs = parse2(attr)?;
    let mut func: ItemFn = parse2(item)?;

    let duration_expr = attr.into_duration_expr()?;
    let block = func.block;
    let is_async = func.sig.asyncness.is_some();
    let is_result = result_output(&func.sig.output).is_some();

    let timeout_call = if is_async {
        quote!(::loose_timer::timeout(__timeout_duration, async move #block).await)
    } else {
        quote!(::loose_timer::timeout_helpers::timeout_sync(move || #block, __timeout_duration))
    };
    let timeout_err_pat = if is_result { quote!(err) } else { quote!(_err) };
    let timeout_err_expr = if is_result {
        quote!(return ::core::result::Result::Err(::core::convert::Into::into(err));)
    } else {
        quote!(panic!("timeout");)
    };
    let wrapped_block = quote!({
        let __timeout_duration = ::loose_timer::IntoTimeoutDuration::into_timeout_duration(#duration_expr);
        let __timeout_result = #timeout_call;
        match __timeout_result {
            ::core::result::Result::Ok(value) => value,
            ::core::result::Result::Err(#timeout_err_pat) => {
                #timeout_err_expr
            }
        }
    });

    func.block = Box::new(parse_quote!(#wrapped_block));
    Ok(quote!(#func))
}

pub fn should_timeout(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let attr: TimeoutArgs = parse2(attr)?;
    let mut func: ItemFn = parse2(item)?;

    let duration_expr = attr.into_duration_expr()?;
    let block = func.block;
    let is_async = func.sig.asyncness.is_some();
    let output = &func.sig.output;
    let result_output = result_output(output);
    let is_result = result_output.is_some();
    let ok_is_unit = match result_output {
        Some((ok, _)) => is_unit_type(ok),
        None => {
            matches!(output, ReturnType::Default)
                || matches!(output, ReturnType::Type(_, ty) if is_unit_type(ty))
        }
    };
    if !ok_is_unit {
        bail!(
            Span::call_site(),
            "should_timeout only supports functions returning () or Result<(), E>"
        );
    }

    let timeout_body = match (is_async, is_result) {
        (true, true) => {
            quote!(async move {
                let _ = (async move #block).await;
            })
        }
        (true, false) => {
            quote!(async move #block)
        }
        (false, true) => {
            quote!(move || {
                let _ = #block;
            })
        }
        (false, false) => {
            quote!(move || #block)
        }
    };
    let should_timeout_call = if is_async {
        quote!(
            ::loose_timer::timeout_helpers::with_should_timeout_async(#timeout_body, __timeout_duration).await
        )
    } else {
        quote!(::loose_timer::timeout_helpers::with_should_timeout(#timeout_body, __timeout_duration))
    };
    let timeout_result_arms = if is_result {
        quote! {
            ::core::result::Result::Ok(()) => ::core::result::Result::Ok(()),
            ::core::result::Result::Err(err) => {
                ::core::result::Result::Err(::core::convert::Into::into(err))
            }
        }
    } else {
        quote! {
            ::core::result::Result::Ok(()) => (),
            ::core::result::Result::Err(_err) => {
                panic!("should timeout");
            }
        }
    };
    let wrapped_block = quote!({
        let __timeout_duration = ::loose_timer::IntoTimeoutDuration::into_timeout_duration(#duration_expr);
        let __timeout_result = #should_timeout_call;
        match __timeout_result {
            #timeout_result_arms
        }
    });

    func.block = Box::new(parse_quote!(#wrapped_block));
    Ok(quote!(#func))
}

struct TimeoutArgs {
    duration: Expr,
}

impl Parse for TimeoutArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            bail!(Span::call_site(), "timeout duration is required");
        }
        let duration: Expr = input.parse()?;
        if !input.is_empty() {
            bail!(Span::call_site(), "expected a single duration expression");
        }
        Ok(Self { duration })
    }
}

impl TimeoutArgs {
    fn into_duration_expr(self) -> Result<TokenStream> {
        match self.duration {
            Expr::Lit(expr_lit) => match expr_lit.lit {
                Lit::Str(lit) => parse_duration_literal(&lit),
                _ => Ok(quote!(#expr_lit)),
            },
            expr => Ok(quote!(#expr)),
        }
    }
}

fn result_output(output: &ReturnType) -> Option<(&Type, &Type)> {
    let ReturnType::Type(_, ty) = output else {
        return None;
    };
    let Type::Path(type_path) = ty.as_ref() else {
        return None;
    };
    let segment = type_path.path.segments.last()?;
    if segment.ident != "Result" {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    let mut iter = args.args.iter();
    let ok = match iter.next()? {
        GenericArgument::Type(ty) => ty,
        _ => return None,
    };
    let err = match iter.next()? {
        GenericArgument::Type(ty) => ty,
        _ => return None,
    };
    if iter.next().is_some() {
        return None;
    }
    Some((ok, err))
}

fn is_unit_type(ty: &Type) -> bool {
    matches!(ty, Type::Tuple(tuple) if tuple.elems.is_empty())
}

fn parse_duration_literal(lit: &LitStr) -> Result<TokenStream> {
    let raw = lit.value();
    if raw.is_empty() {
        bail!(lit.span(), "duration literal is empty");
    }

    let (number, unit) = if let Some(prefix) = raw.strip_suffix("ms") {
        (prefix, "ms")
    } else if let Some(prefix) = raw.strip_suffix('s') {
        (prefix, "s")
    } else if let Some(prefix) = raw.strip_suffix('m') {
        (prefix, "m")
    } else {
        bail!(lit.span(), "invalid duration literal");
    };

    if number.is_empty() {
        bail!(lit.span(), "invalid duration literal");
    }
    let value: f64 = number
        .parse()
        .map_err(|_| syn::Error::new(lit.span(), "invalid duration number"))?;
    if !value.is_finite() || value < 0.0 {
        bail!(lit.span(), "duration must be non-negative and finite");
    }

    let secs = match unit {
        "ms" => value / 1000.0,
        "s" => value,
        "m" => value * 60.0,
        _ => unreachable!(),
    };
    let mut secs_str = format!("{secs}");
    if !secs_str.contains(['.', 'e', 'E']) {
        secs_str.push_str(".0");
    }
    let lit = LitFloat::new(&secs_str, lit.span());
    Ok(quote!(::std::time::Duration::from_secs_f64(#lit)))
}
