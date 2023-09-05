use crate::error::CredentialsError;
use log::debug;
use serde::{Deserialize, Serialize};
use std::env;
use std::ops::Deref;
use time::OffsetDateTime;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Credentials {
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub security_token: Option<String>,
    pub session_token: Option<String>,
    pub expiration: Option<Rfc3339OffsetDateTime>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Rfc3339OffsetDateTime(#[serde(with = "time::serde::rfc3339")] pub OffsetDateTime);

impl From<OffsetDateTime> for Rfc3339OffsetDateTime {
    fn from(v: OffsetDateTime) -> Self {
        Self(v)
    }
}

impl From<Rfc3339OffsetDateTime> for OffsetDateTime {
    fn from(v: Rfc3339OffsetDateTime) -> Self {
        v.0
    }
}

impl Deref for Rfc3339OffsetDateTime {
    type Target = OffsetDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AssumeRoleWithWebIdentityResponse {
    pub assume_role_with_web_identity_result: AssumeRoleWithWebIdentityResult,
    pub response_metadata: ResponseMetadata,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AssumeRoleWithWebIdentityResult {
    pub subject_from_web_identity_token: String,
    pub audience: String,
    pub assumed_role_user: AssumedRoleUser,
    pub credentials: StsResponseCredentials,
    pub provider: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct StsResponseCredentials {
    pub session_token: String,
    pub secret_access_key: String,
    pub expiration: Rfc3339OffsetDateTime,
    pub access_key_id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AssumedRoleUser {
    pub arn: String,
    pub assumed_role_id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ResponseMetadata {
    pub request_id: String,
}

// // The global request timeout in milliseconds. 0 means no timeout.
// // Defaults to 30 seconds.
// static REQUEST_TIMEOUT_MS: AtomicU32 = AtomicU32::new(30_000);
//
// /// Sets the timeout for all credentials HTTP requests and returns the
// /// old timeout value, if any; this timeout applies after a 30-second
// /// connection timeout.
// ///
// /// Short durations are bumped to one millisecond, and durations
// /// greater than 4 billion milliseconds (49 days) are rounded up to
// /// infinity (no timeout).
// /// The global default value is 30 seconds.
// #[cfg(feature = "http-credentials")]
// pub fn set_request_timeout(timeout: Option<Duration>) -> Option<Duration> {
//     use std::convert::TryInto;
//     let duration_ms = timeout
//         .as_ref()
//         .map(Duration::as_millis)
//         .unwrap_or(u128::MAX)
//         .max(1); // A 0 duration means infinity.
//
//     // Store that non-zero u128 value in an AtomicU32 by mapping large
//     // values to 0: `http_get` maps that to no (infinite) timeout.
//     let prev = REQUEST_TIMEOUT_MS.swap(duration_ms.try_into().unwrap_or(0), Ordering::Relaxed);
//
//     if prev == 0 {
//         None
//     } else {
//         Some(Duration::from_millis(prev as u64))
//     }
// }
//
// /// Sends a GET request to `url` with a request timeout if one was set.
// #[cfg(feature = "http-credentials")]
// fn http_get(url: &str) -> attohttpc::Result<attohttpc::Response> {
//     let mut builder = attohttpc::get(url);
//
//     let timeout_ms = REQUEST_TIMEOUT_MS.load(Ordering::Relaxed);
//     if timeout_ms > 0 {
//         builder = builder.timeout(Duration::from_millis(timeout_ms as u64));
//     }
//
//     builder.send()
// }

impl Credentials {
    // compatibility reasons
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Result<Self, CredentialsError> {
        Self::from_env()
    }

    pub fn refresh(&mut self) -> Result<(), CredentialsError> {
        if let Some(expiration) = self.expiration {
            if expiration.0 <= OffsetDateTime::now_utc() {
                debug!("Refreshing credentials!");
                let refreshed = Credentials::default()?;
                *self = refreshed
            }
        }
        Ok(())
    }

    // #[cfg(feature = "http-credentials")]
    // pub fn from_sts_env(session_name: &str) -> Result<Credentials, CredentialsError> {
    //     let role_arn = env::var("AWS_ROLE_ARN")?;
    //     let web_identity_token_file = env::var("AWS_WEB_IDENTITY_TOKEN_FILE")?;
    //     let web_identity_token = std::fs::read_to_string(web_identity_token_file)?;
    //     Credentials::from_sts(&role_arn, session_name, &web_identity_token)
    // }
    //
    // #[cfg(feature = "http-credentials")]
    // pub fn from_sts(
    //     role_arn: &str,
    //     session_name: &str,
    //     web_identity_token: &str,
    // ) -> Result<Credentials, CredentialsError> {
    //     let url = Url::parse_with_params(
    //         "https://sts.amazonaws.com/",
    //         &[
    //             ("Action", "AssumeRoleWithWebIdentity"),
    //             ("RoleSessionName", session_name),
    //             ("RoleArn", role_arn),
    //             ("WebIdentityToken", web_identity_token),
    //             ("Version", "2011-06-15"),
    //         ],
    //     )?;
    //     let response = http_get(url.as_str())?;
    //     let serde_response =
    //         quick_xml::de::from_str::<AssumeRoleWithWebIdentityResponse>(&response.text()?)?;
    //     // assert!(quick_xml::de::from_str::<AssumeRoleWithWebIdentityResponse>(&response.text()?).unwrap());
    //
    //     Ok(Credentials {
    //         access_key: Some(
    //             serde_response
    //                 .assume_role_with_web_identity_result
    //                 .credentials
    //                 .access_key_id,
    //         ),
    //         secret_key: Some(
    //             serde_response
    //                 .assume_role_with_web_identity_result
    //                 .credentials
    //                 .secret_access_key,
    //         ),
    //         security_token: None,
    //         session_token: Some(
    //             serde_response
    //                 .assume_role_with_web_identity_result
    //                 .credentials
    //                 .session_token,
    //         ),
    //         expiration: Some(
    //             serde_response
    //                 .assume_role_with_web_identity_result
    //                 .credentials
    //                 .expiration,
    //         ),
    //     })
    // }
    //
    // #[cfg(feature = "http-credentials")]
    // #[allow(clippy::should_implement_trait)]
    // pub fn default() -> Result<Credentials, CredentialsError> {
    //     Credentials::new(None, None, None, None, None)
    // }

    // pub fn anonymous() -> Result<Credentials, CredentialsError> {
    //     Ok(Credentials {
    //         access_key: None,
    //         secret_key: None,
    //         security_token: None,
    //         session_token: None,
    //         expiration: None,
    //         last_refresh: OffsetDateTime::now_utc(),
    //     })
    // }

    // /// Initialize Credentials directly with key ID, secret key, and optional
    // /// token.
    // #[cfg(feature = "http-credentials")]
    // pub fn new(
    //     access_key: Option<&str>,
    //     secret_key: Option<&str>,
    //     security_token: Option<&str>,
    //     session_token: Option<&str>,
    //     profile: Option<&str>,
    // ) -> Result<Credentials, CredentialsError> {
    //     if access_key.is_some() {
    //         return Ok(Credentials {
    //             access_key: access_key.map(|s| s.to_string()),
    //             secret_key: secret_key.map(|s| s.to_string()),
    //             security_token: security_token.map(|s| s.to_string()),
    //             session_token: session_token.map(|s| s.to_string()),
    //             expiration: None,
    //             last_refresh: OffsetDateTime::now_utc(),
    //         });
    //     }
    //
    //     Credentials::from_sts_env("aws-creds")
    //         .or_else(|_| Credentials::from_env())
    //         .or_else(|_| Credentials::from_profile(profile))
    //         .or_else(|_| Credentials::from_instance_metadata())
    //         .or_else(|_| {
    //             panic!(
    //                 "Could not get valid credentials from STS, ENV, Profile or Instance metadata"
    //             )
    //         })
    // }

    pub fn from_env_specific(
        access_key_var: Option<&str>,
        secret_key_var: Option<&str>,
        security_token_var: Option<&str>,
        session_token_var: Option<&str>,
    ) -> Result<Credentials, CredentialsError> {
        let access_key = from_env_with_default(access_key_var, "S3_ACCESS_KEY_ID")?;
        let secret_key = from_env_with_default(secret_key_var, "S3_ACCESS_KEY_SECRET")?;

        let security_token = from_env_with_default(security_token_var, "S3_SECURITY_TOKEN").ok();
        let session_token = from_env_with_default(session_token_var, "S3_SESSION_TOKEN").ok();
        Ok(Credentials {
            access_key: Some(access_key),
            secret_key: Some(secret_key),
            security_token,
            session_token,
            expiration: None,
        })
    }

    pub fn from_env() -> Result<Credentials, CredentialsError> {
        Credentials::from_env_specific(None, None, None, None)
    }

    // #[cfg(feature = "http-credentials")]
    // pub fn from_instance_metadata() -> Result<Credentials, CredentialsError> {
    //     let resp: CredentialsFromInstanceMetadata =
    //         match env::var("AWS_CONTAINER_CREDENTIALS_RELATIVE_URI") {
    //             Ok(credentials_path) => {
    //                 // We are on ECS
    //                 attohttpc::get(format!("http://169.254.170.2{}", credentials_path))
    //                     .send()?
    //                     .json()?
    //             }
    //             Err(_) => {
    //                 if !is_ec2() {
    //                     return Err(CredentialsError::NotEc2);
    //                 }
    //
    //                 let role = attohttpc::get(
    //                     "http://169.254.169.254/latest/meta-data/iam/security-credentials",
    //                 )
    //                 .send()?
    //                 .text()?;
    //
    //                 attohttpc::get(format!(
    //                     "http://169.254.169.254/latest/meta-data/iam/security-credentials/{}",
    //                     role
    //                 ))
    //                 .send()?
    //                 .json()?
    //             }
    //         };
    //
    //     Ok(Credentials {
    //         access_key: Some(resp.access_key_id),
    //         secret_key: Some(resp.secret_access_key),
    //         security_token: Some(resp.token),
    //         expiration: Some(resp.expiration),
    //         session_token: None,
    //     })
    // }
}

fn from_env_with_default(var: Option<&str>, default: &str) -> Result<String, CredentialsError> {
    let val = var.unwrap_or(default);
    env::var(val)
        .or_else(|_e| env::var(val))
        .map_err(|_| CredentialsError::MissingEnvVar(val.to_string(), default.to_string()))
}
