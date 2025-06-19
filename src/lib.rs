use crate::exports::edgee::components::data_collection::{Dict, EdgeeRequest, Event, HttpMethod};
use anyhow::Context;
use base64::prelude::*;
use exports::edgee::components::data_collection::Guest;
use std::collections::HashMap;

wit_bindgen::generate!({
    world: "data-collection",
    path: ".edgee/wit",
    additional_derives: [serde::Serialize],
    generate_all,
});
export!(Component);

struct Component;

impl Guest for Component {
    fn page(edgee_event: Event, settings: Dict) -> Result<EdgeeRequest, String> {
        send_to_clickhouse(edgee_event, settings)
    }

    fn track(edgee_event: Event, settings: Dict) -> Result<EdgeeRequest, String> {
        send_to_clickhouse(edgee_event, settings)
    }

    fn user(edgee_event: Event, settings: Dict) -> Result<EdgeeRequest, String> {
        send_to_clickhouse(edgee_event, settings)
    }
}

fn send_to_clickhouse(edgee_event: Event, settings_dict: Dict) -> Result<EdgeeRequest, String> {
    let settings = Settings::new(settings_dict).map_err(|e| e.to_string())?;

    // serialize the entire event into JSON
    let body = serde_json::to_string(&edgee_event).unwrap_or_default();

    // insert json objects into ClickHouse table using the JSONEachRow format
    let url = format!(
        "{}?query=INSERT INTO {}.{} FORMAT JSONEachRow",
        settings.endpoint, settings.database, settings.table
    );

    let authorization_basic = format!(
        "Basic {}",
        BASE64_STANDARD.encode(format!("{}:{}", settings.username, settings.password))
    );

    Ok(EdgeeRequest {
        method: HttpMethod::Post,
        url,
        headers: vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("Authorization".to_string(), authorization_basic.to_string()),
        ],
        body,
        forward_client_headers: false,
    })
}

const DEFAULT_DATABASE: &str = "default";
const DEFAULT_USERNAME: &str = "default";

pub struct Settings {
    pub endpoint: String,
    pub database: String,
    pub table: String,
    pub username: String,
    pub password: String,
}

impl Settings {
    pub fn new(settings_dict: Dict) -> anyhow::Result<Self> {
        let settings_map: HashMap<String, String> = settings_dict
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        let endpoint = settings_map
            .get("endpoint")
            .context("Missing endpoint")?
            .to_string();

        let database = settings_map
            .get("database")
            .cloned()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| DEFAULT_DATABASE.to_owned());

        let table = settings_map
            .get("table")
            .context("Missing table")?
            .to_string();

        let username = settings_map
            .get("username")
            .cloned()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| DEFAULT_USERNAME.to_owned());

        let password = settings_map
            .get("password")
            .context("Missing password")?
            .to_string();

        Ok(Self {
            endpoint,
            database,
            table,
            username,
            password,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::exports::edgee::components::data_collection::{
        Campaign, Client, Context, Data, EventType, PageData, Session, TrackData, UserData,
    };
    use exports::edgee::components::data_collection::Consent;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    fn sample_user_data(edgee_id: String) -> UserData {
        UserData {
            user_id: "123".to_string(),
            anonymous_id: "456".to_string(),
            edgee_id,
            properties: vec![
                ("prop1".to_string(), "value1".to_string()),
                ("prop2".to_string(), "10".to_string()),
            ],
        }
    }

    fn sample_context(edgee_id: String, locale: String, session_start: bool) -> Context {
        Context {
            page: sample_page_data(),
            user: sample_user_data(edgee_id),
            client: Client {
                city: "Paris".to_string(),
                ip: "192.168.0.1".to_string(),
                locale,
                timezone: "CET".to_string(),
                user_agent: "Chrome".to_string(),
                user_agent_architecture: "unknown".to_string(),
                user_agent_bitness: "64".to_string(),
                user_agent_full_version_list: "abc".to_string(),
                user_agent_version_list: "abc".to_string(),
                user_agent_mobile: "mobile".to_string(),
                user_agent_model: "unknown".to_string(),
                os_name: "MacOS".to_string(),
                os_version: "latest".to_string(),
                screen_width: 1024,
                screen_height: 768,
                screen_density: 2.0,
                continent: "Europe".to_string(),
                country_code: "FR".to_string(),
                country_name: "France".to_string(),
                region: "West Europe".to_string(),
            },
            campaign: Campaign {
                name: "random".to_string(),
                source: "random".to_string(),
                medium: "random".to_string(),
                term: "random".to_string(),
                content: "random".to_string(),
                creative_format: "random".to_string(),
                marketing_tactic: "random".to_string(),
            },
            session: Session {
                session_id: "random".to_string(),
                previous_session_id: "random".to_string(),
                session_count: 2,
                session_start,
                first_seen: 123,
                last_seen: 123,
            },
        }
    }

    fn sample_page_data() -> PageData {
        PageData {
            name: "page name".to_string(),
            category: "category".to_string(),
            keywords: vec!["value1".to_string(), "value2".into()],
            title: "page title".to_string(),
            url: "https://example.com/full-url?test=1".to_string(),
            path: "/full-path".to_string(),
            search: "?test=1".to_string(),
            referrer: "https://example.com/another-page".to_string(),
            properties: vec![
                ("prop1".to_string(), "value1".to_string()),
                ("prop2".to_string(), "10".to_string()),
                ("currency".to_string(), "USD".to_string()),
            ],
        }
    }

    fn sample_track_data(event_name: String) -> TrackData {
        return TrackData {
            name: event_name,
            products: vec![],
            properties: vec![
                ("prop1".to_string(), "value1".to_string()),
                ("prop2".to_string(), "10".to_string()),
                ("currency".to_string(), "USD".to_string()),
            ],
        };
    }

    fn sample_page_event(
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        session_start: bool,
    ) -> Event {
        Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::Page,
            data: Data::Page(sample_page_data()),
            context: sample_context(edgee_id, locale, session_start),
            consent,
        }
    }

    fn sample_track_event(
        event_name: String,
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        session_start: bool,
    ) -> Event {
        return Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::Track,
            data: Data::Track(sample_track_data(event_name)),
            context: sample_context(edgee_id, locale, session_start),
            consent: consent,
        };
    }

    fn sample_user_event(
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        session_start: bool,
    ) -> Event {
        return Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::User,
            data: Data::User(sample_user_data(edgee_id.clone())),
            context: sample_context(edgee_id, locale, session_start),
            consent: consent,
        };
    }

    #[test]
    fn page_works_fine() {
        let event = sample_page_event(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let settings = vec![
            (
                "endpoint".to_string(),
                "https://XYZ.eu-west-1.aws.clickhouse.cloud:8443".to_string(),
            ),
            ("database".to_string(), "test".to_string()),
            ("table".to_string(), "edgee".to_string()),
            ("username".to_string(), "user".to_string()),
            ("password".to_string(), "12345".to_string()),
        ];
        let result = Component::page(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.is_empty(), false);
        assert_eq!(edgee_request.url.contains("clickhouse.cloud"), true); // hostname
        assert_eq!(edgee_request.url.contains("test.edgee"), true); // database.table
        assert_eq!(edgee_request.url.contains("?query="), true); // query param
    }

    #[test]
    fn track_works_fine() {
        let event = sample_track_event(
            "test_event".to_string(),
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let settings = vec![
            (
                "endpoint".to_string(),
                "https://XYZ.eu-west-1.aws.clickhouse.cloud:8443".to_string(),
            ),
            ("database".to_string(), "test".to_string()),
            ("table".to_string(), "edgee".to_string()),
            ("username".to_string(), "user".to_string()),
            ("password".to_string(), "12345".to_string()),
        ];
        let result = Component::track(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.is_empty(), false);
        assert_eq!(edgee_request.url.contains("clickhouse.cloud"), true); // hostname
        assert_eq!(edgee_request.url.contains("test.edgee"), true); // database.table
        assert_eq!(edgee_request.url.contains("?query="), true); // query param
    }

    #[test]
    fn user_works_fine() {
        let event = sample_user_event(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let settings = vec![
            (
                "endpoint".to_string(),
                "https://XYZ.eu-west-1.aws.clickhouse.cloud:8443".to_string(),
            ),
            ("database".to_string(), "test".to_string()),
            ("table".to_string(), "edgee".to_string()),
            ("username".to_string(), "user".to_string()),
            ("password".to_string(), "12345".to_string()),
        ];
        let result = Component::user(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.is_empty(), false);
        assert_eq!(edgee_request.url.contains("clickhouse.cloud"), true); // hostname
        assert_eq!(edgee_request.url.contains("test.edgee"), true); // database.table
        assert_eq!(edgee_request.url.contains("?query="), true); // query param
    }

    #[test]
    fn test_default_database() {
        let event = sample_page_event(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let settings = vec![
            (
                "endpoint".to_string(),
                "https://XYZ.eu-west-1.aws.clickhouse.cloud:8443".to_string(),
            ),
            ("table".to_string(), "edgee".to_string()),
            ("password".to_string(), "12345".to_string()),
        ];
        let result = Component::page(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.url.contains("default.edgee"), true); // database.table
    }

    #[test]
    fn page_authorization_header_with_username() {
        let event = sample_page_event(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let settings = vec![
            (
                "endpoint".to_string(),
                "https://XYZ.eu-west-1.aws.clickhouse.cloud:8443".to_string(),
            ),
            ("table".to_string(), "edgee".to_string()),
            ("username".to_string(), "user".to_string()),
            ("password".to_string(), "12345".to_string()),
        ];
        let result = Component::page(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();

        let auth_header = edgee_request
            .headers
            .iter()
            .find(|(k, _)| k == "Authorization")
            .map(|(_, v)| v);

        assert!(auth_header.is_some());
        assert!(auth_header.unwrap().starts_with("Basic "));
        let expected_auth = BASE64_STANDARD.encode(format!("{}:{}", "user", "12345"));
        assert_eq!(auth_header.unwrap(), &format!("Basic {}", expected_auth));
    }

    #[test]
    fn page_authorization_header_without_username() {
        let event = sample_page_event(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let settings = vec![
            (
                "endpoint".to_string(),
                "https://XYZ.eu-west-1.aws.clickhouse.cloud:8443".to_string(),
            ),
            ("table".to_string(), "edgee".to_string()),
            ("password".to_string(), "12345".to_string()),
        ];
        let result = Component::page(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();

        let auth_header = edgee_request
            .headers
            .iter()
            .find(|(k, _)| k == "Authorization")
            .map(|(_, v)| v);

        assert!(auth_header.is_some());
        assert!(auth_header.unwrap().starts_with("Basic "));
        let expected_auth = BASE64_STANDARD.encode(format!("{}:{}", "default", "12345"));
        assert_eq!(auth_header.unwrap(), &format!("Basic {}", expected_auth));
    }
}
