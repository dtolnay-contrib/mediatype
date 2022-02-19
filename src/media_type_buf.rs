use super::{error::*, media_type::*, name::*, params::*, parse::*};
use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    str::FromStr,
};

/// An owned and immutable MediaType.
#[derive(Debug, Clone)]
pub struct MediaTypeBuf {
    data: Box<str>,
    indices: Indices,
}

impl MediaTypeBuf {
    /// Constructs a `MediaTypeBuf` from [`String`].
    ///
    /// Unlike [`FromStr::from_str`], this function takes the ownership of [`String`]
    /// instead of making a new copy.
    ///
    /// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
    /// [`FromStr::from_str`]: https://doc.rust-lang.org/std/str/trait.FromStr.html
    pub fn from_string(mut s: String) -> Result<Self, ParseError> {
        let (indices, len) = Indices::parse(&s)?;
        s.truncate(len);
        Ok(Self {
            data: s.into(),
            indices,
        })
    }

    /// Returns the top-level type.
    pub fn ty(&self) -> &str {
        &self.data[self.indices.ty()]
    }

    /// Returns the subtype.
    pub fn subty(&self) -> &str {
        &self.data[self.indices.subty()]
    }

    /// Returns the suffix.
    pub fn suffix(&self) -> Option<&str> {
        self.indices.suffix().map(|range| &self.data[range])
    }

    /// Returns the string representation without parameters.
    /// ```
    /// # use mediatype::MediaTypeBuf;
    /// # use std::str::FromStr;
    /// let media_type: MediaTypeBuf = "image/svg+xml; charset=UTF-8".parse().unwrap();
    /// assert_eq!(media_type.essence(), "image/svg+xml");
    /// ```
    pub fn essence(&self) -> &str {
        self.data.split(';').next().unwrap()
    }

    /// Returns an iterator over the parameters.
    ///
    /// The parameters are alphabetically sorted by their key.
    pub fn params(&self) -> Params {
        Params::from_indices(&self.data, &self.indices)
    }

    /// Gets a parameter value by its key.
    ///
    /// The key is case-insensitive.
    pub fn get_param(&self, key: &Name) -> Option<&str> {
        let params = self.indices.params();
        params
            .binary_search_by_key(key, |&[start, end, _, _]| {
                Name(&self.data[start as usize..end as usize])
            })
            .ok()
            .map(|index| &self.data[params[index][2] as usize..params[index][3] as usize])
    }

    /// Returns the canonicalized `MediaType`.
    ///
    /// All strings except parameter values will be converted to lowercase.
    ///
    /// ```
    /// # use mediatype::MediaTypeBuf;
    /// let media_type: MediaTypeBuf = "IMAGE/SVG+XML;  CHARSET=UTF-8;  ".parse().unwrap();
    /// assert_eq!(
    ///     media_type.canonicalize().to_string(),
    ///     "image/svg+xml; charset=UTF-8"
    /// );
    /// ```
    pub fn canonicalize(&self) -> Self {
        use std::fmt::Write;
        let mut s = String::with_capacity(self.data.len());
        write!(
            s,
            "{}/{}",
            self.ty().to_ascii_lowercase(),
            self.subty().to_ascii_lowercase()
        )
        .unwrap();
        if let Some(suffix) = self.suffix() {
            write!(s, "+{}", suffix.to_ascii_lowercase()).unwrap();
        }
        for (key, value) in self.params() {
            write!(s, "; {}={}", key.to_ascii_lowercase(), value).unwrap();
        }
        s.shrink_to_fit();
        Self::from_string(s).unwrap()
    }

    pub(crate) fn ty_name(&self) -> Name {
        Name(self.ty())
    }

    pub(crate) fn subty_name(&self) -> Name {
        Name(self.subty())
    }

    pub(crate) fn suffix_name(&self) -> Option<Name> {
        self.suffix().map(Name)
    }

    pub(crate) fn params_name(&self) -> impl Iterator<Item = (Name, Name)> {
        self.params().map(|(key, value)| (Name(key), Name(value)))
    }
}

impl FromStr for MediaTypeBuf {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (indices, len) = Indices::parse(s)?;
        Ok(Self {
            data: s[..len].into(),
            indices,
        })
    }
}

impl From<MediaType<'_>> for MediaTypeBuf {
    fn from(m: MediaType<'_>) -> Self {
        m.to_string().parse().unwrap()
    }
}

impl From<&MediaType<'_>> for MediaTypeBuf {
    fn from(m: &MediaType<'_>) -> Self {
        m.to_string().parse().unwrap()
    }
}

impl AsRef<str> for MediaTypeBuf {
    fn as_ref(&self) -> &str {
        &self.data
    }
}

impl PartialEq for MediaTypeBuf {
    fn eq(&self, other: &Self) -> bool {
        self.ty_name() == other.ty_name()
            && self.subty_name() == other.subty_name()
            && self.suffix_name() == other.suffix_name()
            && self.params_name().eq(other.params_name())
    }
}

impl Eq for MediaTypeBuf {}

impl PartialOrd for MediaTypeBuf {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MediaTypeBuf {
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

impl Hash for MediaTypeBuf {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ty_name().hash(state);
        self.subty_name().hash(state);
        self.suffix_name().hash(state);
        for param in self.params_name() {
            param.hash(state);
        }
    }
}

impl PartialEq<MediaType<'_>> for MediaTypeBuf {
    fn eq(&self, other: &MediaType) -> bool {
        self.ty_name() == other.ty_name()
            && self.subty_name() == other.subty_name()
            && self.suffix_name() == other.suffix_name()
            && self.params_name().eq(other.params_name())
    }
}

impl PartialOrd<MediaType<'_>> for MediaTypeBuf {
    fn partial_cmp(&self, other: &MediaType) -> Option<Ordering> {
        match self.ty_name().partial_cmp(&other.ty_name()) {
            Some(Ordering::Equal) => (),
            ne => return ne,
        }
        match self.subty_name().partial_cmp(&other.subty_name()) {
            Some(Ordering::Equal) => (),
            ne => return ne,
        }
        match self.suffix_name().partial_cmp(&other.suffix_name()) {
            Some(Ordering::Equal) => (),
            ne => return ne,
        }
        self.params_name().partial_cmp(other.params_name())
    }
}

impl fmt::Display for MediaTypeBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.ty(), self.subty())?;
        if let Some(suffix) = self.suffix() {
            write!(f, "+{}", suffix)?;
        }
        for (key, value) in self.params() {
            write!(f, "; {}={}", key, value)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::names::*;

    #[test]
    fn get_param() {
        assert_eq!(
            MediaTypeBuf::from_str("image/svg+xml")
                .unwrap()
                .get_param(&CHARSET),
            None
        );
        assert_eq!(
            MediaTypeBuf::from_str("image/svg+xml; charset=UTF-8")
                .unwrap()
                .get_param(&CHARSET),
            Some("UTF-8")
        );
        assert_eq!(
            MediaTypeBuf::from_str("image/svg+xml; charset=UTF-8; HELLO=WORLD")
                .unwrap()
                .get_param(&Name::new("hello").unwrap()),
            Some("WORLD")
        );
    }

    #[test]
    fn essence() {
        assert_eq!(
            MediaTypeBuf::from_str("image/svg+xml").unwrap().essence(),
            "image/svg+xml"
        );
        assert_eq!(
            MediaTypeBuf::from_str("image/svg+xml;  ")
                .unwrap()
                .essence(),
            "image/svg+xml"
        );
        assert_eq!(
            MediaTypeBuf::from_str("image/svg+xml; charset=UTF-8")
                .unwrap()
                .essence(),
            "image/svg+xml"
        );
    }

    #[test]
    fn canonicalize() {
        assert_eq!(
            MediaTypeBuf::from_str("IMAGE/SVG+XML;         CHARSET=UTF-8;     ")
                .unwrap()
                .canonicalize()
                .to_string(),
            "image/svg+xml; charset=UTF-8"
        );
    }

    #[test]
    fn cmp() {
        assert_eq!(
            MediaTypeBuf::from_str("text/plain").unwrap(),
            MediaTypeBuf::from_str("TEXT/PLAIN").unwrap()
        );
        assert_eq!(
            MediaTypeBuf::from_str("image/svg+xml; charset=UTF-8").unwrap(),
            MediaTypeBuf::from_str("IMAGE/SVG+XML; CHARSET=utf-8").unwrap()
        );
        assert_eq!(
            MediaTypeBuf::from_str("image/svg+xml; hello=world; charset=UTF-8").unwrap(),
            MediaTypeBuf::from_str("IMAGE/SVG+XML; CHARSET=utf-8; HELLO=WORLD").unwrap()
        );
    }
}
