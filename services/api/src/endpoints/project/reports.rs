use std::collections::HashMap;

use bencher_json::{
    project::{
        branch::VersionNumber,
        report::{JsonReportQuery, JsonReportQueryParams},
    },
    JsonDirection, JsonNewReport, JsonPagination, JsonReport, JsonReports, ReportUuid, ResourceId,
};
use bencher_rbac::project::Permission;
use diesel::{
    dsl::count, BelongingToDsl, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper,
};
use dropshot::{endpoint, HttpError, Path, Query, RequestContext, TypedBody};
use http::StatusCode;
use schemars::JsonSchema;
use serde::Deserialize;
use slog::Logger;

use crate::{
    context::ApiContext,
    endpoints::{
        endpoint::{CorsResponse, Delete, Get, Post, ResponseCreated, ResponseDeleted, ResponseOk},
        Endpoint,
    },
    error::{bad_request_error, issue_error, resource_conflict_err, resource_not_found_err},
    model::user::auth::{AuthUser, PubBearerToken},
    model::{
        project::{
            branch::{BranchId, QueryBranch},
            report::{results::ReportResults, InsertReport, QueryReport, ReportId},
            testbed::QueryTestbed,
            version::{QueryVersion, VersionId},
            QueryProject,
        },
        user::auth::BearerToken,
    },
    schema,
    util::name_id::filter_name_id,
};

#[derive(Deserialize, JsonSchema)]
pub struct ProjReportsParams {
    pub project: ResourceId,
}

pub type ProjReportsPagination = JsonPagination<ProjReportsSort>;

#[derive(Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProjReportsSort {
    #[default]
    DateTime,
}

#[allow(clippy::unused_async)]
#[endpoint {
    method = OPTIONS,
    path =  "/v0/projects/{project}/reports",
    tags = ["projects", "reports"]
}]
pub async fn proj_reports_options(
    _rqctx: RequestContext<ApiContext>,
    _path_params: Path<ProjReportsParams>,
    _pagination_params: Query<ProjReportsPagination>,
    _query_params: Query<JsonReportQueryParams>,
) -> Result<CorsResponse, HttpError> {
    Ok(Endpoint::cors(&[Get.into(), Post.into()]))
}

#[endpoint {
    method = GET,
    path =  "/v0/projects/{project}/reports",
    tags = ["projects", "reports"]
}]
pub async fn proj_reports_get(
    rqctx: RequestContext<ApiContext>,
    path_params: Path<ProjReportsParams>,
    pagination_params: Query<ProjReportsPagination>,
    query_params: Query<JsonReportQueryParams>,
) -> Result<ResponseOk<JsonReports>, HttpError> {
    // Second round of marshaling
    let json_report_query = query_params
        .into_inner()
        .try_into()
        .map_err(bad_request_error)?;

    let auth_user = AuthUser::new_pub(&rqctx).await?;
    let json = get_ls_inner(
        &rqctx.log,
        rqctx.context(),
        auth_user.as_ref(),
        path_params.into_inner(),
        pagination_params.into_inner(),
        json_report_query,
    )
    .await?;
    Ok(Get::response_ok(json, auth_user.is_some()))
}

async fn get_ls_inner(
    log: &Logger,
    context: &ApiContext,
    auth_user: Option<&AuthUser>,
    path_params: ProjReportsParams,
    pagination_params: ProjReportsPagination,
    json_report_query: JsonReportQuery,
) -> Result<JsonReports, HttpError> {
    let conn = &mut *context.conn().await;

    let query_project =
        QueryProject::is_allowed_public(conn, &context.rbac, &path_params.project, auth_user)?;

    let mut query = QueryReport::belonging_to(&query_project)
        .inner_join(schema::branch::table)
        .inner_join(schema::testbed::table)
        .into_boxed();

    if let Some(branch) = json_report_query.branch.as_ref() {
        filter_name_id!(query, branch, branch);
    }
    if let Some(testbed) = json_report_query.testbed.as_ref() {
        filter_name_id!(query, testbed, testbed);
    }

    if let Some(start_time) = json_report_query.start_time {
        query = query.filter(schema::report::start_time.ge(start_time));
    }
    if let Some(end_time) = json_report_query.end_time {
        query = query.filter(schema::report::end_time.le(end_time));
    }

    query = match pagination_params.order() {
        ProjReportsSort::DateTime => match pagination_params.direction {
            Some(JsonDirection::Asc) => query.order((
                schema::report::start_time.asc(),
                schema::report::end_time.asc(),
                schema::report::created.asc(),
            )),
            Some(JsonDirection::Desc) | None => query.order((
                schema::report::start_time.desc(),
                schema::report::end_time.desc(),
                schema::report::created.desc(),
            )),
        },
    };

    let project = &query_project;
    Ok(query
        .offset(pagination_params.offset())
        .limit(pagination_params.limit())
        .select(QueryReport::as_select())
        .load(conn)
        .map_err(resource_not_found_err!(Report, project))?
        .into_iter()
        .filter_map(|report| match report.into_json(log, conn) {
            Ok(report) => Some(report),
            Err(err) => {
                debug_assert!(false, "{err}");
                #[cfg(feature = "sentry")]
                sentry::capture_error(&err);
                None
            },
        })
        .collect())
}

#[endpoint {
    method = POST,
    path =  "/v0/projects/{project}/reports",
    tags = ["projects", "reports"]
}]
// For simplicity, this query makes the assumption that all posts are perfectly
// chronological. That is, a report will never be posted for X after Y has
// already been submitted when X really happened before Y. For implementing git
// bisect more complex logic will be required.
pub async fn proj_report_post(
    rqctx: RequestContext<ApiContext>,
    bearer_token: BearerToken,
    path_params: Path<ProjReportsParams>,
    body: TypedBody<JsonNewReport>,
) -> Result<ResponseCreated<JsonReport>, HttpError> {
    let auth_user = AuthUser::from_token(rqctx.context(), bearer_token).await?;
    let json = post_inner(
        &rqctx.log,
        rqctx.context(),
        path_params.into_inner(),
        body.into_inner(),
        &auth_user,
    )
    .await?;
    Ok(Post::auth_response_created(json))
}

async fn post_inner(
    log: &Logger,
    context: &ApiContext,
    path_params: ProjReportsParams,
    mut json_report: JsonNewReport,
    auth_user: &AuthUser,
) -> Result<JsonReport, HttpError> {
    let conn = &mut *context.conn().await;

    // Verify that the user is allowed
    let project = QueryProject::is_allowed(
        conn,
        &context.rbac,
        &path_params.project,
        auth_user,
        Permission::Create,
    )?;
    let project_id = project.id;

    // Verify that the branch and testbed are part of the same project
    let branch_id = QueryBranch::from_name_id(conn, project_id, &json_report.branch)?.id;
    let testbed_id = QueryTestbed::from_name_id(conn, project_id, &json_report.testbed)?.id;

    // Check to see if the project is public or private
    // If private, then validate that there is an active subscription or license
    #[cfg(feature = "plus")]
    let plan_kind = crate::model::organization::plan::PlanKind::new_for_project(
        conn,
        context.biller.as_ref(),
        &context.licensor,
        &project,
    )
    .await?;

    // If there is a hash then try to see if there is already a code version for
    // this branch with that particular hash.
    // Otherwise, create a new code version for this branch with/without the hash.
    let version_id =
        QueryVersion::get_or_increment(conn, project_id, branch_id, json_report.hash.as_ref())?;

    let json_settings = json_report.settings.take().unwrap_or_default();
    let adapter = json_settings.adapter.unwrap_or_default();

    // Create a new report and add it to the database
    let insert_report = InsertReport::from_json(
        auth_user.id,
        project_id,
        branch_id,
        version_id,
        testbed_id,
        &json_report,
        adapter,
    );

    diesel::insert_into(schema::report::table)
        .values(&insert_report)
        .execute(conn)
        .map_err(resource_conflict_err!(Report, insert_report))?;

    let query_report = schema::report::table
        .filter(schema::report::uuid.eq(&insert_report.uuid))
        .first::<QueryReport>(conn)
        .map_err(|e| {
            issue_error(
                StatusCode::NOT_FOUND,
                "Failed to find new report that was just created",
                &format!("Failed to find new report ({insert_report:?}) in project ({project_id}) on Bencher even though it was just created."),
                e,
            )
        })?;

    #[cfg(feature = "plus")]
    let mut usage = 0;

    // Process and record the report results
    let mut report_results = ReportResults::new(project_id, branch_id, testbed_id, query_report.id);
    let results_array = json_report
        .results
        .iter()
        .map(AsRef::as_ref)
        .collect::<Vec<&str>>();
    let processed_report = report_results.process(
        log,
        conn,
        &results_array,
        adapter,
        json_settings,
        #[cfg(feature = "plus")]
        &mut usage,
    );

    #[cfg(feature = "plus")]
    plan_kind
        .check_usage(context.biller.as_ref(), &project, usage)
        .await?;

    // Don't return the error from processing the report until after the metrics usage has been checked
    processed_report.and_then(|()| {
        // If the report was processed successfully, then return the report with the results
        query_report.into_json(log, conn)
    })
}

#[derive(Deserialize, JsonSchema)]
pub struct ProjReportParams {
    pub project: ResourceId,
    pub report: ReportUuid,
}

#[allow(clippy::unused_async)]
#[endpoint {
    method = OPTIONS,
    path =  "/v0/projects/{project}/reports/{report}",
    tags = ["projects", "reports"]
}]
pub async fn proj_report_options(
    _rqctx: RequestContext<ApiContext>,
    _path_params: Path<ProjReportParams>,
) -> Result<CorsResponse, HttpError> {
    Ok(Endpoint::cors(&[Get.into(), Delete.into()]))
}

#[endpoint {
    method = GET,
    path =  "/v0/projects/{project}/reports/{report}",
    tags = ["projects", "reports"]
}]
pub async fn proj_report_get(
    rqctx: RequestContext<ApiContext>,
    bearer_token: PubBearerToken,
    path_params: Path<ProjReportParams>,
) -> Result<ResponseOk<JsonReport>, HttpError> {
    let auth_user = AuthUser::from_pub_token(rqctx.context(), bearer_token).await?;
    let json = get_one_inner(
        &rqctx.log,
        rqctx.context(),
        path_params.into_inner(),
        auth_user.as_ref(),
    )
    .await?;
    Ok(Get::response_ok(json, auth_user.is_some()))
}

async fn get_one_inner(
    log: &Logger,
    context: &ApiContext,
    path_params: ProjReportParams,
    auth_user: Option<&AuthUser>,
) -> Result<JsonReport, HttpError> {
    let conn = &mut *context.conn().await;

    let query_project =
        QueryProject::is_allowed_public(conn, &context.rbac, &path_params.project, auth_user)?;

    QueryReport::belonging_to(&query_project)
        .filter(schema::report::uuid.eq(path_params.report.to_string()))
        .first::<QueryReport>(conn)
        .map_err(resource_not_found_err!(
            Report,
            (&query_project, path_params.report)
        ))?
        .into_json(log, conn)
}

#[endpoint {
    method = DELETE,
    path =  "/v0/projects/{project}/reports/{report}",
    tags = ["projects", "reports"]
}]
pub async fn proj_report_delete(
    rqctx: RequestContext<ApiContext>,
    bearer_token: BearerToken,
    path_params: Path<ProjReportParams>,
) -> Result<ResponseDeleted, HttpError> {
    let auth_user = AuthUser::from_token(rqctx.context(), bearer_token).await?;
    delete_inner(rqctx.context(), path_params.into_inner(), &auth_user).await?;
    Ok(Delete::auth_response_deleted())
}

async fn delete_inner(
    context: &ApiContext,
    path_params: ProjReportParams,
    auth_user: &AuthUser,
) -> Result<(), HttpError> {
    let conn = &mut *context.conn().await;

    // Verify that the user is allowed
    let query_project = QueryProject::is_allowed(
        conn,
        &context.rbac,
        &path_params.project,
        auth_user,
        Permission::Delete,
    )?;

    let (report_id, version_id) = QueryReport::belonging_to(&query_project)
        .filter(schema::report::uuid.eq(path_params.report.to_string()))
        .select((schema::report::id, schema::report::version_id))
        .first::<(ReportId, VersionId)>(conn)
        .map_err(resource_not_found_err!(
            Report,
            (&query_project, path_params.report)
        ))?;
    diesel::delete(schema::report::table.filter(schema::report::id.eq(report_id)))
        .execute(conn)
        .map_err(resource_conflict_err!(Report, report_id))?;

    // If there are no more reports for this version, delete the version
    // This is necessary because multiple reports can use the same version via a git hash
    // This will cascade and delete all branch versions for this version
    // Before doing so, decrement all greater versions
    if schema::report::table
        .filter(schema::report::version_id.eq(version_id))
        .select(count(schema::report::id))
        .first::<i64>(conn)
        .map_err(resource_not_found_err!(
            Version,
            (&query_project, report_id, version_id)
        ))?
        == 0
    {
        let query_version = QueryVersion::get(conn, version_id)?;
        // Get all branches that use this version
        let branches = schema::branch::table
            .inner_join(schema::branch_version::table)
            .filter(schema::branch_version::version_id.eq(version_id))
            .select(schema::branch::id)
            .load::<BranchId>(conn)
            .map_err(resource_not_found_err!(
                Branch,
                (&query_project, report_id, version_id)
            ))?;

        let mut version_map = HashMap::new();
        // Get all versions greater than this one for each of the branches
        for branch_id in branches {
            schema::version::table
                .filter(schema::version::number.gt(query_version.number))
                .inner_join(schema::branch_version::table)
                .filter(schema::branch_version::branch_id.eq(branch_id))
                .select((schema::version::id, schema::version::number))
                .load::<(VersionId, VersionNumber)>(conn)
                .map_err(resource_not_found_err!(
                    Version,
                    (&query_project, report_id, branch_id, &query_version)
                ))?
                .into_iter()
                .for_each(|(version_id, version_number)| {
                    version_map.insert(version_id, version_number);
                });
        }

        // For each version greater than this one, decrement the version number
        for (version_id, version_number) in version_map {
            if let Err(e) =
                diesel::update(schema::version::table.filter(schema::version::id.eq(version_id)))
                    .set(schema::version::number.eq(version_number.decrement()))
                    .execute(conn)
            {
                debug_assert!(
                    false,
                    "Failed to decrement version ({version_id}) number ({version_number}): {e}"
                );
                #[cfg(feature = "sentry")]
                sentry::capture_error(&e);
            }
        }

        // Finally delete the dangling version
        diesel::delete(schema::version::table.filter(schema::version::id.eq(version_id)))
            .execute(conn)
            .map_err(resource_conflict_err!(
                Version,
                (&query_project, report_id, &query_version)
            ))?;
    }

    Ok(())
}
