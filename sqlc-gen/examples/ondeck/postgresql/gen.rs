/// @generated by the sqlc-gen-rust on sqlc-generate using sqlc.yaml
/// DO NOT EDIT.
pub(crate) const CREATE_CITY: &str = r#"
INSERT INTO city (
    name,
    slug
) VALUES (
    $1,
    $2
) RETURNING slug, name
"#;
pub(crate) const CREATE_VENUE: &str = r#"
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
pub(crate) const DELETE_VENUE: &str = r#"
DELETE FROM venue
WHERE slug = $1 AND slug = $1
"#;
pub(crate) const GET_CITY: &str = r#"
select slug, name
from city
where slug = $1
"#;
pub(crate) const GET_VENUE: &str = r#"
SELECT id, status, statuses, slug, name, city, spotify_playlist, songkick_id, tags, created_at
FROM venue
WHERE slug = $1 AND city = $2
"#;
pub(crate) const LIST_CITIES: &str = r#"
select slug, name
from city
order by name
"#;
pub(crate) const LIST_VENUES: &str = r#"
SELECT id, status, statuses, slug, name, city, spotify_playlist, songkick_id, tags, created_at
FROM venue
WHERE city = $1
ORDER BY name
"#;
pub(crate) const UPDATE_CITY_NAME: &str = r#"
UPDATE city
SET name = $2
WHERE slug = $1
"#;
pub(crate) const UPDATE_VENUE_NAME: &str = r#"
UPDATE venue
SET name = $2
WHERE slug = $1
RETURNING id
"#;
pub(crate) const VENUE_COUNT_BY_CITY: &str = r#"
SELECT
    city,
    count(*)
FROM venue
GROUP BY 1
ORDER BY 1
"#;
#[derive(Clone, Debug, PartialEq, postgres_derive::ToSql, postgres_derive::FromSql)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
#[postgres(name = "status")]
pub enum Status {
    #[postgres(name = "op!en")]
    #[cfg_attr(feature = "serde_support", serde(rename = "op!en"))]
    Open,
    #[postgres(name = "clo@sed")]
    #[cfg_attr(feature = "serde_support", serde(rename = "clo@sed"))]
    Closed,
}
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct City {
    pub slug: String,
    pub name: String,
}
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct CreateCityParams {
    pub name: String,
    pub slug: String,
}
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct CreateVenueParams {
    pub slug: String,
    pub name: String,
    pub city: String,
    pub spotify_playlist: String,
    pub status: Status,
    pub statuses: Option<Vec<Status>>,
    pub tags: Option<Vec<String>>,
}
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct GetVenueParams {
    pub slug: String,
    pub city: String,
}
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct UpdateCityNameParams {
    pub slug: String,
    pub name: String,
}
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct UpdateVenueNameParams {
    pub slug: String,
    pub name: String,
}
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct Venue {
    pub id: i32,
    pub status: Status,
    pub statuses: Option<Vec<Status>>,
    pub slug: String,
    pub name: String,
    pub city: String,
    pub spotify_playlist: String,
    pub songkick_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub created_at: String,
}
#[derive(Clone, Debug, sqlc_core::FromPostgresRow, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "hash", derive(Eq, Hash))]
pub(crate) struct VenueCountByCityRow {
    pub city: String,
    pub count: i64,
}
pub(crate) async fn create_city(
    client: &impl sqlc_core::DBTX,
    arg: CreateCityParams,
) -> sqlc_core::Result<City> {
    let row = client.query_one(CREATE_CITY, &[&arg.name, &arg.slug]).await?;
    Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
}
pub(crate) async fn create_venue(
    client: &impl sqlc_core::DBTX,
    arg: CreateVenueParams,
) -> sqlc_core::Result<i32> {
    let row = client
        .query_one(
            CREATE_VENUE,
            &[
                &arg.slug,
                &arg.name,
                &arg.city,
                &arg.spotify_playlist,
                &arg.status,
                &arg.statuses,
                &arg.tags,
            ],
        )
        .await?;
    Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
}
pub(crate) async fn delete_venue(
    client: &impl sqlc_core::DBTX,
    slug: String,
) -> sqlc_core::Result<()> {
    client.execute(DELETE_VENUE, &[&slug]).await?;
    Ok(())
}
pub(crate) async fn get_city(
    client: &impl sqlc_core::DBTX,
    slug: String,
) -> sqlc_core::Result<City> {
    let row = client.query_one(GET_CITY, &[&slug]).await?;
    Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
}
pub(crate) async fn get_venue(
    client: &impl sqlc_core::DBTX,
    arg: GetVenueParams,
) -> sqlc_core::Result<Venue> {
    let row = client.query_one(GET_VENUE, &[&arg.slug, &arg.city]).await?;
    Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
}
pub(crate) async fn list_cities(
    client: &impl sqlc_core::DBTX,
) -> sqlc_core::Result<impl std::iter::Iterator<Item = sqlc_core::Result<City>>> {
    let rows = client.query(LIST_CITIES, &[]).await?;
    let iter = rows
        .into_iter()
        .map(|row| Ok(sqlc_core::FromPostgresRow::from_row(&row)?));
    Ok(iter)
}
pub(crate) async fn list_venues(
    client: &impl sqlc_core::DBTX,
    city: String,
) -> sqlc_core::Result<impl std::iter::Iterator<Item = sqlc_core::Result<Venue>>> {
    let rows = client.query(LIST_VENUES, &[&city]).await?;
    let iter = rows
        .into_iter()
        .map(|row| Ok(sqlc_core::FromPostgresRow::from_row(&row)?));
    Ok(iter)
}
pub(crate) async fn update_city_name(
    client: &impl sqlc_core::DBTX,
    arg: UpdateCityNameParams,
) -> sqlc_core::Result<()> {
    client.execute(UPDATE_CITY_NAME, &[&arg.slug, &arg.name]).await?;
    Ok(())
}
pub(crate) async fn update_venue_name(
    client: &impl sqlc_core::DBTX,
    arg: UpdateVenueNameParams,
) -> sqlc_core::Result<i32> {
    let row = client.query_one(UPDATE_VENUE_NAME, &[&arg.slug, &arg.name]).await?;
    Ok(sqlc_core::FromPostgresRow::from_row(&row)?)
}
pub(crate) async fn venue_count_by_city(
    client: &impl sqlc_core::DBTX,
) -> sqlc_core::Result<
    impl std::iter::Iterator<Item = sqlc_core::Result<VenueCountByCityRow>>,
> {
    let rows = client.query(VENUE_COUNT_BY_CITY, &[]).await?;
    let iter = rows
        .into_iter()
        .map(|row| Ok(sqlc_core::FromPostgresRow::from_row(&row)?));
    Ok(iter)
}
