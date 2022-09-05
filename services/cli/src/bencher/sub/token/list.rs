use std::convert::TryFrom;

use async_trait::async_trait;
use bencher_json::ResourceId;

use crate::{
    bencher::{backend::Backend, sub::SubCmd, wide::Wide},
    cli::token::CliTokenList,
    BencherError,
};

#[derive(Debug)]
pub struct List {
    pub user: ResourceId,
    pub backend: Backend,
}

impl TryFrom<CliTokenList> for List {
    type Error = BencherError;

    fn try_from(list: CliTokenList) -> Result<Self, Self::Error> {
        let CliTokenList { user, backend } = list;
        Ok(Self {
            user,
            backend: backend.try_into()?,
        })
    }
}

#[async_trait]
impl SubCmd for List {
    async fn exec(&self, _wide: &Wide) -> Result<(), BencherError> {
        self.backend
            .get(&format!("/v0/users/{}/tokens", self.user.as_str()))
            .await?;
        Ok(())
    }
}
