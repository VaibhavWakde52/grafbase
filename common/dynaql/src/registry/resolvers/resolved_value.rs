use std::{borrow::Borrow, sync::Arc};

use internment::ArcIntern;
use serde_json::Value;

use super::ResolvedPaginationInfo;

/// ResolvedValue are values passed arround between resolvers, it contains the actual Resolver data
/// but will also contain other informations wich may be use later by custom resolvers, like for
/// example Pagination Details.
///
/// Cheap to Clone and take sub-copies of.
#[derive(Debug, Clone)]
pub struct ResolvedValue {
    /// The root of the JSON blob that contains this ResolvedValue.
    ///
    /// The data is sent as-is to the next resolver in the chain. The format of the data is
    /// dependent on the resolver that produced the data.
    ///
    /// For example, the GraphQL resolver returns data in the actual shape of the query. That is, a
    /// resolver that resolves a `user { name }` query, is expected to return a `{ "user": { "name"
    /// "..." } }` JSON object.
    ///
    /// Other resolvers might transform/augment the data before passing it along.
    data_root: Arc<Value>,
    /// The path to this ResolvedValue inside data_root.
    ///
    /// This allows us to take a sub-copy of a ResolvedValue without having to clone the entire
    /// associated serde_json::Value.
    data_path: Vec<PathSegment>,
    /// Optional pagination data for Paginated Resolvers
    pub pagination: Option<ResolvedPaginationInfo>,
    /// Resolvers can set this value when resolving so the engine will know it's
    /// not usefull to continue iterating over the ResolverChain.
    pub early_return_null: bool,
}

impl Borrow<Value> for &ResolvedValue {
    fn borrow(&self) -> &Value {
        self.data_resolved()
    }
}

impl ResolvedValue {
    pub fn new(value: Value) -> Self {
        Self {
            data_root: Arc::new(value),
            data_path: vec![],
            pagination: None,
            early_return_null: false,
        }
    }

    pub fn null() -> Self {
        Self::new(Value::Null)
    }

    pub fn with_pagination(mut self, pagination: ResolvedPaginationInfo) -> Self {
        self.pagination = Some(pagination);
        self
    }

    pub fn with_early_return(mut self) -> Self {
        self.early_return_null = true;
        self
    }

    /// We can check from the schema definition if it's a node, if it is, we need to
    /// have a way to get it
    /// temp: Little hack here, we know that `ResolvedValue` are bound to have a format
    /// of:
    /// ```ignore
    /// {
    ///   "Node": {
    ///     "__sk": {
    ///       "S": "node_id"
    ///     }
    ///   }
    /// }
    /// ```
    /// We use that fact without checking it here.
    ///
    /// This have to be removed when we rework registry & dynaql to have a proper query
    /// planning.
    pub fn node_id<S: AsRef<str>>(&self, entity: S) -> Option<String> {
        self.data_resolved().get(entity.as_ref()).and_then(|x| {
            x.get("__sk")
                .and_then(|x| {
                    if let Value::Object(value) = x {
                        Some(value)
                    } else {
                        None
                    }
                })
                .and_then(|x| x.get("S"))
                .and_then(|value| {
                    if let Value::String(value) = value {
                        Some(value.clone())
                    } else {
                        None
                    }
                })
        })
    }

    pub fn is_early_returned(&self) -> bool {
        self.early_return_null
    }

    pub fn data_resolved(&self) -> &Value {
        self.data_path.iter().fold(self.data_root.as_ref(), |value, index| {
            match index {
                PathSegment::Field(field) => value.get(field.as_str()),
                PathSegment::Index(index) => value.get(*index),
            }
            .expect("data_path to be validated before ResolvedValue construction")
        })
    }

    /// Returns a new ResolvedValue pointing at the given index, assuming this is a list and index exists.
    pub fn get_index(&self, index: usize) -> Option<ResolvedValue> {
        self.data_resolved().get(index)?;

        let mut data_path = self.data_path.clone();
        data_path.push(PathSegment::Index(index));
        Some(ResolvedValue {
            data_root: Arc::clone(&self.data_root),
            data_path,
            early_return_null: false,
            pagination: None,
        })
    }

    /// Returns a new ResolvedValue pointing at the given field, assuming this is an object and field exists.
    pub fn get_field(&self, name: &str) -> Option<ResolvedValue> {
        self.data_resolved().get(name)?;

        let mut data_path = self.data_path.clone();
        data_path.push(PathSegment::Field(ArcIntern::from_ref(name)));
        Some(ResolvedValue {
            data_root: Arc::clone(&self.data_root),
            data_path,
            early_return_null: false,
            pagination: None,
        })
    }

    /// Takes the inner value.
    ///
    /// If possible this will avoid cloning, but if we're not the sole owner of data_root it'll clone.
    pub fn take(mut self) -> Value {
        match Arc::try_unwrap(self.data_root) {
            Ok(value) => self.data_path.iter().fold(value, |mut value, index| match index {
                PathSegment::Field(field) => value.get_mut(field.as_str()).expect("data_path to be validated").take(),
                PathSegment::Index(index) => value.get_mut(*index).expect("data_path to be validated").take(),
            }),
            Err(arc) => {
                self.data_root = arc;
                self.data_resolved().clone()
            }
        }
    }

    /// If this ResolvedValue is an array, returns an Iterator of the items of that list
    pub fn item_iter(&self) -> Option<impl Iterator<Item = ResolvedValue> + '_> {
        match self.data_resolved() {
            Value::Array(array) => {
                Some(array.iter().enumerate().map(|(i, _)| i).map(|index| {
                    // We don't use get_index here because it does a data_resolved lookup everytime and
                    // that'd be inefficient.
                    let mut data_path = self.data_path.clone();
                    data_path.push(PathSegment::Index(index));
                    ResolvedValue {
                        data_root: Arc::clone(&self.data_root),
                        data_path,
                        early_return_null: false,
                        pagination: None,
                    }
                }))
            }
            _ => None,
        }
    }
}

impl Default for ResolvedValue {
    fn default() -> Self {
        Self::null()
    }
}

#[derive(Debug, Clone)]
enum PathSegment {
    /// A field in an object.
    Field(ArcIntern<String>),
    /// An index in a list.
    Index(usize),
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_resolved_value_array() {
        let data = ResolvedValue::new(json!(["hello", "there"]));
        assert_eq!(data.get_index(0).unwrap().data_resolved(), &json!("hello"));
        assert_eq!(data.get_index(1).unwrap().data_resolved(), &json!("there"));
        assert!(data.get_index(2).is_none());

        assert!(data.get_field("1").is_none());

        assert_eq!(data.get_index(0).unwrap().take(), json!("hello"));

        assert_eq!(
            data.item_iter().unwrap().map(ResolvedValue::take).collect::<Vec<_>>(),
            vec![json!("hello"), json!("there")]
        );
    }

    #[test]
    fn test_resolved_value_object() {
        let data = ResolvedValue::new(json!({"a": "hello", "b": "there"}));
        assert_eq!(data.get_field("a").unwrap().data_resolved(), &json!("hello"));
        assert_eq!(data.get_field("b").unwrap().data_resolved(), &json!("there"));
        assert!(data.get_field("c").is_none());

        assert!(data.get_index(1).is_none());

        assert_eq!(data.get_field("a").unwrap().take(), json!("hello"));
    }

    #[test]
    fn test_resolved_value_scalar() {
        let data = ResolvedValue::new(json!(true));

        assert!(data.get_index(0).is_none());
        assert!(data.get_field("hello").is_none());

        assert_eq!(data.data_resolved(), &json!(true));
        assert_eq!(data.take(), json!(true));
    }
}
