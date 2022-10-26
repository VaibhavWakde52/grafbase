use super::types::{Constraint, OperationKind, Record, Row, Sql};
use crate::constant::*;
use crate::graph_transaction::{
    DeleteAllRelationsInternalInput, DeleteMultipleRelationsInternalInput, DeleteNodeConstraintInternalInput,
    DeleteNodeInternalInput, DeleteRelationInternalInput, DeleteUnitNodeConstraintInput, ExecuteChangesOnDatabase,
    InsertNodeConstraintInternalInput, InsertNodeInternalInput, InsertRelationInternalInput, InsertUniqueConstraint,
    InternalChanges, InternalNodeChanges, InternalNodeConstraintChanges, InternalRelationChanges, ToTransactionError,
    ToTransactionFuture, UpdateNodeConstraintInternalInput, UpdateNodeInternalInput, UpdateRelation,
    UpdateRelationInternalInput, UpdateUniqueConstraint,
};
use crate::local::types::SqlValue;
use crate::model::constraint::db::ConstraintID;
use crate::model::node::NodeID;
use crate::{DynamoDBBatchersData, DynamoDBContext};
use chrono::{SecondsFormat, Utc};
use dynomite::{Attribute, AttributeValue};
use itertools::Itertools;
use maplit::hashmap;
use std::collections::HashMap;

impl ExecuteChangesOnDatabase for InsertNodeInternalInput {
    fn to_transaction<'a>(
        self,
        _batchers: &'a DynamoDBBatchersData,
        _ctx: &'a DynamoDBContext,
        pk: String,
        sk: String,
    ) -> ToTransactionFuture<'a> {
        Box::pin(async {
            let InsertNodeInternalInput {
                id,
                user_defined_item,
                ty,
            } = self;

            let id = NodeID::new_owned(ty, id);

            let utc_now = Utc::now();

            let now_attr = utc_now.to_rfc3339_opts(SecondsFormat::Millis, true).into_attr();
            let ty_attr = id.ty().into_attr();
            let autogenerated_id_attr = id.clone().into_attr();

            let mut document = user_defined_item;

            document.insert(PK.to_string(), autogenerated_id_attr.clone());
            document.insert(SK.to_string(), autogenerated_id_attr.clone());
            document.insert(TYPE.to_string(), ty_attr.clone());
            document.insert(CREATED_AT.to_string(), now_attr.clone());
            document.insert(UPDATED_AT.to_string(), now_attr);
            document.insert(TYPE_INDEX_PK.to_string(), ty_attr);
            document.insert(TYPE_INDEX_SK.to_string(), autogenerated_id_attr.clone());
            document.insert(INVERTED_INDEX_PK.to_string(), autogenerated_id_attr.clone());
            document.insert(INVERTED_INDEX_SK.to_string(), autogenerated_id_attr);

            let record = Record {
                pk,
                sk,
                entity_type: Some(id.ty().to_string()),
                created_at: utc_now,
                updated_at: utc_now,
                relation_names: Default::default(),
                gsi1pk: Some(id.ty().to_string()),
                gsi1sk: Some(id.to_string()),
                gsi2pk: Some(id.to_string()),
                gsi2sk: Some(id.to_string()),
                document,
            };

            let row = Row::from_record(record);

            let (query, values) = Sql::Insert(&row).compile(row.values.clone());

            Ok((query, values, None))
        })
    }
}

impl ExecuteChangesOnDatabase for UpdateNodeInternalInput {
    fn to_transaction<'a>(
        self,
        _batchers: &'a DynamoDBBatchersData,
        _ctx: &'a DynamoDBContext,
        pk: String,
        sk: String,
    ) -> ToTransactionFuture<'a> {
        Box::pin(async {
            let UpdateNodeInternalInput { user_defined_item, .. } = self;

            let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            let now_attr = now.clone().into_attr();

            let mut document = user_defined_item;

            document.insert(UPDATED_AT.to_string(), now_attr);

            let updated_at = now;
            let document = serde_json::to_string(&document).expect("must serialize");

            let (query, values) = Sql::Update.compile(hashmap! {
                "pk" => SqlValue::String(pk),
                "sk" => SqlValue::String(sk),
                "document" => SqlValue::String(document),
                "updated_at" => SqlValue::String(updated_at),
            });

            Ok((query, values, None))
        })
    }
}

impl ExecuteChangesOnDatabase for UpdateUniqueConstraint {
    fn to_transaction<'a>(
        self,
        _batchers: &'a DynamoDBBatchersData,
        _ctx: &'a DynamoDBContext,
        pk: String,
        sk: String,
    ) -> ToTransactionFuture<'a> {
        Box::pin(async {
            let UpdateUniqueConstraint {
                target,
                user_defined_item,
                ..
            } = self;

            let id = ConstraintID::try_from(pk.clone()).expect("Wrong Constraint ID");
            let utc_now = Utc::now().to_string();
            let now_attr = utc_now.clone().into_attr();
            let id_attr = id.to_string().into_attr();

            let mut document: HashMap<String, AttributeValue> = user_defined_item;

            document.insert(PK.to_string(), id_attr.clone());
            document.insert(SK.to_string(), id_attr.clone());
            document.insert(CREATED_AT.to_string(), now_attr.clone());
            document.insert(UPDATED_AT.to_string(), now_attr);
            document.insert(INVERTED_INDEX_PK.to_string(), target.into_attr());
            document.insert(INVERTED_INDEX_SK.to_string(), id_attr);

            document.remove(&TYPE_INDEX_PK.to_string());
            document.remove(&TYPE_INDEX_SK.to_string());

            let updated_at = utc_now;
            let document = serde_json::to_string(&document).expect("must serialize");

            let (query, values) = Sql::Update.compile(hashmap! {
                "pk" => SqlValue::String(pk),
                "sk" => SqlValue::String(sk),
                "document" => SqlValue::String(document),
                "updated_at" => SqlValue::String(updated_at),
            });

            Ok((query, values, None))
        })
    }
}

impl ExecuteChangesOnDatabase for DeleteNodeInternalInput {
    fn to_transaction<'a>(
        self,
        _batchers: &'a DynamoDBBatchersData,
        _ctx: &'a DynamoDBContext,
        pk: String,
        sk: String,
    ) -> ToTransactionFuture<'a> {
        Box::pin(async {
            let (query, values) = Sql::DeleteByIds.compile(hashmap! {
                "pk" => SqlValue::String(pk),
                "sk" => SqlValue::String(sk),
            });
            Ok((query, values, None))
        })
    }
}

impl ExecuteChangesOnDatabase for InternalNodeChanges {
    fn to_transaction<'a>(
        self,
        batchers: &'a DynamoDBBatchersData,
        ctx: &'a DynamoDBContext,
        pk: String,
        sk: String,
    ) -> ToTransactionFuture<'a> {
        match self {
            Self::Insert(input) => input.to_transaction(batchers, ctx, pk, sk),
            Self::Delete(input) => input.to_transaction(batchers, ctx, pk, sk),
            Self::Update(input) => input.to_transaction(batchers, ctx, pk, sk),
        }
    }
}

impl ExecuteChangesOnDatabase for InsertRelationInternalInput {
    fn to_transaction<'a>(
        self,
        _batchers: &'a DynamoDBBatchersData,
        _ctx: &'a DynamoDBContext,
        pk: String,
        sk: String,
    ) -> ToTransactionFuture<'a> {
        Box::pin(async move {
            let InsertRelationInternalInput {
                fields,
                relation_names,
                from_ty,
                to_ty,
                ..
            } = self;

            let utc_now = Utc::now();

            let mut document = fields;

            let now_attr = utc_now.to_rfc3339_opts(SecondsFormat::Millis, true).into_attr();
            let gsi1pk_attr = from_ty.clone().into_attr();
            let ty_attr = to_ty.clone().into_attr();

            document.insert(PK.to_string(), pk.clone().into_attr());
            document.insert(SK.to_string(), sk.clone().into_attr());
            document.insert(TYPE.to_string(), ty_attr);
            document.insert(CREATED_AT.to_string(), now_attr.clone());
            document.insert(UPDATED_AT.to_string(), now_attr);
            document.insert(TYPE_INDEX_PK.to_string(), gsi1pk_attr);
            document.insert(TYPE_INDEX_SK.to_string(), pk.clone().into_attr());
            document.insert(INVERTED_INDEX_PK.to_string(), sk.clone().into_attr());
            document.insert(INVERTED_INDEX_SK.to_string(), pk.clone().into_attr());
            document.insert(
                RELATION_NAMES.to_string(),
                AttributeValue {
                    ss: Some(relation_names.clone()),
                    ..Default::default()
                },
            );

            let record = Record {
                pk: pk.clone(),
                sk: sk.clone(),
                entity_type: Some(to_ty),
                created_at: utc_now,
                updated_at: utc_now,
                gsi1pk: Some(from_ty),
                gsi1sk: Some(pk.clone()),
                gsi2pk: Some(sk.clone()),
                gsi2sk: Some(pk.clone()),
                relation_names: relation_names.clone(),
                document,
            };

            let row = Row::from_record(record);

            let mut value_map = row.values.clone();

            value_map.insert("to_add", SqlValue::VecDeque(relation_names.clone().into()));

            let (query, values) = Sql::InsertRelation(&row, relation_names.len()).compile(value_map);

            Ok((query, values, None))
        })
    }
}

impl ExecuteChangesOnDatabase for DeleteAllRelationsInternalInput {
    fn to_transaction<'a>(
        self,
        _batchers: &'a DynamoDBBatchersData,
        _ctx: &'a DynamoDBContext,
        pk: String,
        sk: String,
    ) -> ToTransactionFuture<'a> {
        Box::pin(async {
            let (query, values) = Sql::DeleteByIds.compile(hashmap! {
                "pk"=> SqlValue::String(pk),
                "sk" => SqlValue::String(sk),
            });
            Ok((query, values, None))
        })
    }
}

impl ExecuteChangesOnDatabase for DeleteMultipleRelationsInternalInput {
    fn to_transaction<'a>(
        self,
        _batchers: &'a DynamoDBBatchersData,
        _ctx: &'a DynamoDBContext,
        pk: String,
        sk: String,
    ) -> ToTransactionFuture<'a> {
        Box::pin(async {
            let DeleteMultipleRelationsInternalInput { relation_names, .. } = self;

            let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            let now_attr = now.clone().into_attr();

            let mut document = HashMap::<String, AttributeValue>::new();

            document.insert(UPDATED_AT.to_string(), now_attr);

            let updated_at = now;
            let document = serde_json::to_string(&document).expect("must serialize");

            let value_map = hashmap! {
                "pk" => SqlValue::String(pk),
                "sk" => SqlValue::String(sk),
                "to_remove" => SqlValue::VecDeque(relation_names.clone().into()),
                "document" => SqlValue::String(document),
                "updated_at" => SqlValue::String(updated_at),
            };

            let (query, values) = Sql::DeleteRelations(relation_names.len()).compile(value_map);

            Ok((query, values, None))
        })
    }
}

impl ExecuteChangesOnDatabase for DeleteRelationInternalInput {
    fn to_transaction<'a>(
        self,
        batchers: &'a DynamoDBBatchersData,
        ctx: &'a DynamoDBContext,
        pk: String,
        sk: String,
    ) -> ToTransactionFuture<'a> {
        match self {
            Self::All(a) => a.to_transaction(batchers, ctx, pk, sk),
            Self::Multiple(a) => a.to_transaction(batchers, ctx, pk, sk),
        }
    }
}

impl ExecuteChangesOnDatabase for UpdateRelationInternalInput {
    fn to_transaction<'a>(
        self,
        _batchers: &'a DynamoDBBatchersData,
        _ctx: &'a DynamoDBContext,
        pk: String,
        sk: String,
    ) -> ToTransactionFuture<'a> {
        Box::pin(async {
            let UpdateRelationInternalInput {
                user_defined_item,
                relation_names,
                ..
            } = self;

            let (removed, added): (Vec<String>, Vec<String>) =
                relation_names.into_iter().partition_map(|relation| match relation {
                    UpdateRelation::Add(a) => itertools::Either::Right(a),
                    UpdateRelation::Remove(a) => itertools::Either::Left(a),
                });

            let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            let now_attr = now.clone().into_attr();

            let mut document = user_defined_item;

            document.insert(UPDATED_AT.to_string(), now_attr);

            let updated_at = now;
            let document = serde_json::to_string(&document).expect("must serialize");

            let value_map = hashmap! {
                "pk" => SqlValue::String(pk),
                "sk" => SqlValue::String(sk),
                "to_remove" => SqlValue::VecDeque(removed.clone().into()),
                "to_add" => SqlValue::VecDeque(added.clone().into()),
                "document" => SqlValue::String(document),
                "updated_at" => SqlValue::String(updated_at)
            };

            let (query, values) = Sql::UpdateWithRelations(removed.len(), added.len()).compile(value_map);

            Ok((query, values, None))
        })
    }
}

impl ExecuteChangesOnDatabase for InternalRelationChanges {
    fn to_transaction<'a>(
        self,
        batchers: &'a DynamoDBBatchersData,
        ctx: &'a DynamoDBContext,
        pk: String,
        sk: String,
    ) -> ToTransactionFuture<'a> {
        match self {
            Self::Insert(input) => input.to_transaction(batchers, ctx, pk, sk),
            Self::Delete(input) => input.to_transaction(batchers, ctx, pk, sk),
            Self::Update(input) => input.to_transaction(batchers, ctx, pk, sk),
        }
    }
}

impl ExecuteChangesOnDatabase for Vec<InternalChanges> {
    fn to_transaction<'a>(
        self,
        batchers: &'a DynamoDBBatchersData,
        ctx: &'a DynamoDBContext,
        pk: String,
        sk: String,
    ) -> ToTransactionFuture<'a> {
        let mut list = self.into_iter();
        let first = list.next().map(|first| list.try_fold(first, |acc, cur| acc.with(cur)));

        let first = match first {
            Some(Ok(first)) => first,
            _ => return Box::pin(async { Err(ToTransactionError::Unknown) }),
        };

        first.to_transaction(batchers, ctx, pk, sk)
    }
}

impl ExecuteChangesOnDatabase for InternalChanges {
    fn to_transaction<'a>(
        self,
        batchers: &'a DynamoDBBatchersData,
        ctx: &'a DynamoDBContext,
        pk: String,
        sk: String,
    ) -> ToTransactionFuture<'a> {
        match self {
            Self::Node(input) => input.to_transaction(batchers, ctx, pk, sk),
            Self::Relation(input) => input.to_transaction(batchers, ctx, pk, sk),
            Self::NodeConstraints(input) => input.to_transaction(batchers, ctx, pk, sk),
        }
    }
}

impl ExecuteChangesOnDatabase for DeleteUnitNodeConstraintInput {
    fn to_transaction<'a>(
        self,
        _batchers: &'a DynamoDBBatchersData,
        _ctx: &'a DynamoDBContext,
        pk: String,
        sk: String,
    ) -> ToTransactionFuture<'a> {
        Box::pin(async {
            let (query, values) = Sql::DeleteByIds.compile(hashmap! {
                "pk" => SqlValue::String(pk),
                "sk" => SqlValue::String(sk)
            });

            Ok((query, values, None))
        })
    }
}

impl ExecuteChangesOnDatabase for InsertUniqueConstraint {
    fn to_transaction<'a>(
        self,
        _batchers: &'a DynamoDBBatchersData,
        _ctx: &'a DynamoDBContext,
        pk: String,
        sk: String,
    ) -> ToTransactionFuture<'a> {
        Box::pin(async {
            let InsertUniqueConstraint {
                target,
                user_defined_item,
            } = self;

            let id = ConstraintID::try_from(pk.clone()).expect("Wrong Constraint ID");
            let utc_now = Utc::now();
            let now_attr = utc_now.to_rfc3339_opts(SecondsFormat::Millis, true).into_attr();
            let id_attr = id.to_string().into_attr();

            let mut document: HashMap<String, AttributeValue> = user_defined_item;

            document.insert(PK.to_string(), id_attr.clone());
            document.insert(SK.to_string(), id_attr.clone());
            document.insert(CREATED_AT.to_string(), now_attr.clone());
            document.insert(UPDATED_AT.to_string(), now_attr);
            document.insert(INVERTED_INDEX_PK.to_string(), target.clone().into_attr());
            document.insert(INVERTED_INDEX_SK.to_string(), id_attr);

            let record = Record {
                pk,
                sk,
                entity_type: None,
                created_at: utc_now,
                updated_at: utc_now,
                relation_names: Default::default(),
                gsi1pk: None,
                gsi1sk: None,
                gsi2pk: Some(target),
                gsi2sk: Some(id.to_string()),
                document,
            };

            let row = Row::from_record(record);

            let (query, values) = Sql::Insert(&row).compile(row.values.clone());

            Ok((
                query,
                values,
                Some(OperationKind::Constraint(Constraint::Unique {
                    value: id.value().to_string(),
                    field: id.field().to_string(),
                })),
            ))
        })
    }
}

impl ExecuteChangesOnDatabase for DeleteNodeConstraintInternalInput {
    fn to_transaction<'a>(
        self,
        batchers: &'a DynamoDBBatchersData,
        ctx: &'a DynamoDBContext,
        pk: String,
        sk: String,
    ) -> ToTransactionFuture<'a> {
        match self {
            Self::Unit(a) => a.to_transaction(batchers, ctx, pk, sk),
        }
    }
}

impl ExecuteChangesOnDatabase for InsertNodeConstraintInternalInput {
    fn to_transaction<'a>(
        self,
        batchers: &'a DynamoDBBatchersData,
        ctx: &'a DynamoDBContext,
        pk: String,
        sk: String,
    ) -> ToTransactionFuture<'a> {
        match self {
            Self::Unique(a) => a.to_transaction(batchers, ctx, pk, sk),
        }
    }
}

impl ExecuteChangesOnDatabase for UpdateNodeConstraintInternalInput {
    fn to_transaction<'a>(
        self,
        batchers: &'a DynamoDBBatchersData,
        ctx: &'a DynamoDBContext,
        pk: String,
        sk: String,
    ) -> ToTransactionFuture<'a> {
        match self {
            Self::Unique(a) => a.to_transaction(batchers, ctx, pk, sk),
        }
    }
}

impl ExecuteChangesOnDatabase for InternalNodeConstraintChanges {
    fn to_transaction<'a>(
        self,
        batchers: &'a DynamoDBBatchersData,
        ctx: &'a DynamoDBContext,
        pk: String,
        sk: String,
    ) -> ToTransactionFuture<'a> {
        match self {
            Self::Delete(a) => a.to_transaction(batchers, ctx, pk, sk),
            Self::Insert(a) => a.to_transaction(batchers, ctx, pk, sk),
            Self::Update(a) => a.to_transaction(batchers, ctx, pk, sk),
        }
    }
}
