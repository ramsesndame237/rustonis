// Rustonis proc-macros

extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Lit, Meta};

// ─── Placeholders ─────────────────────────────────────────────────────────────

#[proc_macro_attribute]
pub fn provider(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn controller(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn inject(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn model(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn validator(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// ─── #[derive(Validate)] ──────────────────────────────────────────────────────

/// Génère une implémentation de `rustonis_validator::Validate` à partir des
/// attributs `#[validate(...)]` sur les champs.
///
/// # Règles supportées (chaînes)
/// - `#[validate(required)]` — non vide après trim
/// - `#[validate(email)]` — format email valide
/// - `#[validate(url)]` — format URL valide (http/https)
/// - `#[validate(alphanumeric)]` — seulement des caractères alphanumériques
/// - `#[validate(min_length = N)]` — longueur minimum
/// - `#[validate(max_length = N)]` — longueur maximum
///
/// # Règles supportées (numériques)
/// - `#[validate(min = N)]` — valeur minimum (cast en i64)
/// - `#[validate(max = N)]` — valeur maximum (cast en i64)
///
/// # Messages personnalisés
/// Toutes les règles acceptent un attribut `message` :
/// `#[validate(email, message = "Adresse email invalide")]`
///
/// # Exemple
///
/// ```rust,ignore
/// use serde::Deserialize;
/// use rustonis_macros::Validate;
///
/// #[derive(Deserialize, Validate)]
/// pub struct CreateUserInput {
///     #[validate(email)]
///     pub email: String,
///
///     #[validate(min_length = 8, max_length = 100)]
///     pub password: String,
///
///     #[validate(min = 18, max = 120)]
///     pub age: u32,
/// }
/// ```
#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_validate(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand_validate(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    name,
                    "#[derive(Validate)] ne supporte que les structs avec champs nommés",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                name,
                "#[derive(Validate)] ne supporte que les structs",
            ))
        }
    };

    let mut checks = Vec::<TokenStream2>::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        for attr in &field.attrs {
            if !attr.path().is_ident("validate") {
                continue;
            }

            // Parse la liste de metas dans #[validate(...)]
            let nested = attr.parse_args_with(
                syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
            )?;

            // Extraire le message custom s'il existe
            let custom_message: Option<String> = nested.iter().find_map(|m| {
                if let Meta::NameValue(nv) = m {
                    if nv.path.is_ident("message") {
                        if let syn::Expr::Lit(lit) = &nv.value {
                            if let Lit::Str(s) = &lit.lit {
                                return Some(s.value());
                            }
                        }
                    }
                }
                None
            });

            for meta in &nested {
                match meta {
                    // ── Flags booléens : required, email, url, alphanumeric ──
                    Meta::Path(path) => {
                        if path.is_ident("required") {
                            let msg = custom_message
                                .clone()
                                .unwrap_or_else(|| "is required".into());
                            checks.push(quote! {
                                if !::rustonis_validator::rules::required(
                                    ::std::convert::AsRef::<str>::as_ref(&self.#field_name)
                                ) {
                                    errors.add(#field_name_str, #msg);
                                }
                            });
                        } else if path.is_ident("email") {
                            let msg = custom_message
                                .clone()
                                .unwrap_or_else(|| "must be a valid email address".into());
                            checks.push(quote! {
                                if !::rustonis_validator::rules::email(
                                    ::std::convert::AsRef::<str>::as_ref(&self.#field_name)
                                ) {
                                    errors.add(#field_name_str, #msg);
                                }
                            });
                        } else if path.is_ident("url") {
                            let msg = custom_message
                                .clone()
                                .unwrap_or_else(|| "must be a valid URL".into());
                            checks.push(quote! {
                                if !::rustonis_validator::rules::url(
                                    ::std::convert::AsRef::<str>::as_ref(&self.#field_name)
                                ) {
                                    errors.add(#field_name_str, #msg);
                                }
                            });
                        } else if path.is_ident("alphanumeric") {
                            let msg = custom_message
                                .clone()
                                .unwrap_or_else(|| "must contain only alphanumeric characters".into());
                            checks.push(quote! {
                                if !::rustonis_validator::rules::alphanumeric(
                                    ::std::convert::AsRef::<str>::as_ref(&self.#field_name)
                                ) {
                                    errors.add(#field_name_str, #msg);
                                }
                            });
                        }
                        // "message" seul est ignoré (traité plus haut)
                    }

                    // ── Valeurs : min_length, max_length, min, max ───────────
                    Meta::NameValue(nv) => {
                        let key = nv.path.get_ident().map(|i| i.to_string());
                        let int_val = extract_int_lit(&nv.value);

                        match key.as_deref() {
                            Some("min_length") => {
                                if let Some(n) = int_val {
                                    let msg = custom_message.clone().unwrap_or_else(|| {
                                        format!("must be at least {n} characters")
                                    });
                                    let n = n as usize;
                                    checks.push(quote! {
                                        if !::rustonis_validator::rules::min_length(
                                            ::std::convert::AsRef::<str>::as_ref(&self.#field_name),
                                            #n
                                        ) {
                                            errors.add(#field_name_str, #msg);
                                        }
                                    });
                                }
                            }
                            Some("max_length") => {
                                if let Some(n) = int_val {
                                    let msg = custom_message.clone().unwrap_or_else(|| {
                                        format!("must be at most {n} characters")
                                    });
                                    let n = n as usize;
                                    checks.push(quote! {
                                        if !::rustonis_validator::rules::max_length(
                                            ::std::convert::AsRef::<str>::as_ref(&self.#field_name),
                                            #n
                                        ) {
                                            errors.add(#field_name_str, #msg);
                                        }
                                    });
                                }
                            }
                            Some("min") => {
                                if let Some(n) = int_val {
                                    let msg = custom_message
                                        .clone()
                                        .unwrap_or_else(|| format!("must be at least {n}"));
                                    checks.push(quote! {
                                        if !::rustonis_validator::rules::min_val(
                                            self.#field_name as i64,
                                            #n
                                        ) {
                                            errors.add(#field_name_str, #msg);
                                        }
                                    });
                                }
                            }
                            Some("max") => {
                                if let Some(n) = int_val {
                                    let msg = custom_message
                                        .clone()
                                        .unwrap_or_else(|| format!("must be at most {n}"));
                                    checks.push(quote! {
                                        if !::rustonis_validator::rules::max_val(
                                            self.#field_name as i64,
                                            #n
                                        ) {
                                            errors.add(#field_name_str, #msg);
                                        }
                                    });
                                }
                            }
                            _ => {} // message= et autres clés inconnues ignorées
                        }
                    }

                    Meta::List(_) => {} // syntaxes complexes ignorées pour l'instant
                }
            }
        }
    }

    Ok(quote! {
        impl ::rustonis_validator::Validate for #name {
            fn validate(&self) -> ::std::result::Result<(), ::rustonis_validator::ValidationErrors> {
                let mut errors = ::rustonis_validator::ValidationErrors::new();
                #(#checks)*
                errors.into_result()
            }
        }
    })
}

/// Extrait un entier littéral depuis une expression syn.
fn extract_int_lit(expr: &syn::Expr) -> Option<i64> {
    if let syn::Expr::Lit(lit) = expr {
        if let Lit::Int(i) = &lit.lit {
            return i.base10_parse::<i64>().ok();
        }
    }
    None
}
