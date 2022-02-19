use super::{error::*, name::*, params::*, parse::*, value::*};
use std::{
    borrow::Cow,
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
};

/// A parsed MediaType.
#[derive(Debug, Clone)]
pub struct MediaType<'a> {
    ty: Name<'a>,
    subty: Name<'a>,
    suffix: Option<Name<'a>>,
    params: Cow<'a, [(Name<'a>, Value<'a>)]>,
}

impl<'a> MediaType<'a> {
    /// Constructs a `MediaType` from a top-level type and a subtype.
    /// ```
    /// # use mediatype::{names::*, MediaType};
    /// const IMAGE_PNG: MediaType = MediaType::new(IMAGE, PNG);
    /// assert_eq!(IMAGE_PNG, MediaType::parse("image/png").unwrap());
    /// ```
    pub const fn new(ty: Name<'a>, subty: Name<'a>) -> Self {
        Self {
            ty,
            subty,
            suffix: None,
            params: Cow::Borrowed(&[]),
        }
    }

    /// Constructs a `MediaType` with an optional suffix and parameters.
    ///
    /// This function accepts only one parameter
    /// because key duplication is not detectable at compile-time.
    ///
    /// ```
    /// # use mediatype::{names::*, values::*, MediaType};
    /// const IMAGE_SVG: MediaType =
    ///     MediaType::from_parts(IMAGE, SVG, Some(XML), Some(&[(CHARSET, UTF_8)]));
    /// assert_eq!(
    ///     IMAGE_SVG,
    ///     MediaType::parse("image/svg+xml; charset=UTF-8").unwrap()
    /// );
    /// ```
    pub const fn from_parts(
        ty: Name<'a>,
        subty: Name<'a>,
        suffix: Option<Name<'a>>,
        param: Option<&'a [(Name<'a>, Value<'a>); 1]>,
    ) -> Self {
        let params: &[(Name, Value)] = if let Some(param) = param { param } else { &[] };
        Self {
            ty,
            subty,
            suffix,
            params: Cow::Borrowed(params),
        }
    }

    /// Constructs a `MediaType` from `str`.
    pub fn parse<'s: 'a>(s: &'s str) -> Result<Self, ParseError> {
        let (indices, _) = Indices::parse(s)?;
        let params = indices
            .params()
            .iter()
            .map(|param| {
                (
                    Name(&s[param[0] as usize..param[1] as usize]),
                    Value(&s[param[2] as usize..param[3] as usize]),
                )
            })
            .collect();
        Ok(Self {
            ty: Name(&s[indices.ty()]),
            subty: Name(&s[indices.subty()]),
            suffix: indices.suffix().map(|range| Name(&s[range])),
            params: Cow::Owned(params),
        })
    }

    /// Returns the top-level type.
    pub const fn ty(&self) -> &'a str {
        self.ty.0
    }

    /// Returns the subtype.
    pub const fn subty(&self) -> &'a str {
        self.subty.0
    }

    /// Returns the suffix.
    pub fn suffix(&self) -> Option<&str> {
        self.suffix.map(|x| x.0)
    }

    /// Sets the top-level type.
    pub fn set_ty<'t: 'a>(&mut self, ty: &'t Name) {
        self.ty = *ty;
    }

    /// Sets the subtype.
    pub fn set_subty<'t: 'a>(&mut self, subty: &'t Name) {
        self.subty = *subty;
    }

    /// Sets the suffix.
    pub fn set_suffix<'s: 'a>(&mut self, suffix: &'s Option<Name>) {
        self.suffix = *suffix;
    }

    /// Returns an iterator over the parameters.
    ///
    /// The parameters are alphabetically sorted by their key.
    pub fn params(&self) -> Params {
        Params::from_slice(&self.params)
    }

    /// Gets a parameter value by its key.
    ///
    /// The key is case-insensitive.
    pub fn get_param(&self, key: &Name) -> Option<&str> {
        self.params
            .binary_search_by_key(key, |(key, _)| *key)
            .ok()
            .map(|index| self.params[index].1.as_ref())
    }

    /// Sets a parameter value.
    ///
    /// If the parameter is already set, replaces it with a new value and
    /// returns the old value.
    /// The key is case-insensitive.
    pub fn set_param<'k: 'a, 'v: 'a>(&mut self, key: &'k Name, value: &'v Value) -> Option<&str> {
        if let Ok(index) = self
            .params
            .binary_search_by_key(&Name(key.as_ref()), |(key, _)| *key)
        {
            Some(std::mem::replace(&mut self.params.to_mut()[index].1, *value).0)
        } else {
            let params = self.params.to_mut();
            params.push((*key, *value));
            params.sort_unstable_by_key(|&(key, _)| key);
            None
        }
    }

    /// Removes and returns a parameter value by its key.
    ///
    /// The key is case-insensitive.
    pub fn remove_param(&mut self, key: &Name) -> Option<&str> {
        self.params
            .binary_search_by_key(key, |(key, _)| *key)
            .ok()
            .map(|index| self.params.to_mut().remove(index).1 .0)
    }

    pub(crate) fn ty_name(&self) -> Name<'a> {
        self.ty
    }

    pub(crate) fn subty_name(&self) -> Name<'a> {
        self.subty
    }

    pub(crate) fn suffix_name(&self) -> Option<Name<'a>> {
        self.suffix
    }

    pub(crate) fn params_name(&self) -> impl Iterator<Item = (Name, Name)> {
        self.params().map(|(key, value)| (Name(key), Name(value)))
    }
}

impl<'a> fmt::Display for MediaType<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.ty, self.subty)?;
        if let Some(suffix) = self.suffix {
            write!(f, "+{}", suffix)?;
        }
        for (key, value) in &*self.params {
            write!(f, "; {}={}", key, value)?;
        }
        Ok(())
    }
}

impl<'a> PartialEq for MediaType<'a> {
    fn eq(&self, other: &MediaType) -> bool {
        self.ty_name() == other.ty_name()
            && self.subty_name() == other.subty_name()
            && self.suffix_name() == other.suffix_name()
            && self.params_name().eq(other.params_name())
    }
}

impl<'a> Eq for MediaType<'a> {}

impl<'a> PartialOrd for MediaType<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for MediaType<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.ty_name().cmp(&other.ty_name()) {
            Ordering::Equal => (),
            ne => return ne,
        }
        match self.subty_name().cmp(&other.subty_name()) {
            Ordering::Equal => (),
            ne => return ne,
        }
        match self.suffix_name().cmp(&other.suffix_name()) {
            Ordering::Equal => (),
            ne => return ne,
        }
        self.params_name().cmp(other.params_name())
    }
}

impl<'a> Hash for MediaType<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ty_name().hash(state);
        self.subty_name().hash(state);
        self.suffix_name().hash(state);
        for param in self.params_name() {
            param.hash(state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{names::*, values::*};

    #[test]
    fn to_string() {
        assert_eq!(MediaType::new(TEXT, PLAIN).to_string(), "text/plain");
        assert_eq!(
            MediaType::from_parts(IMAGE, SVG, Some(XML), None).to_string(),
            "image/svg+xml"
        );
        assert_eq!(
            MediaType::from_parts(TEXT, PLAIN, None, Some(&[(CHARSET, UTF_8)])).to_string(),
            "text/plain; charset=UTF-8"
        );
        assert_eq!(
            MediaType::from_parts(IMAGE, SVG, Some(XML), Some(&[(CHARSET, UTF_8)])).to_string(),
            "image/svg+xml; charset=UTF-8"
        );
    }

    #[test]
    fn set_ty() {
        let mut media_type = MediaType::from_parts(TEXT, PLAIN, None, Some(&[(CHARSET, UTF_8)]));
        let upper_text = Name::new("TEXT").unwrap();
        media_type.set_ty(&upper_text);
        assert_eq!(media_type.to_string(), "TEXT/plain; charset=UTF-8");
    }

    #[test]
    fn set_subty() {
        let mut media_type = MediaType::from_parts(TEXT, PLAIN, None, Some(&[(CHARSET, UTF_8)]));
        let markdown = Name::new("markdown").unwrap();
        media_type.set_subty(&markdown);
        assert_eq!(media_type.to_string(), "text/markdown; charset=UTF-8");
    }

    #[test]
    fn set_suffix() {
        let mut media_type = MediaType::from_parts(IMAGE, SVG, None, Some(&[(CHARSET, UTF_8)]));
        media_type.set_suffix(&Some(XML));
        assert_eq!(media_type.to_string(), "image/svg+xml; charset=UTF-8");
    }

    #[test]
    fn get_param() {
        assert_eq!(MediaType::new(TEXT, PLAIN).get_param(&CHARSET), None);
        assert_eq!(
            MediaType::from_parts(TEXT, PLAIN, None, Some(&[(CHARSET, UTF_8)])).get_param(&CHARSET),
            Some("UTF-8")
        );
        assert_eq!(
            MediaType::parse("image/svg+xml; charset=UTF-8; HELLO=WORLD")
                .unwrap()
                .get_param(&Name::new("hello").unwrap()),
            Some("WORLD")
        );
    }

    #[test]
    fn set_param() {
        let mut media_type = MediaType::from_parts(TEXT, PLAIN, None, Some(&[(CHARSET, UTF_8)]));
        let lower_utf8 = Value::new("utf-8").unwrap();
        assert_eq!(media_type.set_param(&CHARSET, &lower_utf8), Some("UTF-8"));
        assert_eq!(media_type.to_string(), "text/plain; charset=utf-8");

        let alice = Name::new("ALICE").unwrap();
        let bob = Value::new("bob").unwrap();
        assert_eq!(media_type.set_param(&alice, &bob), None);
        assert_eq!(
            media_type.to_string(),
            "text/plain; ALICE=bob; charset=utf-8"
        );
    }

    #[test]
    fn remove_param() {
        assert_eq!(MediaType::new(TEXT, PLAIN).remove_param(&CHARSET), None);

        let mut media_type = MediaType::from_parts(TEXT, PLAIN, None, Some(&[(CHARSET, UTF_8)]));
        assert_eq!(media_type.remove_param(&CHARSET), Some("UTF-8"));
        assert_eq!(media_type.remove_param(&CHARSET), None);
        assert_eq!(media_type.to_string(), "text/plain");

        let mut media_type = MediaType::parse("image/svg+xml; charset=UTF-8; HELLO=WORLD").unwrap();
        assert_eq!(
            media_type.remove_param(&Name::new("hello").unwrap()),
            Some("WORLD")
        );
        assert_eq!(media_type.remove_param(&Name::new("hello").unwrap()), None);
        assert_eq!(media_type.to_string(), "image/svg+xml; charset=UTF-8");
    }

    #[test]
    fn cmp() {
        assert_eq!(
            MediaType::parse("text/plain").unwrap(),
            MediaType::parse("TEXT/PLAIN").unwrap()
        );
        assert_eq!(
            MediaType::parse("image/svg+xml; charset=UTF-8").unwrap(),
            MediaType::parse("IMAGE/SVG+XML; CHARSET=utf-8").unwrap()
        );
        assert_eq!(
            MediaType::parse("image/svg+xml; hello=world; charset=UTF-8").unwrap(),
            MediaType::parse("IMAGE/SVG+XML; CHARSET=utf-8; HELLO=WORLD").unwrap()
        );
    }
}
