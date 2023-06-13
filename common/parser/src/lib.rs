#[cfg(test)]
#[macro_use]
extern crate assert_matches;

use std::collections::{HashMap, HashSet};

use dynaql::registry::enums::DynaqlEnums;
use dynaql::registry::scalars::{PossibleScalar, SDLDefinitionScalar};
use dynaql::Pos;
use dynaql_parser::types::ServiceDocument;
use dynaql_parser::{parse_schema, Error as ParserError};
use rules::auth_directive::AuthDirective;
use rules::basic_type::BasicType;
use rules::check_field_lowercase::CheckFieldCamelCase;
use rules::check_known_directives::CheckAllDirectivesAreKnown;
use rules::check_type_collision::CheckTypeCollision;
use rules::check_type_validity::CheckTypeValidity;
use rules::check_types_underscore::CheckBeginsWithDoubleUnderscore;
use rules::default_directive::DefaultDirective;
use rules::default_directive_types::DefaultDirectiveTypes;
use rules::directive::Directives;
use rules::enum_type::EnumType;
use rules::extend_connector_types::ExtendConnectorTypes;
use rules::extend_query_and_mutation_types::ExtendQueryAndMutationTypes;
use rules::graphql_directive::GraphqlVisitor;
use rules::input_object::InputObjectVisitor;
use rules::length_directive::LengthDirective;
use rules::model_directive::ModelDirective;
use rules::one_of_directive::OneOfDirective;
use rules::openapi_directive::OpenApiVisitor;
use rules::relations::{relations_rules, RelationEngine};
use rules::resolver_directive::ResolverDirective;
use rules::search_directive::SearchDirective;
use rules::unique_directive::UniqueDirective;
use rules::unique_fields::UniqueObjectFields;
use rules::visitor::{visit, RuleError, Visitor, VisitorContext};

mod models;

use crate::rules::cache_directive::visitor::CacheVisitor;
use crate::rules::cache_directive::CacheDirective;
pub use connector_parsers::ConnectorParsers;
pub use dynaql::registry::Registry;
pub use migration_detection::{required_migrations, RequiredMigration};
pub use rules::cache_directive::global::{GlobalCacheRules, GlobalCacheTarget};
pub use rules::graphql_directive::GraphqlDirective;
pub use rules::openapi_directive::{OpenApiDirective, OpenApiQueryNamingStrategy, OpenApiTransforms};

use crate::rules::scalar_hydratation::ScalarHydratation;

pub mod connector_parsers;

mod directive_de;
mod dynamic_string;
mod migration_detection;
mod registry;
mod rules;
#[cfg(test)]
mod tests;
mod utils;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Parser(
        #[from]
        #[source]
        ParserError,
    ),
    #[error("{0:?}")]
    Validation(Vec<RuleError>),
    #[error("Errors parsing {} connector: \n\n{}", .0.as_deref().unwrap_or("unnamed"), .1.join("\n"))]
    ConnectorErrors(Option<String>, Vec<String>, Pos),
}

impl From<Vec<RuleError>> for Error {
    fn from(value: Vec<RuleError>) -> Self {
        Error::Validation(value)
    }
}

impl Error {
    #[cfg(test)]
    fn validation_errors(self) -> Option<Vec<RuleError>> {
        if let Error::Validation(err) = self {
            Some(err)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct ParseResult<'a> {
    pub registry: Registry,
    pub required_resolvers: HashSet<String>,
    pub global_cache_rules: GlobalCacheRules<'a>,
}

/// Transform the input into a Registry
pub async fn parse<'a>(
    schema: &'a str,
    variables: &HashMap<String, String>,
    connector_parsers: &dyn ConnectorParsers,
) -> Result<ParseResult<'a>, Error> {
    let directives = Directives::new()
        .with::<AuthDirective>()
        .with::<DefaultDirective>()
        .with::<LengthDirective>()
        .with::<ModelDirective>()
        .with::<OneOfDirective>()
        .with::<RelationEngine>()
        .with::<ResolverDirective>()
        .with::<UniqueDirective>()
        .with::<SearchDirective>()
        .with::<OpenApiDirective>()
        .with::<GraphqlDirective>()
        .with::<CacheDirective>();

    let schema = format!(
        "{}\n{}\n{}\n{}",
        schema,
        DynaqlEnums::sdl(),
        PossibleScalar::sdl(),
        directives.to_definition(),
    );
    let schema = parse_schema(schema)?;

    let mut ctx = VisitorContext::new_with_variables(&schema, variables);

    // We parse out connectors (and run their sub-parsers) first so that our schema
    // can reference types generated by those connectors
    parse_connectors(&schema, &mut ctx, connector_parsers).await?;

    // Building all relations first are it requires to parse the whole schema (for ManyToMany). This allows later
    // rules to rely on RelationEngine::get to have correct information on relations.
    parse_relations(&schema, &mut ctx);
    if !ctx.errors.is_empty() {
        return Err(ctx.errors.into());
    }

    parse_types(&schema, &mut ctx);
    if !ctx.errors.is_empty() {
        return Err(ctx.errors.into());
    }

    Ok(ctx.finish())
}

async fn parse_connectors<'a>(
    schema: &'a ServiceDocument,
    ctx: &mut VisitorContext<'a>,
    connector_parsers: &dyn ConnectorParsers,
) -> Result<(), Error> {
    let mut connector_rules = rules::visitor::VisitorNil.with(OpenApiVisitor).with(GraphqlVisitor);

    visit(&mut connector_rules, ctx, schema);

    // We could probably parallelise this, but the schemas and the associated
    // processing use a reasonable amount of memory so going to keep it sequential
    for (directive, position) in std::mem::take(&mut ctx.openapi_directives) {
        let directive_name = directive.name.clone();
        match connector_parsers.fetch_and_parse_openapi(directive).await {
            Ok(registry) => {
                connector_parsers::merge_registry(ctx, registry, position);
            }
            Err(errors) => return Err(Error::ConnectorErrors(Some(directive_name), errors, position)),
        }
    }

    for (mut directive, position) in std::mem::take(&mut ctx.graphql_directives) {
        directive.id = Some(ctx.connector_id_generator.new_id());
        let directive_name = directive.name.clone();
        match connector_parsers.fetch_and_parse_graphql(directive).await {
            Ok(registry) => {
                connector_parsers::merge_registry(ctx, registry, position);
            }
            Err(errors) => return Err(Error::ConnectorErrors(directive_name, errors, position)),
        }
    }

    Ok(())
}

fn parse_relations<'a>(schema: &'a ServiceDocument, ctx: &mut VisitorContext<'a>) {
    visit(&mut relations_rules().with(CheckTypeCollision::default()), ctx, schema);
}

fn parse_types<'a>(schema: &'a ServiceDocument, ctx: &mut VisitorContext<'a>) {
    let mut rules = rules::visitor::VisitorNil
        .with(CheckBeginsWithDoubleUnderscore)
        .with(CheckFieldCamelCase)
        .with(CheckTypeValidity)
        .with(SearchDirective)
        .with(ModelDirective)
        .with(AuthDirective)
        .with(ResolverDirective)
        .with(CacheVisitor)
        .with(InputObjectVisitor)
        .with(BasicType)
        .with(ExtendQueryAndMutationTypes)
        .with(ExtendConnectorTypes)
        .with(EnumType)
        .with(ScalarHydratation)
        .with(LengthDirective)
        .with(UniqueObjectFields)
        .with(CheckAllDirectivesAreKnown::default());

    visit(&mut rules, ctx, schema);

    // FIXME: Get rid of the ugly double pass.
    let mut second_pass_rules = rules::visitor::VisitorNil.with(DefaultDirectiveTypes);
    visit(&mut second_pass_rules, ctx, schema);
}

pub fn parse_registry<S: AsRef<str>>(input: S) -> Result<Registry, Error> {
    let input = input.as_ref();
    Ok(futures::executor::block_on(async move {
        let variables = HashMap::new();
        let connector_parsers = connector_parsers::MockConnectorParsers::default();
        parse(input, &variables, &connector_parsers).await
    })?
    .registry)
}

#[cfg(test)]
fn to_parse_result_with_variables<'a>(
    input: &'a str,
    variables: &HashMap<String, String>,
) -> Result<ParseResult<'a>, Error> {
    futures::executor::block_on(async move {
        let connector_parsers = connector_parsers::MockConnectorParsers::default();
        parse(input, variables, &connector_parsers).await
    })
}
