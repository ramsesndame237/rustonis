// Rustonis proc-macros

extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Lit, Meta, LitStr};

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

/// `#[derive(Model)]` — génère l'implémentation Active Record pour une struct.
///
/// # Ce que la macro génère
///
/// 1. `impl rustonis_orm::Model for StructName` avec `table_name()` (snake_case pluriel)
/// 2. `derive(sqlx::FromRow)` pour la désérialisation depuis la DB
/// 3. `impl StructName { async fn save(&self, pool) }` — UPDATE
/// 4. `impl StructName { async fn delete(&self, pool) }` — DELETE
/// 5. `impl StructName { async fn create(pool, |s| ...) }` — INSERT via builder
///
/// # Options d'attribut
///
/// - `#[model(table = "my_table")]` — override le nom de table
/// - `#[model(primary_key = "uuid")]` — override la colonne PK (défaut: `id`)
///
/// # Exemple
///
/// ```rust,ignore
/// use rustonis_orm::prelude::*;
///
/// #[derive(Model, Debug, Clone, serde::Serialize, serde::Deserialize)]
/// pub struct User {
///     pub id:    i64,
///     pub email: String,
///     pub name:  String,
/// }
/// ```
#[proc_macro_derive(model, attributes(model))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_model(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand_model(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;

    // ── Parse #[model(table = "...", primary_key = "...")] ────────────────
    let mut table_override: Option<String> = None;
    let mut pk_override: Option<String>    = None;

    for attr in &input.attrs {
        if !attr.path().is_ident("model") { continue; }
        let nested = attr.parse_args_with(
            syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
        )?;
        for meta in &nested {
            if let Meta::NameValue(nv) = meta {
                if nv.path.is_ident("table") {
                    if let syn::Expr::Lit(lit) = &nv.value {
                        if let Lit::Str(s) = &lit.lit { table_override = Some(s.value()); }
                    }
                } else if nv.path.is_ident("primary_key") {
                    if let syn::Expr::Lit(lit) = &nv.value {
                        if let Lit::Str(s) = &lit.lit { pk_override = Some(s.value()); }
                    }
                }
            }
        }
    }

    // ── Derive table name: PascalCase → snake_case + "s" ─────────────────
    let struct_name = name.to_string();
    let table_name = table_override.unwrap_or_else(|| to_table_name(&struct_name));
    let pk_col     = pk_override.unwrap_or_else(|| "id".to_string());

    // ── Extract struct fields ─────────────────────────────────────────────
    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => return Err(syn::Error::new_spanned(name, "#[derive(model)] requires named fields")),
        },
        _ => return Err(syn::Error::new_spanned(name, "#[derive(model)] requires a struct")),
    };

    let pk_ident = syn::Ident::new(&pk_col, name.span());

    // Non-PK fields (used in UPDATE SET and INSERT)
    let non_pk_fields: Vec<_> = fields
        .iter()
        .filter(|f| f.ident.as_ref().map(|i| i != &pk_ident).unwrap_or(true))
        .collect();

    // ── Generate UPDATE SQL: "SET col1 = ?, col2 = ?" ────────────────────
    let update_set: Vec<String> = non_pk_fields
        .iter()
        .map(|f| format!("{} = ?", f.ident.as_ref().unwrap()))
        .collect();
    let update_sql = format!(
        "UPDATE {} SET {} WHERE {} = ?",
        table_name,
        update_set.join(", "),
        pk_col,
    );

    let save_binds: Vec<TokenStream2> = non_pk_fields
        .iter()
        .map(|f| {
            let ident = f.ident.as_ref().unwrap();
            quote! { .bind(&self.#ident) }
        })
        .collect();

    // ── Generate INSERT SQL ───────────────────────────────────────────────
    let insert_cols: Vec<String> = non_pk_fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect();
    let placeholders = vec!["?"; non_pk_fields.len()].join(", ");
    let insert_sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table_name,
        insert_cols.join(", "),
        placeholders,
    );

    let table_name_lit  = LitStr::new(&table_name, name.span());
    let pk_col_lit      = LitStr::new(&pk_col, name.span());
    let update_sql_lit  = LitStr::new(&update_sql, name.span());
    let insert_sql_lit  = LitStr::new(&insert_sql, name.span());

    // ── Emit code ─────────────────────────────────────────────────────────
    Ok(quote! {
        // Derive sqlx::FromRow so the struct can be loaded from any DB row
        #[derive(::sqlx::FromRow)]
        #[allow(dead_code)]

        impl ::rustonis_orm::Model for #name {
            fn table_name() -> &'static str { #table_name_lit }
            fn primary_key() -> &'static str { #pk_col_lit }
        }

        impl #name {
            /// Persist changes to an existing row (UPDATE).
            pub async fn save(
                &self,
                pool: &::sqlx::AnyPool,
            ) -> ::std::result::Result<(), ::rustonis_orm::OrmError> {
                let mut q = ::sqlx::query::<::sqlx::Any>(#update_sql_lit);
                #(q = q #save_binds;)*
                q = q.bind(self.#pk_ident);
                q.execute(pool).await.map_err(::rustonis_orm::OrmError::from)?;
                Ok(())
            }

            /// Delete this row from the database (DELETE).
            pub async fn delete(
                &self,
                pool: &::sqlx::AnyPool,
            ) -> ::std::result::Result<(), ::rustonis_orm::OrmError> {
                let sql = format!(
                    "DELETE FROM {} WHERE {} = ?",
                    #table_name_lit,
                    #pk_col_lit,
                );
                ::sqlx::query::<::sqlx::Any>(&sql)
                    .bind(self.#pk_ident)
                    .execute(pool)
                    .await
                    .map_err(::rustonis_orm::OrmError::from)?;
                Ok(())
            }

            /// Insert a new row.  Returns the inserted id.
            ///
            /// Populate the struct with desired values (leave `id` as 0),
            /// then call this method.
            pub async fn insert(
                &self,
                pool: &::sqlx::AnyPool,
            ) -> ::std::result::Result<i64, ::rustonis_orm::OrmError> {
                let mut q = ::sqlx::query::<::sqlx::Any>(#insert_sql_lit);
                #(q = q #save_binds;)*
                let result = q.execute(pool).await.map_err(::rustonis_orm::OrmError::from)?;
                Ok(result.last_insert_id() as i64)
            }
        }
    })
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

/// Converts `PascalCase` → `snake_case` and appends `s` for table naming.
/// e.g. `User` → `users`, `BlogPost` → `blog_posts`
fn to_table_name(pascal: &str) -> String {
    let mut out = String::new();
    for (i, ch) in pascal.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.extend(ch.to_lowercase());
    }
    out.push('s');
    out
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
