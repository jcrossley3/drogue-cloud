use crate::management;
use serde::{Deserialize, Serialize};

/// Authenticate a device.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthenticationRequest {
    pub application: String,
    pub device: String,
    pub credential: Credential,
    pub r#as: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Credential {
    #[serde(rename = "user")]
    UsernamePassword { username: String, password: String },
    #[serde(rename = "pass")]
    Password(String),
    #[serde(rename = "cert")]
    Certificate(Vec<Vec<u8>>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// The authentication request passed. The outcome also contains application and device
    /// details for further processing.
    Pass {
        application: management::Application,
        device: management::Device,
    },
    /// The authentication request failed. The device is not authenticated, and the device's
    /// request must be rejected.
    Fail,
}

/// The result of an authentication request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuthenticationResponse {
    /// The outcome, if the request.
    pub outcome: Outcome,
}

impl AuthenticationResponse {
    pub fn failed() -> Self {
        Self {
            outcome: Outcome::Fail,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::{TimeZone, Utc};
    use serde_json::json;

    #[test]
    fn ser_credentials() {
        let ser = serde_json::to_value(vec![
            Credential::Password("foo".into()),
            Credential::UsernamePassword {
                username: "foo".into(),
                password: "bar".into(),
            },
        ]);
        assert!(ser.is_ok());
        assert_eq!(
            ser.unwrap(),
            json! {[
                {"pass": "foo"},
                {"user": {"username": "foo", "password": "bar"}}
            ]}
        )
    }

    #[test]
    fn test_encode_fail() {
        let str = serde_json::to_string(&AuthenticationResponse {
            outcome: Outcome::Fail,
        });
        assert!(str.is_ok());
        assert_eq!(String::from(r#"{"outcome":"fail"}"#), str.unwrap());
    }

    #[test]
    fn test_encode_pass() {
        let str = serde_json::to_string(&AuthenticationResponse {
            outcome: Outcome::Pass {
                application: management::Application {
                    metadata: management::NonScopedMetadata {
                        name: "a1".to_string(),
                        creation_timestamp: Utc.timestamp_millis(1000),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                device: management::Device {
                    metadata: management::ScopedMetadata {
                        application: "a1".to_string(),
                        name: "d1".to_string(),
                        creation_timestamp: Utc.timestamp_millis(1234),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        });

        assert!(str.is_ok());
        assert_eq!(
            String::from(
                r#"{"outcome":{"pass":{"application":{"metadata":{"name":"a1","creationTimestamp":"1970-01-01T00:00:01Z","generation":0}},"device":{"metadata":{"application":"a1","name":"d1","creationTimestamp":"1970-01-01T00:00:01.234Z","generation":0}}}}}"#
            ),
            str.unwrap()
        );
    }
}