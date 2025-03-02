use std::convert::TryFrom;

use async_trait::async_trait;
use bencher_client::types::{AlertStatus, JsonUpdateAlert};
use bencher_json::{AlertUuid, ResourceId};

use crate::{
    bencher::{backend::Backend, sub::SubCmd},
    parser::project::alert::{CliAlertStatus, CliAlertUpdate},
    CliError,
};

#[derive(Debug, Clone)]
pub struct Update {
    pub project: ResourceId,
    pub alert: AlertUuid,
    pub status: Option<AlertStatus>,
    pub backend: Backend,
}

impl TryFrom<CliAlertUpdate> for Update {
    type Error = CliError;

    fn try_from(create: CliAlertUpdate) -> Result<Self, Self::Error> {
        let CliAlertUpdate {
            project,
            alert,
            status,
            backend,
        } = create;
        Ok(Self {
            project,
            alert,
            status: status.map(Into::into),
            backend: backend.try_into()?,
        })
    }
}

impl From<CliAlertStatus> for AlertStatus {
    fn from(status: CliAlertStatus) -> Self {
        match status {
            CliAlertStatus::Active => Self::Active,
            CliAlertStatus::Dismissed => Self::Dismissed,
        }
    }
}

impl From<Update> for JsonUpdateAlert {
    fn from(update: Update) -> Self {
        let Update { status, .. } = update;
        Self { status }
    }
}

#[async_trait]
impl SubCmd for Update {
    async fn exec(&self) -> Result<(), CliError> {
        let _json = self
            .backend
            .send(|client| async move {
                client
                    .proj_alert_patch()
                    .project(self.project.clone())
                    .alert(self.alert)
                    .body(self.clone())
                    .send()
                    .await
            })
            .await?;
        Ok(())
    }
}
