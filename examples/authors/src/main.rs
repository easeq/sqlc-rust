use geo_types::line_string;
use postgresql_embedded::blocking::PostgreSQL;
use postgresql_embedded::Result;

#[path = "./db/gen.rs"]
pub mod db;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("postgresql/migrations");
}

fn main() -> Result<()> {
    let mut postgresql = PostgreSQL::default();
    postgresql.setup()?;
    postgresql.start()?;

    let database_name = "test";
    postgresql.create_database(database_name)?;

    let settings = postgresql.settings();
    let mut client = postgres::Client::connect(
        format!(
            "postgresql://{username}:{password}@{host}:{port}/{database_name}",
            host = settings.host,
            port = settings.port,
            username = settings.username,
            password = settings.password,
            database_name = database_name,
        )
        .as_str(),
        postgres::NoTls,
    )
    .unwrap();

    client.execute("CREATE extension hstore", &[]).unwrap();

    embedded::migrations::runner().run(&mut client).unwrap();

    let mut queries = db::Queries::new(client);

    let authors = queries.list_authors().unwrap();
    assert_eq!(authors.len(), 0);

    let author_res_err = queries.get_author(1).is_err();
    assert_eq!(author_res_err, true);

    let delete_res = queries.delete_author(1).is_ok();
    assert_eq!(delete_res, true);

    let author_full_req = db::CreateAuthorFullParams {
        name: "Author Full".to_string(),
        bio: None,
        genre: db::TypeGenre::CLaSSic,
        data: Some(serde_json::json!({
            "age":  50,
            "gender": "male",
        })),
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

    let author_full_res = queries.create_author_full(author_full_req.clone()).unwrap();
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
    println!("{author_full_res:?}");

    let delete_res = queries.delete_author(1).is_ok();
    assert_eq!(delete_res, true);

    let author1_req = db::CreateAuthorParams {
        name: "Author 1".to_string(),
        bio: None,
    };
    let author1_res = queries.create_author(author1_req.clone()).unwrap();
    assert_eq!(author1_res.name, author1_req.name);
    assert_eq!(author1_res.bio, author1_req.bio.clone());
    assert_eq!(author1_res.uuid, author1_res.uuid.clone());
    assert_ne!(author1_res.uuid, None);
    assert!(author1_res.id == 2);
    println!("{author1_res:?}");

    let mut authors_list_prepared = vec![author1_res.clone()];
    let authors = queries.list_authors().unwrap();
    assert_eq!(authors.len(), 1);
    assert_eq!(authors, authors_list_prepared);

    let author2_req = db::CreateAuthorParams {
        name: "Author 2".to_string(),
        bio: Some("My name is Author 2".to_string()),
    };
    let author2_res = queries.create_author(author2_req.clone()).unwrap();
    assert_eq!(author2_res.name, author2_req.name);
    assert_eq!(author2_res.bio, author2_req.bio);
    assert!(author2_res.id == 3);

    authors_list_prepared.push(author2_res.clone());

    let authors = queries.list_authors().unwrap();
    assert_eq!(authors.len(), 2);
    assert_eq!(authors, authors_list_prepared);

    let author = queries.get_author(2).unwrap();
    assert_eq!(author, author1_res);

    queries.delete_author(2).unwrap();
    let authors = queries.list_authors().unwrap();
    assert_eq!(authors.len(), 1);
    assert_eq!(authors, authors_list_prepared[1..]);

    postgresql.stop()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_all_queries() {
        crate::main().unwrap()
    }
}
