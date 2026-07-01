use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Attribute, Ident, Path, Result, Token, Visibility, bracketed};

/// One function path in an explicit test registry.
pub(super) struct Entry {
    /// Attributes applied to the generated array entry.
    pub(super) attributes: Vec<Attribute>,

    /// Path to the attributed test function.
    pub(super) path: Path,
}

impl Parse for Entry {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            attributes: Attribute::parse_outer(input)?,
            path: input.parse()?,
        })
    }
}

/// Parsed `test_registry!` declaration.
pub(super) struct Input {
    /// Attributes applied to the generated static.
    pub(super) attributes: Vec<Attribute>,

    /// Registry visibility.
    pub(super) visibility: Visibility,

    /// Registry static identifier.
    pub(super) ident: Ident,

    /// Ordered test entries.
    pub(super) entries: Vec<Entry>,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attributes = Attribute::parse_outer(input)?;
        let visibility = input.parse()?;
        input.parse::<Token![static]>()?;
        let ident = input.parse()?;
        input.parse::<Token![=]>()?;

        let content;
        bracketed!(content in input);
        let entries = Punctuated::<Entry, Token![,]>::parse_terminated(&content)?
            .into_iter()
            .collect();

        if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
        }

        if !input.is_empty() {
            return Err(input.error("unexpected tokens after test registry declaration"));
        }

        Ok(Self {
            attributes,
            visibility,
            ident,
            entries,
        })
    }
}
