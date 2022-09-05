use std::convert::TryFrom;

use async_trait::async_trait;

use crate::{bencher::wide::Wide, cli::CliSub, BencherError};

mod auth;
mod benchmark;
mod branch;
mod perf;
mod project;
mod report;
mod run;
mod subcmd;
mod testbed;
mod threshold;
mod token;

use auth::Auth;
use benchmark::Benchmark;
use branch::Branch;
use perf::Perf;
use project::Project;
use report::Report;
use run::Run;
pub use subcmd::SubCmd;
use testbed::Testbed;
use threshold::Threshold;
use token::Token;

#[derive(Debug)]
pub enum Sub {
    Auth(Auth),
    Project(Project),
    Report(Report),
    Branch(Branch),
    Testbed(Testbed),
    Threshold(Threshold),
    Run(Run),
    Benchmark(Benchmark),
    Perf(Perf),
    Token(Token),
}

impl TryFrom<CliSub> for Sub {
    type Error = BencherError;

    fn try_from(sub: CliSub) -> Result<Self, Self::Error> {
        Ok(match sub {
            CliSub::Auth(auth) => Self::Auth(auth.try_into()?),
            CliSub::Project(project) => Self::Project(project.try_into()?),
            CliSub::Report(report) => Self::Report(report.try_into()?),
            CliSub::Branch(branch) => Self::Branch(branch.try_into()?),
            CliSub::Testbed(testbed) => Self::Testbed(testbed.try_into()?),
            CliSub::Threshold(threshold) => Self::Threshold(threshold.try_into()?),
            CliSub::Run(run) => Self::Run(run.try_into()?),
            CliSub::Benchmark(benchmark) => Self::Benchmark(benchmark.try_into()?),
            CliSub::Perf(perf) => Self::Perf(perf.try_into()?),
            CliSub::Token(token) => Self::Token(token.try_into()?),
        })
    }
}

pub fn map_sub(sub: Option<CliSub>) -> Result<Option<Sub>, BencherError> {
    if let Some(sub) = sub {
        Ok(Some(sub.try_into()?))
    } else {
        Ok(None)
    }
}

#[async_trait]
impl SubCmd for Sub {
    async fn exec(&self, wide: &Wide) -> Result<(), BencherError> {
        match self {
            Self::Auth(auth) => auth.exec(wide).await,
            Self::Project(project) => project.exec(wide).await,
            Self::Report(report) => report.exec(wide).await,
            Self::Branch(branch) => branch.exec(wide).await,
            Self::Testbed(testbed) => testbed.exec(wide).await,
            Self::Threshold(threshold) => threshold.exec(wide).await,
            Self::Run(run) => run.exec(wide).await,
            Self::Benchmark(benchmark) => benchmark.exec(wide).await,
            Self::Perf(perf) => perf.exec(wide).await,
            Self::Token(token) => token.exec(wide).await,
        }
    }
}
