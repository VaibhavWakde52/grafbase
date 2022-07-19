use std::collections::HashSet;

use super::visitor::{Visitor, VisitorContext};

use dynaql::ServerError;
use dynaql_parser::types::ConstDirective;
use dynaql_value::ConstValue;

use serde::{Deserialize, Serialize};

pub const AUTH_DIRECTIVE: &str = "auth";

pub struct AuthDirective;

#[derive(Debug, Serialize, Deserialize)]
struct Auth {
    providers: Vec<AuthProvider>,
    rules: Vec<AuthRule>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
enum AuthProvider {
    #[serde(rename_all = "camelCase")]
    Oidc { issuer: url::Url },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "allow")]
#[serde(deny_unknown_fields)]
enum AuthRule {
    #[serde(rename_all = "camelCase")]
    Groups { groups: HashSet<String> },
}

impl<'a> Visitor<'a> for AuthDirective {
    // FIXME: this snippet is parsed, but not enforced by the server, why?
    fn directives(&self) -> String {
        r#"
        directive @auth(providers: [AuthProviderDefinition!]!) on SCHEMA
        input AuthProviderDefinition {
          type: String!
          issuer: String!
        }
        "#
        .to_string()
    }

    fn enter_schema(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        schema_definition: &'a dynaql::Positioned<dynaql_parser::types::SchemaDefinition>,
    ) {
        if let Some(directive) = schema_definition
            .node
            .directives
            .iter()
            .find(|d| d.node.name.node == AUTH_DIRECTIVE)
        {
            match (&directive.node).try_into() as Result<Auth, ServerError> {
                Ok(auth) => {
                    ctx.registry.get_mut().auth = Some(auth.into());
                }
                Err(err) => {
                    ctx.report_error(vec![directive.pos], err.message);
                }
            }
        }
    }
}

impl TryFrom<&ConstDirective> for Auth {
    type Error = ServerError;

    fn try_from(value: &ConstDirective) -> Result<Self, Self::Error> {
        let pos = Some(value.name.pos);

        let providers = match value.get_argument("providers") {
            Some(arg) => match &arg.node {
                ConstValue::List(value) if !value.is_empty() => value
                    .iter()
                    .map(AuthProvider::try_from)
                    .collect::<Result<_, _>>()
                    .map_err(|err| ServerError::new(err.message, pos))?,
                _ => return Err(ServerError::new("auth providers must be a non-empty list", pos)),
            },
            None => Vec::new(),
        };

        let rules = match value.get_argument("rules") {
            Some(arg) => match &arg.node {
                ConstValue::List(value) if !value.is_empty() => value
                    .iter()
                    .map(AuthRule::try_from)
                    .collect::<Result<_, _>>()
                    .map_err(|err| ServerError::new(err.message, pos))?,
                _ => return Err(ServerError::new("auth rules must be a non-empty list", pos)),
            },
            None => Vec::new(),
        };

        Ok(Auth { providers, rules })
    }
}

impl TryFrom<&ConstValue> for AuthProvider {
    type Error = ServerError;

    fn try_from(value: &ConstValue) -> Result<Self, Self::Error> {
        // We convert the value to JSON to leverage serde for deserialization
        let value = match value {
            ConstValue::Object(_) => value
                .clone()
                .into_json()
                .map_err(|err| ServerError::new(err.to_string(), None))?,
            _ => return Err(ServerError::new("auth provider must be an object", None)),
        };

        let provider: AuthProvider =
            serde_json::from_value(value).map_err(|err| ServerError::new(format!("auth provider: {err}"), None))?;

        Ok(provider)
    }
}

impl TryFrom<&ConstValue> for AuthRule {
    type Error = ServerError;

    fn try_from(value: &ConstValue) -> Result<Self, Self::Error> {
        // We convert the value to JSON to leverage serde for deserialization
        let value = match value {
            ConstValue::Object(_) => value
                .clone()
                .into_json()
                .map_err(|err| ServerError::new(err.to_string(), None))?,
            _ => return Err(ServerError::new("auth rule must be an object", None)),
        };

        let rule: AuthRule =
            serde_json::from_value(value).map_err(|err| ServerError::new(format!("auth rule: {err}"), None))?;

        Ok(rule)
    }
}

impl From<Auth> for dynaql::Auth {
    fn from(auth: Auth) -> Self {
        Self {
            oidc_providers: auth
                .providers
                .iter()
                .map(|provider| match provider {
                    AuthProvider::Oidc { issuer } => dynaql::OidcProvider { issuer: issuer.clone() },
                })
                .collect(),

            allowed_groups: auth
                .rules
                .iter()
                .flat_map(|rule| -> HashSet<String> {
                    match rule {
                        AuthRule::Groups { groups } => groups.clone(),
                    }
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::visitor::visit;
    use dynaql_parser::parse_schema;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_oidc_basic() {
        let schema = r#"
            schema @auth(
              providers: [ { type: oidc, issuer: "https://my.idp.com" } ]
            ){
              query: Boolean
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut AuthDirective, &mut ctx, &schema);

        assert!(ctx.errors.is_empty());
        assert_eq!(
            ctx.registry.borrow().auth.as_ref().unwrap(),
            &dynaql::Auth {
                oidc_providers: vec![dynaql::OidcProvider {
                    issuer: url::Url::parse("https://my.idp.com").unwrap(),
                }],
                allowed_groups: HashSet::new(),
            }
        );
    }

    #[test]
    fn test_oidc_groups() {
        let schema = r#"
            schema @auth(
              providers: [ { type: oidc, issuer: "https://my.idp.com" } ],
              rules: [ { allow: groups, groups: ["admin", "user", "admin"] } ],
            ){
              query: Boolean
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut AuthDirective, &mut ctx, &schema);

        assert!(ctx.errors.is_empty());
        assert_eq!(
            ctx.registry.borrow().auth.as_ref().unwrap(),
            &dynaql::Auth {
                oidc_providers: vec![dynaql::OidcProvider {
                    issuer: url::Url::parse("https://my.idp.com").unwrap(),
                }],
                allowed_groups: vec!["admin", "user"].into_iter().map(String::from).collect(),
            }
        );
    }

    #[test]
    fn test_oidc_missing_field() {
        let schema = r#"
            schema @auth(
              providers: [ { type: oidc } ]
            ){
              query: Boolean
            }
            "#;

        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new(&schema);
        visit(&mut AuthDirective, &mut ctx, &schema);

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(
            ctx.errors.get(0).unwrap().message,
            "auth provider: missing field `issuer`",
        );
    }
}
