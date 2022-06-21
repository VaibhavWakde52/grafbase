use crate::rules::model_directive::MODEL_DIRECTIVE;
use case::CaseExt;
use dynaql::{Name, Positioned};
use dynaql_parser::types::{BaseType, FieldDefinition, Type, TypeDefinition, TypeKind};
use std::borrow::Cow;
use std::collections::HashMap;

fn is_type_primitive_internal(name: &str) -> bool {
    matches!(name, "String" | "Float" | "Boolean" | "ID" | "Int")
}

/// Check if the given type is a primitive
///
/// A Primitive type is a custom scalar which can be cast into a primitive like an i32.
///   - Int
///   - Float
///   - String
///   - Boolean
///   - ID
pub fn is_type_primitive(field: &FieldDefinition) -> bool {
    match &field.ty.node.base {
        BaseType::Named(name) => is_type_primitive_internal(name.as_ref()),
        _ => false,
    }
}

#[allow(dead_code)]
fn get_base_from_type(ty: &Type) -> &str {
    match &ty.base {
        BaseType::Named(name) => name.as_str(),
        BaseType::List(ty_boxed) => get_base_from_type(ty_boxed.as_ref()),
    }
}

/// Check if the given type is a basic type
///
/// A BasicType is an Object and not an entity: it's not modelized.
#[allow(dead_code)]
pub fn is_type_basic_type<'a>(ctx: &HashMap<String, &'a Positioned<TypeDefinition>>, ty: &Type) -> bool {
    let ty = get_base_from_type(ty);

    let is_a_basic_type = ctx
        .get(ty)
        .map(|type_def| {
            !type_def
                .node
                .directives
                .iter()
                .any(|directive| directive.node.name.node == MODEL_DIRECTIVE)
        })
        .expect("weird");

    is_a_basic_type
}

fn to_input_base_type(ctx: &HashMap<String, Cow<'_, Positioned<TypeDefinition>>>, base_type: BaseType) -> BaseType {
    match base_type {
        BaseType::Named(name) => {
            let ty = ctx.get(name.as_ref());
            let type_def = ty.map(|x| &x.node.kind);
            let is_modelized = ty
                .map(|ty| {
                    ty.node
                        .directives
                        .iter()
                        .any(|directive| directive.node.name.node == MODEL_DIRECTIVE)
                })
                .unwrap_or(false);
            match (type_def, is_modelized) {
                (Some(TypeKind::Scalar), _) => BaseType::Named(name),
                (Some(TypeKind::Enum(_)), _) => BaseType::Named(name),
                (Some(TypeKind::Object(_)), false) => BaseType::Named(Name::new(format!("{}Input", name))),
                (Some(TypeKind::Object(_)), true) => BaseType::Named(Name::new("ID")),
                _ => BaseType::Named(Name::new("Error")),
            }
        }
        BaseType::List(list) => BaseType::List(Box::new(Type {
            base: to_input_base_type(ctx, list.base),
            nullable: list.nullable,
        })),
    }
}

fn is_modelized_node_base_type<'a>(
    ctx: &'a HashMap<String, Cow<'a, Positioned<TypeDefinition>>>,
    base_type: &'a BaseType,
) -> Option<&'a Cow<'a, Positioned<TypeDefinition>>> {
    match base_type {
        BaseType::Named(name) => {
            let ty = ctx.get(name.as_ref());
            let type_def = ty.map(|x| &x.node.kind);
            let is_modelized = ty
                .map(|ty| {
                    ty.node
                        .directives
                        .iter()
                        .any(|directive| directive.node.name.node == MODEL_DIRECTIVE)
                })
                .unwrap_or(false);
            match (type_def, is_modelized) {
                (Some(TypeKind::Object(_)), true) => ty,
                _ => None,
            }
        }
        BaseType::List(list) => is_modelized_node_base_type(ctx, &list.base),
    }
}

/// Get the base type string for a type.
pub fn to_base_type_str(ty: &BaseType) -> String {
    match ty {
        BaseType::Named(name) => name.to_string(),
        BaseType::List(ty_list) => to_base_type_str(&ty_list.base),
    }
}

/// Transform a type into his associated input Type.
/// The type must not be a modelized type.
///
/// For a modelized type, the return type id `ID`.
///
/// # Examples
///
/// For String -> String
/// For Author -> AuthorInput
/// For [String!]! -> [String!]!
/// For [Author!] -> [AuthorInput!]
///
pub fn to_input_type(
    ctx: &HashMap<String, Cow<'_, Positioned<TypeDefinition>>>,
    Type { base, nullable }: Type,
) -> Type {
    Type {
        base: to_input_base_type(ctx, base),
        nullable,
    }
}

/// Transform a relation type to the associated an input type
pub fn to_defined_input_type(Type { base, nullable }: Type, relation_type: String) -> Type {
    Type {
        base: to_defined_input_base_type(base, &relation_type),
        nullable,
    }
}

fn to_defined_input_base_type(base_type: BaseType, relation_type: &str) -> BaseType {
    match base_type {
        BaseType::Named(_name) => BaseType::Named(Name::new(relation_type)),
        BaseType::List(list) => BaseType::List(Box::new(Type {
            base: to_defined_input_base_type(list.base, relation_type),
            nullable: list.nullable,
        })),
    }
}

/// Tell if the type is a Modelized Node.
pub fn is_modelized_node<'a>(
    ctx: &'a HashMap<String, Cow<'_, Positioned<TypeDefinition>>>,
    Type { base, .. }: &'a Type,
) -> Option<&'a Cow<'a, Positioned<TypeDefinition>>> {
    is_modelized_node_base_type(ctx, base)
}

/// Check if the given type is a non-nullable ID type
pub fn is_id_type_and_non_nullable(field: &FieldDefinition) -> bool {
    match &field.ty.node.base {
        BaseType::Named(name) => matches!(name.as_ref(), "ID"),
        _ => false,
    }
}

pub fn to_lower_camelcase<S: AsRef<str>>(field: S) -> String {
    field.as_ref().to_snake().to_camel_lowercase()
}
