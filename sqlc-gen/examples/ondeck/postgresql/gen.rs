#[derive(Debug, Display)]
pub enum Status {
    Open,
    Closed,
}
const LIST_CITIES: &str = r#"
select slug, name
from city
order by name
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct ListCitiesRow {
    pub(crate) slug: String,
    pub(crate) name: String,
}
const GET_CITY: &str = r#"
select slug, name
from city
where slug = $1
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct GetCityParams {
    pub(crate) slug: String,
}
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct GetCityRow {
    pub(crate) slug: String,
    pub(crate) name: String,
}
const CREATE_CITY: &str = r#"
INSERT INTO city (
    name,
    slug
) VALUES (
    $1,
    $2
) RETURNING slug, name
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct CreateCityParams {
    pub(crate) name: String,
    pub(crate) slug: String,
}
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct CreateCityRow {
    pub(crate) slug: String,
    pub(crate) name: String,
}
const UPDATE_CITY_NAME: &str = r#"
UPDATE city
SET name = $2
WHERE slug = $1
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct UpdateCityNameParams {
    pub(crate) slug: String,
    pub(crate) name: String,
}
const LIST_VENUES: &str = r#"
SELECT id, status, statuses, slug, name, city, spotify_playlist, songkick_id, tags, created_at
FROM venue
WHERE city = $1
ORDER BY name
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct ListVenuesParams {
    pub(crate) city: String,
}
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct ListVenuesRow {
    pub(crate) id: u16,
    pub(crate) status: String,
    pub(crate) statuses: Option<Vec<String>>,
    pub(crate) slug: String,
    pub(crate) name: String,
    pub(crate) city: String,
    pub(crate) spotify_playlist: String,
    pub(crate) songkick_id: Option<String>,
    pub(crate) tags: Option<Vec<String>>,
    pub(crate) created_at: String,
}
const DELETE_VENUE: &str = r#"
DELETE FROM venue
WHERE slug = $1 AND slug = $1
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct DeleteVenueParams {
    pub(crate) slug: String,
}
const GET_VENUE: &str = r#"
SELECT id, status, statuses, slug, name, city, spotify_playlist, songkick_id, tags, created_at
FROM venue
WHERE slug = $1 AND city = $2
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct GetVenueParams {
    pub(crate) slug: String,
    pub(crate) city: String,
}
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct GetVenueRow {
    pub(crate) id: u16,
    pub(crate) status: String,
    pub(crate) statuses: Option<Vec<String>>,
    pub(crate) slug: String,
    pub(crate) name: String,
    pub(crate) city: String,
    pub(crate) spotify_playlist: String,
    pub(crate) songkick_id: Option<String>,
    pub(crate) tags: Option<Vec<String>>,
    pub(crate) created_at: String,
}
const CREATE_VENUE: &str = r#"
INSERT INTO venue (
    slug,
    name,
    city,
    created_at,
    spotify_playlist,
    status,
    statuses,
    tags
) VALUES (
    $1,
    $2,
    $3,
    NOW(),
    $4,
    $5,
    $6,
    $7
) RETURNING id
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct CreateVenueParams {
    pub(crate) slug: String,
    pub(crate) name: String,
    pub(crate) city: String,
    pub(crate) spotify_playlist: String,
    pub(crate) status: String,
    pub(crate) statuses: Option<Vec<String>>,
    pub(crate) tags: Option<Vec<String>>,
}
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct CreateVenueRow {
    pub(crate) id: u16,
}
const UPDATE_VENUE_NAME: &str = r#"
UPDATE venue
SET name = $2
WHERE slug = $1
RETURNING id
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct UpdateVenueNameParams {
    pub(crate) slug: String,
    pub(crate) name: String,
}
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct UpdateVenueNameRow {
    pub(crate) id: u16,
}
const VENUE_COUNT_BY_CITY: &str = r#"
SELECT
    city,
    count(*)
FROM venue
GROUP BY 1
ORDER BY 1
"#;
#[derive(Debug, Display, sqlc_derive::FromPostgresRow)]
pub(crate) struct VenueCountByCityRow {
    pub(crate) city: String,
    pub(crate) count: i64,
}
pub struct Queries {
    client: postgres::Client,
}
impl Queries {
    pub fn new(
        host: &str,
        port: &str,
        username: &str,
        password: &str,
        database_name: &str,
    ) -> Self {
        let client = postgres::Client::connect(
                format!(
                    "postgresql://{username}:{password}@{host}:{port}/{database_name}",
                    host = settings.host, port = settings.port, username = settings
                    .username, password = settings.password, database_name =
                    database_name,
                )
                    .as_str(),
                postgres::NoTls,
            )
            .unwrap();
        Self { client }
    }
    pub fn list_cities(
        &self,
        params: ListCitiesParams,
    ) -> anyhow::Result<Vec<ListCitiesRow>> {
        let rows = self.client.query(LIST_CITIES, &[])?;
        let result: Vec<ListCitiesRow> = vec![];
        for row in rows {
            result.push(row.into());
        }
        Ok(result)
    }
    pub fn get_city(&self, params: GetCityParams) -> anyhow::Result<GetCityRow> {
        let row: GetCityRow = self.client.query_one(GET_CITY, &[&params.slug])?;
        Ok(row)
    }
    pub fn create_city(
        &self,
        params: CreateCityParams,
    ) -> anyhow::Result<CreateCityRow> {
        let row: CreateCityRow = self
            .client
            .query_one(CREATE_CITY, &[&params.name, &params.slug])?;
        Ok(row)
    }
    pub fn update_city_name(&self, params: UpdateCityNameParams) -> anyhow::Result<()> {
        self.client.execute(UPDATE_CITY_NAME, &[&params.slug, &params.name])?;
        Ok(())
    }
    pub fn list_venues(
        &self,
        params: ListVenuesParams,
    ) -> anyhow::Result<Vec<ListVenuesRow>> {
        let rows = self.client.query(LIST_VENUES, &[&params.city])?;
        let result: Vec<ListVenuesRow> = vec![];
        for row in rows {
            result.push(row.into());
        }
        Ok(result)
    }
    pub fn delete_venue(&self, params: DeleteVenueParams) -> anyhow::Result<()> {
        self.client.execute(DELETE_VENUE, &[&params.slug])?;
        Ok(())
    }
    pub fn get_venue(&self, params: GetVenueParams) -> anyhow::Result<GetVenueRow> {
        let row: GetVenueRow = self
            .client
            .query_one(GET_VENUE, &[&params.slug, &params.city])?;
        Ok(row)
    }
    pub fn create_venue(
        &self,
        params: CreateVenueParams,
    ) -> anyhow::Result<CreateVenueRow> {
        let row: CreateVenueRow = self
            .client
            .query_one(
                CREATE_VENUE,
                &[
                    &params.slug,
                    &params.name,
                    &params.city,
                    &params.spotify_playlist,
                    &params.status,
                    &params.statuses,
                    &params.tags,
                ],
            )?;
        Ok(row)
    }
    pub fn update_venue_name(
        &self,
        params: UpdateVenueNameParams,
    ) -> anyhow::Result<UpdateVenueNameRow> {
        let row: UpdateVenueNameRow = self
            .client
            .query_one(UPDATE_VENUE_NAME, &[&params.slug, &params.name])?;
        Ok(row)
    }
    pub fn venue_count_by_city(
        &self,
        params: VenueCountByCityParams,
    ) -> anyhow::Result<Vec<VenueCountByCityRow>> {
        let rows = self.client.query(VENUE_COUNT_BY_CITY, &[])?;
        let result: Vec<VenueCountByCityRow> = vec![];
        for row in rows {
            result.push(row.into());
        }
        Ok(result)
    }
}
