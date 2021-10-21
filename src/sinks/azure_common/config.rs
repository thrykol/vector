use crate::event::{EventFinalizers, EventStatus, Finalizable};
use crate::sinks::{util::retries::RetryLogic, Healthcheck};
use azure_core::prelude::*;
use azure_core::HttpError;
use azure_storage::blob::blob::responses::PutBlockBlobResponse;
use azure_storage::blob::prelude::*;
use azure_storage::core::prelude::*;
use bytes::Bytes;
use futures::FutureExt;
use http::StatusCode;
use snafu::Snafu;
use std::sync::Arc;
use vector_core::buffers::Ackable;
use vector_core::internal_event::EventsSent;
use vector_core::stream::DriverResponse;

#[derive(Debug, Clone)]
pub struct AzureBlobRequest {
    pub blob_data: Bytes,
    pub content_encoding: Option<&'static str>,
    pub content_type: &'static str,
    pub metadata: AzureBlobMetadata,
}

impl Ackable for AzureBlobRequest {
    fn ack_size(&self) -> usize {
        self.metadata.count
    }
}

impl Finalizable for AzureBlobRequest {
    fn take_finalizers(&mut self) -> EventFinalizers {
        std::mem::take(&mut self.metadata.finalizers)
    }
}

#[derive(Clone, Debug)]
pub struct AzureBlobMetadata {
    pub partition_key: String,
    pub count: usize,
    pub byte_size: usize,
    pub finalizers: EventFinalizers,
}

#[derive(Debug, Clone)]
pub struct AzureBlobRetryLogic;

impl RetryLogic for AzureBlobRetryLogic {
    type Error = HttpError;
    type Response = AzureBlobResponse;

    fn is_retriable_error(&self, error: &Self::Error) -> bool {
        match error {
            HttpError::UnexpectedStatusCode { received, .. } => {
                received.is_server_error() || received == &StatusCode::TOO_MANY_REQUESTS
            }
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct AzureBlobResponse {
    pub inner: PutBlockBlobResponse,
    pub count: usize,
    pub events_byte_size: usize,
}

impl DriverResponse for AzureBlobResponse {
    fn event_status(&self) -> EventStatus {
        EventStatus::Delivered
    }

    fn events_sent(&self) -> EventsSent {
        EventsSent {
            count: self.count,
            byte_size: self.events_byte_size,
        }
    }
}

#[derive(Debug, Snafu)]
pub enum HealthcheckError {
    #[snafu(display("Invalid connection string specified"))]
    InvalidCredentials,
    #[snafu(display("Container: {:?} not found", container))]
    UnknownContainer { container: String },
    #[snafu(display("Unknown status code: {}", status))]
    Unknown { status: StatusCode },
}

pub fn build_healthcheck(
    container_name: String,
    client: Arc<ContainerClient>,
) -> crate::Result<Healthcheck> {
    let healthcheck = async move {
        let request = client.get_properties().execute().await;

        match request {
            Ok(_) => Ok(()),
            Err(reason) => Err(match reason.downcast_ref::<HttpError>() {
                Some(HttpError::UnexpectedStatusCode { received, .. }) => match *received {
                    StatusCode::FORBIDDEN => HealthcheckError::InvalidCredentials.into(),
                    StatusCode::NOT_FOUND => HealthcheckError::UnknownContainer {
                        container: container_name,
                    }
                    .into(),
                    status => HealthcheckError::Unknown { status }.into(),
                },
                _ => reason,
            }),
        }
    };

    Ok(healthcheck.boxed())
}

pub fn build_client(
    connection_string: String,
    container_name: String,
) -> crate::Result<Arc<ContainerClient>> {
    let client =
        StorageAccountClient::new_connection_string(new_http_client(), connection_string.as_str())?
            .as_storage_client()
            .as_container_client(container_name);

    Ok(client)
}