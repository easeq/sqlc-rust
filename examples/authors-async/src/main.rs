use geo_types::line_string;
use itertools::Itertools;
use postgresql_embedded::{PostgreSQL, Result};

#[path = "./db/gen.rs"]
pub mod db;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("postgresql/migrations");
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut postgresql = PostgreSQL::default();
    postgresql.setup().await?;
    postgresql.start().await?;

    let database_name = "test";
    postgresql.create_database(database_name).await?;

    let settings = postgresql.settings();
    let conn_str = format!(
        "postgresql://{username}:{password}@{host}:{port}/{database_name}",
        host = settings.host,
        port = settings.port,
        username = settings.username,
        password = settings.password,
        database_name = database_name,
    );
    let (mut db_client, connection) = tokio_postgres::connect(&conn_str, tokio_postgres::NoTls)
        .await
        .expect("failed to connect to db");

    tokio::task::spawn(async move {
        let _ = connection.await;
    });

    db_client
        .execute("CREATE extension hstore", &[])
        .await
        .unwrap();

    embedded::migrations::runner()
        .run_async(&mut db_client)
        .await
        .expect("failed to load migrations");

    let authors = db::list_authors(&db_client).await.unwrap();
    assert_eq!(authors.try_len().unwrap(), 0);

    let author_res_err = db::get_author(&db_client, 1).await.is_err();
    assert_eq!(author_res_err, true);

    let delete_res = db::delete_author(&db_client, 1).await.is_ok();
    assert_eq!(delete_res, true);

    let author_full_req = db::CreateAuthorFullParams {
        name: "Author Full".to_string(),
        bio: None,
        data: Some(serde_json::json!({
            "age":  50,
            "gender": "male",
        })),
        genre: db::TypeGenre::CLaSSic,
        attrs: Some(
            [
                ("attr_1".to_string(), Some("attr1 value".to_string())),
                ("attr_2".to_string(), None),
            ]
            .into_iter()
            .collect(),
        ),
        ip_inet: cidr::IpInet::V6(
            "2001:DB8:1234:5678::/64"
                .parse::<cidr::Ipv6Inet>()
                .expect("ip_inet init failed"),
        ),
        ip_cidr: cidr::IpCidr::V6(cidr::Ipv6Cidr::new_host(core::net::Ipv6Addr::UNSPECIFIED)),
        mac_address: eui48::MacAddress::parse_str("01-02-03-0A-0b-0f").expect("Parse error {}"),
        geo_point: Some(geo_types::point! { x: 1., y: 181.2 }),
        geo_rect: Some(geo_types::Rect::new(
            geo_types::coord! { x: 10., y: 20. },
            geo_types::coord! { x: 30., y: 10. },
        )),
        geo_path: Some(line_string![
        (x: -21.95156, y: 64.1446),
        (x: -21.951, y: 64.14479),
        (x: -21.95044, y: 64.14527),
        (x: -21.951445, y: 64.145508),
        ]),
        bit_a: Some(bit_vec::BitVec::from_elem(3, false)),
        varbit_a: Some(bit_vec::BitVec::from_elem(4, false)),
        created_at: time::OffsetDateTime::now_utc(),
        updated_at: time::OffsetDateTime::now_utc(),
    };
    let author_full_res = db::create_author_full(&db_client, author_full_req.clone())
        .await
        .unwrap();
    assert_eq!(author_full_res.name, author_full_req.name);
    assert_eq!(author_full_res.bio, author_full_req.bio);
    assert_ne!(author_full_res.uuid, None);
    assert_eq!(author_full_res.data, author_full_req.data);
    assert_eq!(author_full_res.genre, author_full_req.genre);
    assert_eq!(author_full_res.attrs, author_full_req.attrs);
    assert_eq!(author_full_res.ip_inet, author_full_req.ip_inet);
    assert_eq!(author_full_res.ip_cidr, author_full_req.ip_cidr);
    assert_eq!(author_full_res.mac_address, author_full_req.mac_address);
    assert_eq!(author_full_res.geo_point, author_full_req.geo_point);
    assert_eq!(author_full_res.geo_rect, author_full_req.geo_rect);
    assert_eq!(author_full_res.geo_path, author_full_req.geo_path);
    assert_eq!(author_full_res.bit_a, author_full_req.bit_a);
    assert_eq!(author_full_res.varbit_a, author_full_req.varbit_a);
    assert_eq!(
        author_full_res.created_at.to_hms_milli(),
        author_full_req.created_at.to_hms_milli()
    );
    assert_eq!(
        author_full_res.updated_at.to_hms_micro(),
        author_full_req.updated_at.to_hms_micro()
    );
    assert!(author_full_res.id == 1);
    println!("{author_full_res:#?}");

    let delete_res = db::delete_author(&db_client, 1).await.is_ok();
    assert_eq!(delete_res, true);

    let author1_req = db::CreateAuthorParams {
        name: "Author 1".to_string(),
        bio: None,
    };
    let author1_res = db::create_author(&db_client, author1_req.clone())
        .await
        .unwrap();
    assert_eq!(author1_res.name, author1_req.name);
    assert_eq!(author1_res.bio, author1_req.bio.clone());
    assert!(author1_res.id == 2);

    let mut authors_list_prepared = vec![author1_res.clone()];
    let authors: Vec<_> = db::list_authors(&db_client)
        .await
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(authors.len(), 1);
    assert_eq!(authors, authors_list_prepared);

    let author2_req = db::CreateAuthorParams {
        name: "Author 2".to_string(),
        bio: Some("My name is Author 2".to_string()),
    };
    let author2_res = db::create_author(&db_client, author2_req.clone())
        .await
        .unwrap();
    assert_eq!(author2_res.name, author2_req.name);
    assert_eq!(author2_res.bio, author2_req.bio);
    assert!(author2_res.id == 3);

    authors_list_prepared.push(author2_res.clone());

    let authors: Vec<_> = db::list_authors(&db_client)
        .await
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(authors.len(), 2);
    assert_eq!(authors, authors_list_prepared);

    let author = db::get_author(&db_client, 2).await.unwrap();
    assert_eq!(author, author1_res);

    db::delete_author(&db_client, 2).await.unwrap();
    let authors: Vec<_> = db::list_authors(&db_client)
        .await
        .unwrap()
        .try_collect()
        .unwrap();
    assert_eq!(authors.len(), 1);
    assert_eq!(authors, authors_list_prepared[1..]);

    postgresql.stop().await
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_all_queries() {
        crate::main().unwrap()
    }
}
