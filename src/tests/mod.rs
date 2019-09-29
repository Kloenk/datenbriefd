#[test]
fn config_parse_time() {
    use super::{Company, Config};
    let mut config = Config::new();
    let test_company = Company {
        alias: String::new(),
        name: String::from("test"),
        interval: 0,
        mail: String::new(),
        next_hit: chrono::Utc::now(),
        onw_name: String::new(),
        reminder: 0,
    };
    config.companies.push(test_company);

    let json = r#"{"test":{"next":"2019-09-29T11:13:56.692549889+00:00","reminder":20}}"#;
    config.parse_time(json);

    assert_eq!(
        config.companies[0].next_hit,
        "2019-09-29T11:13:56.692549889+00:00"
            .parse::<chrono::DateTime<chrono::Utc>>()
            .unwrap()
    );
    assert_eq!(config.companies[0].reminder, 20);
}
