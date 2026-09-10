//! GraphQL over the engine. Schema lives here; resolvers forward to gRPC.

use async_graphql::{
    Context, EmptyMutation, EmptySubscription, Object, Result, Schema, SimpleObject,
};
use async_graphql_axum::GraphQL;
use axum::{
    Router,
    response::{Html, IntoResponse},
    routing::get,
};
use tbd_proto::engine::v1::EvaluateRequest;

use crate::AppState;

/// The executable schema.
pub type AppSchema = Schema<Query, EmptyMutation, EmptySubscription>;

/// Root query.
pub struct Query;

/// Result of `evaluate`. `stub` is forwarded from the engine untouched.
#[derive(SimpleObject)]
pub struct Evaluation {
    /// Subject that was scored.
    subject_id: String,
    /// The score.
    score: f64,
    /// True while the score is a placeholder, not a model output.
    stub: bool,
    /// Model version that produced the score.
    model_version: String,
}

#[Object]
impl Query {
    /// Protocol version.
    async fn version(&self) -> &'static str {
        tbd_common::VERSION
    }

    /// True when the engine reports healthy.
    async fn engine_ready(&self, ctx: &Context<'_>) -> bool {
        ctx.data_unchecked::<AppState>().engine_ready().await
    }

    /// Score a subject once.
    async fn evaluate(
        &self,
        ctx: &Context<'_>,
        subject_id: String,
        payload: Option<String>,
    ) -> Result<Evaluation> {
        let resp = ctx
            .data_unchecked::<AppState>()
            .engine()
            .evaluate(EvaluateRequest {
                subject_id,
                payload: payload.unwrap_or_default().into_bytes(),
            })
            .await
            .map_err(|s| async_graphql::Error::new(s.message().to_owned()))?
            .into_inner();
        Ok(Evaluation {
            subject_id: resp.subject_id,
            score: resp.score,
            stub: resp.stub,
            model_version: resp.model_version,
        })
    }
}

/// Build the schema with the app state injected.
pub fn schema(state: &AppState) -> AppSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(state.clone())
        .finish()
}

pub fn routes(state: &AppState) -> Router<AppState> {
    let schema = schema(state);
    Router::new().route("/graphql", get(graphiql).post_service(GraphQL::new(schema)))
}

async fn graphiql() -> impl IntoResponse {
    Html(
        async_graphql::http::GraphiQLSource::build()
            .endpoint("/graphql")
            .finish(),
    )
}
