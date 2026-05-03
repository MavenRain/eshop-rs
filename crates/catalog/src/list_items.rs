//! `GET /api/catalog/items` handler.
//!
//! Pagination follows upstream's `PaginationRequest` shape:
//! `?page_size=&page_index=` query parameters with the same defaults
//! (10, 0).  Slicing happens at the API layer rather than in the
//! repository because toasty does not yet expose `LIMIT`/`OFFSET`
//! primitives in its query DSL.

use axum::Json;
use axum::extract::{Query, State};

use crate::error::Error;
use crate::repository::CatalogItemRepository;
use crate::request::PaginationQuery;
use crate::response::{CatalogItemResponse, PaginatedItemsResponse};
use crate::state::AppState;

pub async fn handle(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedItemsResponse<CatalogItemResponse>>, Error> {
    // FFI-mut exception: see `create_item::handle`.
    let mut conn = state.db().connection().await?;
    let mut tx = conn.transaction().await?;
    let items = CatalogItemRepository::list_all(&mut tx).await?;
    tx.commit().await?;

    let page_size = query.page_size();
    let page_index = query.page_index();
    let count = u64::try_from(items.len()).map_err(|e| Error::NumericRange {
        context: format!("catalog item count out of u64 range: {e}"),
    })?;
    let skip = usize::try_from(page_index)
        .map_err(|e| Error::NumericRange {
            context: format!("page_index out of usize range: {e}"),
        })?
        .saturating_mul(usize::try_from(page_size).map_err(|e| Error::NumericRange {
            context: format!("page_size out of usize range: {e}"),
        })?);
    let take = usize::try_from(page_size).map_err(|e| Error::NumericRange {
        context: format!("page_size out of usize range: {e}"),
    })?;
    let data: Vec<CatalogItemResponse> = items
        .iter()
        .skip(skip)
        .take(take)
        .map(CatalogItemResponse::from)
        .collect();
    Ok(Json(PaginatedItemsResponse::new(
        page_index, page_size, count, data,
    )))
}
