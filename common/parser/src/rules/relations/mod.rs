use super::visitor::{VisitorCons, VisitorNil};

mod relations_engine;
pub use relations_engine::generate_metarelation;

pub const fn relations_rules() -> VisitorCons<relations_engine::RelationEngine, VisitorNil> {
    // TODO: Add Check to ensure the directive is not used outside of Modelized node.
    VisitorNil.with(relations_engine::RelationEngine)
}

#[cfg(test)]
mod tests {
    use super::relations_rules;
    use crate::rules::visitor::{visit, VisitorContext};
    use dynaql_parser::parse_schema;
    use insta::assert_debug_snapshot;
    use serde_json as _;

    #[test]
    fn multiple_relations() {
        let schema = r#"
            type Author @model {
                id: ID!
                postsToday: [Post!]
                postsYesterday: [Post!]
            }

            type Post @model {
                id: ID!
                publishedBy: [Author!]
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut relations_rules(), &mut ctx, &schema);

        assert!(!ctx.errors.is_empty(), "shouldn't be empty");
        assert_debug_snapshot!(ctx.errors);
    }

    #[test]
    fn multiple_relations_directive_not_defined() {
        let schema = r#"
            type Author @model {
                id: ID!
                postsToday: [Post!] @relation(name: "postsToday")
                postsYesterday: [Post!] @relation(name: "postsYesterday")
                posts: [Post!] @relation(name: "published")
            }

            type Post @model {
                id: ID!
                publishedBy: [Author!] @relation(name: "published")
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut relations_rules(), &mut ctx, &schema);

        assert_debug_snapshot!(ctx.relations);
        assert!(ctx.errors.is_empty(), "should be empty");
    }
}
