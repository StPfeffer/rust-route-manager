use async_trait::async_trait;
use bigdecimal::BigDecimal;
use sqlx::Error;
use uuid::Uuid;

use crate::{
    dtos::route::SaveRouteParamsDTO,
    models::route::{Route, RouteStatus},
};

use super::client::DBClient;

#[async_trait]
pub trait RouteExt {
    async fn get_route(&self, route_id: Option<Uuid>) -> Result<Option<Route>, sqlx::Error>;

    async fn list_routes(&self, page: u32, limit: usize) -> Result<Vec<Route>, sqlx::Error>;

    async fn save_route<B: Into<BigDecimal> + Send, S: Into<String> + Send>(
        &self,
        params: SaveRouteParamsDTO<B, S>,
    ) -> Result<Route, sqlx::Error>;

    async fn delete_route(&self, route_id: Option<Uuid>) -> Result<Option<Route>, sqlx::Error>;
}

#[async_trait]
impl RouteExt for DBClient {
    async fn get_route(&self, route_id: Option<Uuid>) -> Result<Option<Route>, sqlx::Error> {
        if let Some(route_id) = route_id {
            let vehicle = sqlx::query_as!(Route, r#"SELECT * FROM routes WHERE id = $1"#, route_id)
                .fetch_optional(&self.pool)
                .await?;
            return Ok(vehicle);
        }

        Ok(None)
    }

    async fn list_routes(&self, page: u32, limit: usize) -> Result<Vec<Route>, sqlx::Error> {
        let offset = (page - 1) * limit as u32;

        let routes = sqlx::query_as!(
            Route,
            r#"SELECT * FROM routes LIMIT $1 OFFSET $2"#,
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(routes)
    }

    async fn save_route<B: Into<BigDecimal> + Send, S: Into<String> + Send>(
        &self,
        params: SaveRouteParamsDTO<B, S>,
    ) -> Result<Route, sqlx::Error> {
        let SaveRouteParamsDTO {
            initial_lat,
            initial_long,
            final_lat,
            final_long,
            // driver_id,
            status_id,
            initial_address_id,
            final_address_id,
            vehicle_id,
        } = params;

        let initial_address_id = initial_address_id
            .map(|id| Uuid::parse_str(&id.into()))
            .transpose()
            .map_err(|e| Error::Protocol(format!("Failed to parse initial_address_id: {}", e)))?;

        let final_address_id = final_address_id
            .map(|id| Uuid::parse_str(&id.into()))
            .transpose()
            .map_err(|e| Error::Protocol(format!("Failed to parse final_address_id: {}", e)))?;

        let vehicle_id = Uuid::parse_str(&vehicle_id.into())
            .map_err(|e| Error::Protocol(format!("Failed to parse vehicle_id: {}", e)))?;

        let status_id = Uuid::parse_str(&status_id.into())
            .map_err(|e| Error::Protocol(format!("Failed to parse status_id: {}", e)))?;

        let route = sqlx::query_as!(
            Route,
            r#"
            INSERT INTO routes (initial_lat, initial_long, final_lat, final_long, initial_address_id, final_address_id, vehicle_id, status_id) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
            RETURNING *"#,
            &initial_lat.into(),
            &initial_long.into(),
            &final_lat.map(Into::into).unwrap_or_default(),
            &final_long.map(Into::into).unwrap_or_default(),
            initial_address_id,
            final_address_id,
            &vehicle_id,
            &status_id,
            // Uuid::parse_str(&driver_id.into()).unwrap(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(route)
    }

    async fn delete_route(&self, route_id: Option<Uuid>) -> Result<Option<Route>, sqlx::Error> {
        let mut route = None;

        if let Some(route_id) = route_id {
            route = sqlx::query_as!(
                Route,
                r#"DELETE FROM routes WHERE id = $1 RETURNING *"#,
                route_id
            )
            .fetch_optional(&self.pool)
            .await?;
        }

        Ok(route)
    }
}

#[async_trait]
pub trait RouteStatusExt {
    async fn get_route_status(
        &self,
        status_id: Option<Uuid>,
        code: Option<String>,
    ) -> Result<Option<RouteStatus>, sqlx::Error>;

    async fn list_route_status(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<RouteStatus>, sqlx::Error>;

    async fn save_route_status<T: Into<String> + Send>(
        &self,
        code: Option<T>,
        description: T,
    ) -> Result<RouteStatus, sqlx::Error>;

    async fn delete_route_status(
        &self,
        status_id: Option<Uuid>,
    ) -> Result<Option<RouteStatus>, sqlx::Error>;
}

#[async_trait]
impl RouteStatusExt for DBClient {
    async fn get_route_status(
        &self,
        status_id: Option<Uuid>,
        code: Option<String>,
    ) -> Result<Option<RouteStatus>, sqlx::Error> {
        let mut status = None;

        if let Some(status_id) = status_id {
            status = sqlx::query_as!(
                RouteStatus,
                r#"SELECT * FROM route_status WHERE id = $1"#,
                status_id
            )
            .fetch_optional(&self.pool)
            .await?;
        } else if let Some(code) = code {
            status = sqlx::query_as!(
                RouteStatus,
                r#"SELECT * FROM route_status WHERE code = $1"#,
                code
            )
            .fetch_optional(&self.pool)
            .await?;
        }

        Ok(status)
    }

    async fn list_route_status(
        &self,
        page: u32,
        limit: usize,
    ) -> Result<Vec<RouteStatus>, sqlx::Error> {
        let offset = (page - 1) * limit as u32;

        let statuses = sqlx::query_as!(
            RouteStatus,
            r#"SELECT * FROM route_status LIMIT $1 OFFSET $2"#,
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(statuses)
    }

    async fn save_route_status<T: Into<String> + Send>(
        &self,
        code: Option<T>,
        description: T,
    ) -> Result<RouteStatus, sqlx::Error> {
        let state = sqlx::query_as!(
            RouteStatus,
            r#"INSERT INTO route_status (code, description) VALUES ($1, $2) RETURNING *"#,
            &code.map(Into::into).unwrap_or_default(),
            &description.into(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(state)
    }

    async fn delete_route_status(
        &self,
        status_id: Option<Uuid>,
    ) -> Result<Option<RouteStatus>, sqlx::Error> {
        let mut status = None;

        if let Some(status_id) = status_id {
            status = sqlx::query_as!(
                RouteStatus,
                r#"DELETE FROM route_status WHERE id = $1 RETURNING *"#,
                status_id
            )
            .fetch_optional(&self.pool)
            .await?;
        }

        Ok(status)
    }
}
