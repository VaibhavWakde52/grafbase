#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct StringId(usize);

impl StringId {
    pub(crate) const MIN: StringId = StringId(usize::MIN);
    pub(crate) const MAX: StringId = StringId(usize::MAX);
}

/// Very simple implementation of string interning.
#[derive(Default)]
pub(crate) struct Strings(indexmap::IndexSet<Box<str>>);

impl Strings {
    /// Interns a string-like value, avoiding allocation if the same string has already been
    /// interned.
    pub(crate) fn intern<T>(&mut self, string: T) -> StringId
    where
        T: AsRef<str> + Into<Box<str>>,
    {
        if let Some(idx) = self.0.get_index_of(string.as_ref()) {
            return StringId(idx);
        }

        StringId(self.0.insert_full(string.into()).0)
    }

    /// Resolve a [StringId] into an `&str`. The unwrap is safe because we only issue [StringId]s
    /// here. The only plausible error cases is if you pass in a [StringId] produced by another
    /// [Strings] instance.
    pub(crate) fn resolve(&self, id: StringId) -> &str {
        self.0.get_index(id.0).unwrap().as_ref()
    }
}

// Sugar for [Strings::resolve()].
impl std::ops::Index<StringId> for Strings {
    type Output = str;

    fn index(&self, index: StringId) -> &Self::Output {
        self.resolve(index)
    }
}
