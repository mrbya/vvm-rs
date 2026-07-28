use syn::{Data, DeriveInput, Error, Expr, Field, Fields, Ident, LitStr, Path, Result, Type};

/// Parsed coverage derive input.
pub struct Input {
    /// Coverage model type.
    pub(super) ident: Ident,
    /// Stable definition name.
    pub(super) definition: LitStr,
    /// Definition revision.
    pub(super) revision: Expr,
    /// Stimulus type.
    pub(super) stimulus: Type,
    /// Observation type.
    pub(super) observation: Type,
    /// Ordered coverage fields.
    pub(super) fields: Vec<FieldRole>,
}

/// One annotated coverage field.
pub enum FieldRole {
    /// Typed coverpoint field.
    Coverpoint {
        /// Field name.
        ident: Ident,
        /// Coverpoint builder path.
        build: Path,
        /// Transaction extractor path.
        sample: Path,
    },
    /// Two-way cross field.
    Cross {
        /// Field name.
        ident: Ident,
        /// Left source coverpoint.
        left: Ident,
        /// Right source coverpoint.
        right: Ident,
        /// Optional custom builder path.
        build: Option<Path>,
    },
}

impl Input {
    /// Parses one derive input.
    pub(super) fn parse(input: DeriveInput) -> Result<Self> {
        if !input.generics.params.is_empty() || input.generics.where_clause.is_some() {
            return Err(Error::new_spanned(
                input.generics,
                "generic coverage structs are not supported",
            ));
        }

        let attributes = GroupAttributes::parse(&input.attrs)?;
        validate_definition_name(&attributes.definition)?;
        let fields = match input.data {
            Data::Struct(data) => match data.fields {
                Fields::Named(fields) => fields.named.into_iter().collect::<Vec<_>>(),
                Fields::Unnamed(fields) => {
                    return Err(Error::new_spanned(
                        fields,
                        "Coverage supports only named-field structs",
                    ));
                }
                Fields::Unit => {
                    return Err(Error::new_spanned(
                        input.ident,
                        "Coverage supports only named-field structs",
                    ));
                }
            },
            Data::Enum(data) => {
                return Err(Error::new_spanned(
                    data.enum_token,
                    "Coverage supports only named-field structs",
                ));
            }
            Data::Union(data) => {
                return Err(Error::new_spanned(
                    data.union_token,
                    "Coverage supports only named-field structs",
                ));
            }
        };
        if fields.is_empty() {
            return Err(Error::new_spanned(
                input.ident,
                "Coverage structs must contain at least one field",
            ));
        }

        let mut roles = Vec::new();
        let mut coverpoints = Vec::new();
        let mut saw_cross = false;
        for field in &fields {
            let role = FieldRole::parse(field)?;
            match role {
                FieldRole::Coverpoint { ref ident, .. } => {
                    if saw_cross {
                        return Err(Error::new_spanned(
                            ident,
                            "coverpoint fields must appear before cross fields",
                        ));
                    }
                    coverpoints.push(ident.clone());
                }
                FieldRole::Cross {
                    ref left,
                    ref right,
                    ..
                } => {
                    saw_cross = true;
                    for source in [left, right] {
                        if !coverpoints.iter().any(|coverpoint| coverpoint == source) {
                            return Err(Error::new_spanned(
                                source,
                                "cross sources must name previously declared coverpoint fields",
                            ));
                        }
                    }
                }
            }
            roles.push(role);
        }

        Ok(Self {
            ident: input.ident,
            definition: attributes.definition,
            revision: attributes.revision,
            stimulus: attributes.stimulus,
            observation: attributes.observation,
            fields: roles,
        })
    }
}

/// Parsed group-level attributes.
struct GroupAttributes {
    /// Stable definition name.
    definition: LitStr,
    /// Definition revision expression.
    revision: Expr,
    /// Stimulus type.
    stimulus: Type,
    /// Observation type.
    observation: Type,
}
impl GroupAttributes {
    /// Parses group-level attributes.
    fn parse(attributes: &[syn::Attribute]) -> Result<Self> {
        let mut definition = None;
        let mut revision = None;
        let mut stimulus = None;
        let mut observation = None;
        for attribute in attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident("vvm"))
        {
            attribute.parse_nested_meta(|meta| {
                if meta.path.is_ident("definition") {
                    if definition.is_some() {
                        return Err(meta.error("duplicate `definition` option"));
                    }
                    definition = Some(meta.value()?.parse()?);
                    return Ok(());
                }
                if meta.path.is_ident("revision") {
                    if revision.is_some() {
                        return Err(meta.error("duplicate `revision` option"));
                    }
                    revision = Some(meta.value()?.parse()?);
                    return Ok(());
                }
                if meta.path.is_ident("stimulus") {
                    if stimulus.is_some() {
                        return Err(meta.error("duplicate `stimulus` option"));
                    }
                    stimulus = Some(meta.value()?.parse()?);
                    return Ok(());
                }
                if meta.path.is_ident("observation") {
                    if observation.is_some() {
                        return Err(meta.error("duplicate `observation` option"));
                    }
                    observation = Some(meta.value()?.parse()?);
                    return Ok(());
                }
                Err(meta.error(
                    "unsupported Coverage option: expected `definition`, `revision`, `stimulus`, \
                     or `observation`",
                ))
            })?;
        }
        Ok(Self {
            definition: definition.ok_or_else(|| {
                Error::new(
                    proc_macro2::Span::call_site(),
                    "missing `definition` option",
                )
            })?,
            revision: revision.ok_or_else(|| {
                Error::new(proc_macro2::Span::call_site(), "missing `revision` option")
            })?,
            stimulus: stimulus.ok_or_else(|| {
                Error::new(proc_macro2::Span::call_site(), "missing `stimulus` option")
            })?,
            observation: observation.ok_or_else(|| {
                Error::new(
                    proc_macro2::Span::call_site(),
                    "missing `observation` option",
                )
            })?,
        })
    }
}

impl FieldRole {
    /// Parses one coverage field role.
    fn parse(field: &Field) -> Result<Self> {
        let ident = field.ident.clone().ok_or_else(|| {
            Error::new_spanned(field, "Coverage supports only named-field structs")
        })?;
        let mut role = None;
        for attribute in field
            .attrs
            .iter()
            .filter(|attribute| attribute.path().is_ident("vvm"))
        {
            attribute.parse_nested_meta(|meta| {
                if role.is_some() {
                    return Err(meta.error("duplicate coverage field role"));
                }
                let next = if meta.path.is_ident("coverpoint") {
                    if !is_named_type(&field.ty, "Coverpoint") {
                        return Err(meta.error("coverpoint fields must have type `Coverpoint<T>`"));
                    }
                    let (build, sample) = parse_coverpoint(&meta)?;
                    Self::Coverpoint {
                        ident: ident.clone(),
                        build,
                        sample,
                    }
                } else if meta.path.is_ident("cross") {
                    if !is_named_type(&field.ty, "Cross2") {
                        return Err(meta.error("cross fields must have type `Cross2`"));
                    }
                    let (left, right, build) = parse_cross(&meta)?;
                    Self::Cross {
                        ident: ident.clone(),
                        left,
                        right,
                        build,
                    }
                } else {
                    return Err(
                        meta.error("coverage fields require `coverpoint(...)` or `cross(...)`")
                    );
                };
                role = Some(next);
                Ok(())
            })?;
        }
        role.ok_or_else(|| Error::new_spanned(field, "coverage fields require an explicit role"))
    }
}

/// Parses one coverpoint role.
fn parse_coverpoint(meta: &syn::meta::ParseNestedMeta<'_>) -> Result<(Path, Path)> {
    let mut build = None;
    let mut sample = None;
    meta.parse_nested_meta(|nested| {
        if nested.path.is_ident("build") {
            if build.is_some() {
                return Err(nested.error("duplicate `build` option"));
            }
            build = Some(nested.value()?.parse()?);
            return Ok(());
        }
        if nested.path.is_ident("sample") {
            if sample.is_some() {
                return Err(nested.error("duplicate `sample` option"));
            }
            sample = Some(nested.value()?.parse()?);
            return Ok(());
        }
        Err(nested.error("unsupported coverpoint option: expected `build` or `sample`"))
    })?;
    Ok((
        build.ok_or_else(|| meta.error("coverpoint requires `build`"))?,
        sample.ok_or_else(|| meta.error("coverpoint requires `sample`"))?,
    ))
}

/// Parses one cross role.
fn parse_cross(meta: &syn::meta::ParseNestedMeta<'_>) -> Result<(Ident, Ident, Option<Path>)> {
    let mut left = None;
    let mut right = None;
    let mut build = None;
    meta.parse_nested_meta(|nested| {
        if nested.path.is_ident("left") {
            if left.is_some() {
                return Err(nested.error("duplicate `left` option"));
            }
            left = Some(nested.value()?.parse()?);
            return Ok(());
        }
        if nested.path.is_ident("right") {
            if right.is_some() {
                return Err(nested.error("duplicate `right` option"));
            }
            right = Some(nested.value()?.parse()?);
            return Ok(());
        }
        if nested.path.is_ident("build") {
            if build.is_some() {
                return Err(nested.error("duplicate `build` option"));
            }
            build = Some(nested.value()?.parse()?);
            return Ok(());
        }
        Err(nested.error("unsupported cross option: expected `left`, `right`, or `build`"))
    })?;
    Ok((
        left.ok_or_else(|| meta.error("cross requires `left`"))?,
        right.ok_or_else(|| meta.error("cross requires `right`"))?,
        build,
    ))
}

/// Returns whether a type ends in the expected identifier.
fn is_named_type(ty: &Type, expected: &str) -> bool {
    matches!(ty, Type::Path(path) if path.qself.is_none() && path.path.segments.last().is_some_and(|segment| segment.ident == expected))
}

/// Validates a coverage definition identifier.
fn validate_definition_name(definition: &LitStr) -> Result<()> {
    let value = definition.value();
    let mut characters = value.chars();
    let valid = characters
        .next()
        .is_some_and(|first| first.is_ascii_lowercase())
        && characters.all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        });
    if valid {
        Ok(())
    } else {
        Err(Error::new(
            definition.span(),
            "invalid coverage definition name",
        ))
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::{DeriveInput, parse2};

    use super::{FieldRole, Input};

    #[test]
    fn parses_coverpoints_and_crosses() -> Result<(), Box<dyn std::error::Error>> {
        let input: DeriveInput = parse2(quote! {
            #[vvm(definition = "model", revision = 3, stimulus = Stimulus, observation = Observation)]
            struct Model {
                #[vvm(coverpoint(build = build_left, sample = sample_left))]
                left: Coverpoint<u8>,
                #[vvm(coverpoint(build = crate::build_right, sample = crate::sample_right))]
                right: Coverpoint<bool>,
                #[vvm(cross(left = left, right = right))]
                left_x_right: Cross2,
            }
        })?;
        let parsed = Input::parse(input)?;

        assert_eq!(parsed.definition.value(), "model");
        assert_eq!(parsed.fields.len(), 3);
        assert!(matches!(
            parsed.fields.first(),
            Some(FieldRole::Coverpoint { .. })
        ));
        assert!(matches!(
            parsed.fields.get(2),
            Some(FieldRole::Cross { .. })
        ));
        Ok(())
    }

    #[test]
    fn rejects_cross_before_source() -> Result<(), Box<dyn std::error::Error>> {
        let input: DeriveInput = parse2(quote! {
            #[vvm(definition = "model", revision = 1, stimulus = S, observation = O)]
            struct Model {
                #[vvm(cross(left = left, right = left))]
                cross: Cross2,
                #[vvm(coverpoint(build = build_left, sample = sample_left))]
                left: Coverpoint<u8>,
            }
        })?;

        assert!(Input::parse(input).is_err());
        Ok(())
    }

    #[test]
    fn retains_coverage_metadata_and_custom_cross_builder() -> Result<(), Box<dyn std::error::Error>>
    {
        let input: DeriveInput = parse2(quote! {
            #[vvm(definition = "bus_model", revision = REVISION + 1, stimulus = crate::Stimulus, observation = Observation)]
            struct Model {
                #[vvm(coverpoint(build = build_address, sample = sample_address))]
                address: Coverpoint<u16>,
                #[vvm(cross(left = address, right = address, build = build_cross))]
                address_x_address: Cross2,
            }
        })?;

        let parsed = Input::parse(input)?;
        let revision = &parsed.revision;
        let stimulus = &parsed.stimulus;
        let observation = &parsed.observation;

        assert_eq!(parsed.ident, "Model");
        assert_eq!(parsed.definition.value(), "bus_model");
        assert_eq!(quote!(#revision).to_string(), "REVISION + 1");
        assert_eq!(quote!(#stimulus).to_string(), "crate :: Stimulus");
        assert_eq!(quote!(#observation).to_string(), "Observation");
        assert!(matches!(
            parsed.fields.get(1),
            Some(FieldRole::Cross { build: Some(_), .. })
        ));
        Ok(())
    }

    #[test]
    fn rejects_invalid_structures_and_metadata_with_specific_diagnostics() {
        let cases: [(DeriveInput, &str); 7] = [
            (
                parse2(quote! {
                    #[vvm(revision = 1, stimulus = S, observation = O)]
                    struct Model { field: Coverpoint<u8> }
                })
                .expect("derive input parses"),
                "missing `definition` option",
            ),
            (
                parse2(quote! {
                    #[vvm(definition = "Model", revision = 1, stimulus = S, observation = O)]
                    struct Model { field: Coverpoint<u8> }
                })
                .expect("derive input parses"),
                "invalid coverage definition name",
            ),
            (
                parse2(quote! {
                    #[vvm(definition = "model", revision = 1, stimulus = S, observation = O)]
                    struct Model;
                })
                .expect("derive input parses"),
                "Coverage supports only named-field structs",
            ),
            (
                parse2(quote! {
                    #[vvm(definition = "model", revision = 1, stimulus = S, observation = O)]
                    struct Model<T> { field: T }
                })
                .expect("derive input parses"),
                "generic coverage structs are not supported",
            ),
            (
                parse2(quote! {
                    #[vvm(definition = "model", revision = 1, stimulus = S, observation = O)]
                    struct Model { field: Coverpoint<u8> }
                })
                .expect("derive input parses"),
                "coverage fields require an explicit role",
            ),
            (
                parse2(quote! {
                    #[vvm(definition = "model", revision = 1, stimulus = S, observation = O)]
                    struct Model {
                        #[vvm(coverpoint(build = build, sample = sample))]
                        point: u8,
                    }
                })
                .expect("derive input parses"),
                "coverpoint fields must have type `Coverpoint<T>`",
            ),
            (
                parse2(quote! {
                    #[vvm(definition = "model", revision = 1, stimulus = S, observation = O)]
                    struct Model {
                        #[vvm(coverpoint(build = build, sample = sample))]
                        point: Coverpoint<u8>,
                        #[vvm(cross(left = point, right = missing))]
                        cross: Cross2,
                    }
                })
                .expect("derive input parses"),
                "cross sources must name previously declared coverpoint fields",
            ),
        ];

        for (input, expected) in cases {
            let result = Input::parse(input);

            assert_eq!(
                result.as_ref().err().map(ToString::to_string).as_deref(),
                Some(expected)
            );
        }
    }

    #[test]
    fn rejects_role_options_with_specific_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let inputs = [
            (
                quote!(#[vvm(coverpoint(build = build))] point: Coverpoint<u8>),
                "coverpoint requires `sample`",
            ),
            (
                quote!(#[vvm(cross(left = point, right = point, build = a, build = b))] cross: Cross2),
                "duplicate `build` option",
            ),
        ];

        for (field, expected) in inputs {
            let input: DeriveInput = parse2(quote! {
                #[vvm(definition = "model", revision = 1, stimulus = S, observation = O)]
                struct Model {
                    #[vvm(coverpoint(build = build, sample = sample))]
                    point: Coverpoint<u8>,
                    #field
                }
            })?;
            let result = Input::parse(input);

            assert_eq!(
                result.as_ref().err().map(ToString::to_string).as_deref(),
                Some(expected)
            );
        }

        Ok(())
    }

    #[test]
    fn rejects_remaining_group_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let cases = [
            (
                quote!(
                    #[vvm(definition = "model", stimulus = S, observation = O)]
                    struct Model {
                        #[vvm(coverpoint(build = build, sample = sample))]
                        point: Coverpoint<u8>,
                    }
                ),
                "missing `revision` option",
            ),
            (
                quote!(
                    #[vvm(definition = "model", revision = 1, observation = O)]
                    struct Model {
                        #[vvm(coverpoint(build = build, sample = sample))]
                        point: Coverpoint<u8>,
                    }
                ),
                "missing `stimulus` option",
            ),
            (
                quote!(
                    #[vvm(definition = "model", revision = 1, stimulus = S)]
                    struct Model {
                        #[vvm(coverpoint(build = build, sample = sample))]
                        point: Coverpoint<u8>,
                    }
                ),
                "missing `observation` option",
            ),
            (
                quote!(
                    #[vvm(definition = "model", definition = "other", revision = 1, stimulus = S, observation = O)]
                    struct Model {
                        #[vvm(coverpoint(build = build, sample = sample))]
                        point: Coverpoint<u8>,
                    }
                ),
                "duplicate `definition` option",
            ),
            (
                quote!(
                    #[vvm(definition = "model", revision = 1, stimulus = S, observation = O, clock = C)]
                    struct Model {
                        #[vvm(coverpoint(build = build, sample = sample))]
                        point: Coverpoint<u8>,
                    }
                ),
                "unsupported Coverage option: expected `definition`, `revision`, `stimulus`, or \
                 `observation`",
            ),
            (
                quote!(
                    #[vvm(definition = "model", revision = 1, stimulus = S, observation = O)]
                    struct Model {}
                ),
                "Coverage structs must contain at least one field",
            ),
        ];

        assert_diagnostics(cases)
    }

    #[test]
    fn rejects_coverage_field_order_and_types() -> Result<(), Box<dyn std::error::Error>> {
        let cases = [
            (
                quote!(
                    #[vvm(definition = "model", revision = 1, stimulus = S, observation = O)]
                    struct Model {
                        #[vvm(cross(left = point, right = point))]
                        cross: Cross2,
                        #[vvm(coverpoint(build = build, sample = sample))]
                        point: Coverpoint<u8>,
                    }
                ),
                "cross sources must name previously declared coverpoint fields",
            ),
            (
                quote!(
                    #[vvm(definition = "model", revision = 1, stimulus = S, observation = O)]
                    struct Model {
                        #[vvm(coverpoint(build = build, sample = sample))]
                        first: Coverpoint<u8>,
                        #[vvm(cross(left = first, right = first))]
                        cross: Cross2,
                        #[vvm(coverpoint(build = build, sample = sample))]
                        second: Coverpoint<u8>,
                    }
                ),
                "coverpoint fields must appear before cross fields",
            ),
            (
                quote!(
                    #[vvm(definition = "model", revision = 1, stimulus = S, observation = O)]
                    struct Model {
                        #[vvm(role)]
                        point: Coverpoint<u8>,
                    }
                ),
                "coverage fields require `coverpoint(...)` or `cross(...)`",
            ),
            (
                quote!(
                    #[vvm(definition = "model", revision = 1, stimulus = S, observation = O)]
                    struct Model {
                        #[vvm(coverpoint(sample = sample))]
                        point: Coverpoint<u8>,
                    }
                ),
                "coverpoint requires `build`",
            ),
            (
                quote!(
                    #[vvm(definition = "model", revision = 1, stimulus = S, observation = O)]
                    struct Model {
                        #[vvm(coverpoint(build = build, sample = sample))]
                        point: Coverpoint<u8>,
                        #[vvm(cross(left = point, right = point))]
                        cross: u8,
                    }
                ),
                "cross fields must have type `Cross2`",
            ),
        ];

        assert_diagnostics(cases)
    }

    #[test]
    fn rejects_remaining_coverage_role_options() -> Result<(), Box<dyn std::error::Error>> {
        let cases = [
            (
                quote!(
                    #[vvm(definition = "model", revision = 1, stimulus = S, observation = O)]
                    struct Model {
                        #[vvm(coverpoint(build = build, sample = sample, unknown = x))]
                        point: Coverpoint<u8>,
                    }
                ),
                "unsupported coverpoint option: expected `build` or `sample`",
            ),
            (
                quote!(
                    #[vvm(definition = "model", revision = 1, stimulus = S, observation = O)]
                    struct Model {
                        #[vvm(coverpoint(build = build, sample = sample), cross(left = point, right = point))]
                        point: Coverpoint<u8>,
                    }
                ),
                "duplicate coverage field role",
            ),
            (
                quote!(
                    #[vvm(definition = "model", revision = 1, stimulus = S, observation = O)]
                    struct Model {
                        #[vvm(coverpoint(build = build, sample = sample))]
                        point: Coverpoint<u8>,
                        #[vvm(cross(left = point, right = point, unknown = x))]
                        cross: Cross2,
                    }
                ),
                "unsupported cross option: expected `left`, `right`, or `build`",
            ),
            (
                quote!(
                    #[vvm(definition = "model", revision = 1, stimulus = S, observation = O)]
                    struct Model {
                        #[vvm(coverpoint(build = build, sample = sample))]
                        point: Coverpoint<u8>,
                        #[vvm(cross(right = point))]
                        cross: Cross2,
                    }
                ),
                "cross requires `left`",
            ),
            (
                quote!(
                    #[vvm(definition = "model", revision = 1, stimulus = S, observation = O)]
                    struct Model {
                        #[vvm(coverpoint(build = build, sample = sample))]
                        point: Coverpoint<u8>,
                        #[vvm(cross(left = point))]
                        cross: Cross2,
                    }
                ),
                "cross requires `right`",
            ),
        ];

        assert_diagnostics(cases)
    }

    fn assert_diagnostics(
        cases: impl IntoIterator<Item = (proc_macro2::TokenStream, &'static str)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (tokens, expected) in cases {
            let input: DeriveInput = parse2(tokens)?;
            let result = Input::parse(input);

            assert_eq!(
                result.as_ref().err().map(ToString::to_string).as_deref(),
                Some(expected)
            );
        }

        Ok(())
    }
}
