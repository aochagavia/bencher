use bencher_json::JsonMetric;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use uuid::Uuid;

use crate::{
    context::DbConnection,
    schema::{self, metric as metric_table},
    ApiError,
};

use super::{
    metric_kind::{MetricKindId, QueryMetricKind},
    perf::{PerfId, QueryPerf},
};

crate::util::typed_id::typed_id!(MetricId);

#[derive(diesel::Queryable, diesel::Identifiable, diesel::Associations, diesel::Selectable)]
#[diesel(table_name = metric_table)]
#[diesel(belongs_to(QueryPerf, foreign_key = perf_id))]
#[diesel(belongs_to(QueryMetricKind, foreign_key = metric_kind_id))]
pub struct QueryMetric {
    pub id: MetricId,
    pub uuid: String,
    pub perf_id: PerfId,
    pub metric_kind_id: MetricKindId,
    pub value: f64,
    pub lower_value: Option<f64>,
    pub upper_value: Option<f64>,
}

impl QueryMetric {
    pub fn from_uuid(conn: &mut DbConnection, uuid: String) -> Result<Self, ApiError> {
        schema::metric::table
            .filter(schema::metric::uuid.eq(uuid))
            .first::<Self>(conn)
            .map_err(ApiError::from)
    }

    pub fn json(value: f64, lower_value: Option<f64>, upper_value: Option<f64>) -> JsonMetric {
        JsonMetric {
            value: value.into(),
            lower_value: lower_value.map(Into::into),
            upper_value: upper_value.map(Into::into),
        }
    }

    pub fn into_json(self) -> JsonMetric {
        let Self {
            value,
            lower_value,
            upper_value,
            ..
        } = self;
        JsonMetric {
            value: value.into(),
            lower_value: lower_value.map(Into::into),
            upper_value: upper_value.map(Into::into),
        }
    }
}

#[derive(Debug, diesel::Insertable)]
#[diesel(table_name = metric_table)]
pub struct InsertMetric {
    pub uuid: String,
    pub perf_id: PerfId,
    pub metric_kind_id: MetricKindId,
    pub value: f64,
    pub lower_value: Option<f64>,
    pub upper_value: Option<f64>,
}

impl InsertMetric {
    pub fn from_json(perf_id: PerfId, metric_kind_id: MetricKindId, metric: JsonMetric) -> Self {
        let JsonMetric {
            value,
            lower_value,
            upper_value,
        } = metric;
        Self {
            perf_id,
            metric_kind_id,
            uuid: Uuid::new_v4().to_string(),
            value: value.into(),
            lower_value: lower_value.map(Into::into),
            upper_value: upper_value.map(Into::into),
        }
    }
}
