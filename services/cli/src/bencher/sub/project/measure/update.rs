use std::convert::TryFrom;

use async_trait::async_trait;
use bencher_client::types::JsonUpdateMeasure;
use bencher_json::{NonEmpty, ResourceId, Slug};

use crate::{
    bencher::{backend::Backend, sub::SubCmd},
    parser::project::measure::CliMeasureUpdate,
    CliError,
};

#[derive(Debug, Clone)]
pub struct Update {
    pub project: ResourceId,
    pub measure: ResourceId,
    pub name: Option<NonEmpty>,
    pub slug: Option<Slug>,
    pub units: Option<NonEmpty>,
    pub backend: Backend,
}

impl TryFrom<CliMeasureUpdate> for Update {
    type Error = CliError;

    fn try_from(create: CliMeasureUpdate) -> Result<Self, Self::Error> {
        let CliMeasureUpdate {
            project,
            measure,
            name,
            slug,
            units,
            backend,
        } = create;
        Ok(Self {
            project,
            measure,
            name,
            slug,
            units,
            backend: backend.try_into()?,
        })
    }
}

impl From<Update> for JsonUpdateMeasure {
    fn from(update: Update) -> Self {
        let Update {
            name, slug, units, ..
        } = update;
        Self {
            name: name.map(Into::into),
            slug: slug.map(Into::into),
            units: units.map(Into::into),
        }
    }
}

#[async_trait]
impl SubCmd for Update {
    async fn exec(&self) -> Result<(), CliError> {
        let _json = self
            .backend
            .send(|client| async move {
                client
                    .proj_measure_patch()
                    .project(self.project.clone())
                    .measure(self.measure.clone())
                    .body(self.clone())
                    .send()
                    .await
            })
            .await?;
        Ok(())
    }
}
