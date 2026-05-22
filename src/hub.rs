use crate::domain::{DeviceDescriptor, DeviceSample, ServerEvent};
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc, oneshot};

use uuid::Uuid;

#[derive(Clone)]
pub struct HubHandle {
    tx: mpsc::Sender<HubCommand>,
}

impl HubHandle {
    pub fn new(buffer: usize) -> (Self, HubActor) {
        let (tx, rx) = mpsc::channel(buffer);
        let (samples_tx, _) = broadcast::channel(buffer);

        (Self { tx }, HubActor::new(rx, samples_tx))
    }

    pub async fn ingest_sample(
        &self,
        sample: DeviceSample,
    ) -> Result<IngestSampleResult, HubError> {
        let (reply_tx, reply_rx) = oneshot::channel();

        self.tx
            .send(HubCommand::IngestSample {
                sample,
                reply: reply_tx,
            })
            .await
            .map_err(|_| HubError::Unavailable)?;

        reply_rx.await.map_err(|_| HubError::Unavailable)
    }

    pub async fn put_descriptor(&self, descriptor: DeviceDescriptor) -> Result<(), HubError> {
        let (reply_tx, reply_rx) = oneshot::channel();

        self.tx
            .send(HubCommand::PutDescriptor {
                descriptor,
                reply: reply_tx,
            })
            .await
            .map_err(|_| HubError::Unavailable)?;

        reply_rx.await.map_err(|_| HubError::Unavailable)
    }

    pub async fn subscribe_samples(&self) -> Result<broadcast::Receiver<ServerEvent>, HubError> {
        let (reply_tx, reply_rx) = oneshot::channel();

        self.tx
            .send(HubCommand::SubscribeSamples { reply: reply_tx })
            .await
            .map_err(|_| HubError::Unavailable)?;

        reply_rx.await.map_err(|_| HubError::Unavailable)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IngestSampleResult {
    Ingested,
    IngestedNeedDescriptor,
}

#[derive(Debug, Clone, Copy)]
pub enum HubError {
    Unavailable,
}

pub struct HubActor {
    rx: mpsc::Receiver<HubCommand>,
    hub: Hub,
    samples_tx: broadcast::Sender<ServerEvent>,
}

impl HubActor {
    fn new(rx: mpsc::Receiver<HubCommand>, samples_tx: broadcast::Sender<ServerEvent>) -> Self {
        Self {
            rx,
            hub: Hub::new(),
            samples_tx,
        }
    }

    pub async fn run(mut self) {
        while let Some(cmd) = self.rx.recv().await {
            match cmd {
                HubCommand::IngestSample { sample, reply } => {
                    let result = self.hub.ingest_sample(sample.clone());
                    let _ = self.samples_tx.send(ServerEvent::Sample(sample));
                    let _ = reply.send(result);
                }

                HubCommand::PutDescriptor { descriptor, reply } => {
                    let _ = self
                        .samples_tx
                        .send(ServerEvent::Descriptor(descriptor.clone()));
                    self.hub.put_descriptor(descriptor);
                    let _ = reply.send(());
                }

                HubCommand::SubscribeSamples { reply } => {
                    let _ = reply.send(self.samples_tx.subscribe());
                }
            }
        }
    }
}

enum HubCommand {
    IngestSample {
        sample: DeviceSample,
        reply: oneshot::Sender<IngestSampleResult>,
    },
    PutDescriptor {
        descriptor: DeviceDescriptor,
        reply: oneshot::Sender<()>,
    },
    SubscribeSamples {
        reply: oneshot::Sender<broadcast::Receiver<ServerEvent>>,
    },
}

struct Hub {
    descriptors: HashMap<Uuid, DeviceDescriptor>,
    latest_samples: HashMap<Uuid, DeviceSample>,
}

impl Hub {
    fn new() -> Self {
        Self {
            descriptors: HashMap::new(),
            latest_samples: HashMap::new(),
        }
    }

    fn ingest_sample(&mut self, sample: DeviceSample) -> IngestSampleResult {
        let need_descriptor = !self.descriptors.contains_key(&sample.device_id);
        self.latest_samples.insert(sample.device_id, sample);

        if need_descriptor {
            IngestSampleResult::IngestedNeedDescriptor
        } else {
            IngestSampleResult::Ingested
        }
    }

    fn put_descriptor(&mut self, descriptor: DeviceDescriptor) {
        self.descriptors.insert(descriptor.device_id, descriptor);
    }
}
