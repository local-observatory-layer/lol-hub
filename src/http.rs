use crate::{
    domain::{DeviceDescriptor, DeviceSample},
    hub::{HubError, HubHandle, IngestSampleResult},
};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    routing::post,
};
use futures_util::stream::{self, Stream};
use std::{convert::Infallible, time::Duration};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
#[derive(Clone)]
pub struct AppState {
    pub hub: HubHandle,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/samples", post(post_sample))
        .route("/samples/stream", axum::routing::get(stream_samples))
        .route("/descriptor", post(post_descriptor))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
}

async fn post_sample(
    State(state): State<AppState>,
    Json(sample): Json<DeviceSample>,
) -> Result<StatusCode, StatusCode> {
    match state.hub.ingest_sample(sample).await {
        Ok(IngestSampleResult::Ingested) => Ok(StatusCode::NO_CONTENT),
        Ok(IngestSampleResult::IngestedNeedDescriptor) => Ok(StatusCode::ACCEPTED),
        Err(err) => Err(map_hub_error(err)),
    }
}

async fn stream_samples(
    State(state): State<AppState>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    let rx = state.hub.subscribe_samples().await.map_err(map_hub_error)?;

    let stream = stream::unfold(rx, |mut rx| async move {
        loop {
            match rx.recv().await {
                Ok(sample) => {
                    let event = Event::default().event("sample").json_data(&sample).unwrap();

                    return Some((Ok(event), rx));
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    return None;
                }
            }
        }
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
}
async fn post_descriptor(
    State(state): State<AppState>,
    Json(descriptor): Json<DeviceDescriptor>,
) -> Result<StatusCode, StatusCode> {
    state
        .hub
        .put_descriptor(descriptor)
        .await
        .map_err(map_hub_error)?;

    Ok(StatusCode::NO_CONTENT)
}

fn map_hub_error(_: HubError) -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}
