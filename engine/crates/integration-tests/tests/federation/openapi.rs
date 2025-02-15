use std::net::SocketAddr;

use integration_tests::{runtime, Engine, EngineBuilder, ResponseExt};
use serde_json::{json, Value};
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

#[test]
fn simple_inferred_stripe_federation() {
    // Simple test of our inferred federation keys w/ the stripe API
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = build_engine(stripe_schema(mock_server.address())).await;

        Mock::given(method("GET"))
            .and(path("/v1/products/123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(stripe_product()))
            .mount(&mock_server)
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                    r#"
                    query($repr: _Any!) {
                        _entities(representations: [$repr]) {
                            __typename
                            ... on Product {
                                name
                                shippable
                            }
                        }
                    }
                "#,
                )
                .variables(json!({"repr": {
                    "__typename": "Product",
                    "id": "123"
                }}))
                .await
                .into_data::<Value>(),
            @r###"
        {
          "_entities": [
            {
              "__typename": "Product",
              "name": "Widget",
              "shippable": true
            }
          ]
        }
        "###
        );
    });
}

#[test]
fn extending_openapi_type_with_key() {
    // Simple test of our inferred federation keys w/ the stripe API
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = build_engine(format!(
            r#"
            {}

            extend type Quote @key(fields: "id", select: "quote(quote: $id)")
            "#,
            stripe_schema(mock_server.address())
        ))
        .await;

        Mock::given(method("GET"))
            .and(path("/v1/quotes/123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(stripe_quote()))
            .mount(&mock_server)
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                    r#"
                    query($repr: _Any!) {
                        _entities(representations: [$repr]) {
                            __typename
                            ... on Quote {
                                amountTotal
                            }
                        }
                    }
                "#,
                )
                .variables(json!({"repr": {
                    "__typename": "Quote",
                    "id": "123"
                }}))
                .await
                .into_data::<Value>(),
            @r###"
        {
          "_entities": [
            {
              "__typename": "Quote",
              "amountTotal": 100
            }
          ]
        }
        "###
        );
    });
}

fn stripe_product() -> Value {
    json!({"name": "Widget", "shippable": true})
}

fn stripe_quote() -> Value {
    json!({"amount_total": 100})
}

async fn build_engine(schema: String) -> Engine {
    EngineBuilder::new(schema)
        .with_openapi_schema(
            "http://example.com/stripe.json",
            include_str!("../../../parser-openapi/test_data/stripe.openapi.json"),
        )
        .build()
        .await
}

fn stripe_schema(address: &SocketAddr) -> String {
    format!(
        r#"
          extend schema
          @openapi(
            name: "stripe",
            namespace: false,
            url: "http://{address}",
            schema: "http://example.com/stripe.json",
          )
          @federation(version: "2.3")
        "#
    )
}
