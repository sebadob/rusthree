extern crate base64;
extern crate md5;

use bytes::Bytes;
use futures::TryStreamExt;
use reqwest::Body;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;
use time::OffsetDateTime;

use super::request_trait::{Request, ResponseData, ResponseDataStream};
use crate::bucket::Bucket;
use crate::command::Command;
use crate::command::HttpMethod;
use crate::error::S3Error;

use tokio_stream::StreamExt;

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

// Temporary structure for making a request
pub struct HttpRequest<'a> {
    pub bucket: &'a Bucket,
    pub path: &'a str,
    pub command: Command<'a>,
    pub datetime: OffsetDateTime,
}

#[async_trait::async_trait]
impl<'a> Request for HttpRequest<'a> {
    type Response = reqwest::Response;
    type HeaderMap = http::header::HeaderMap;

    async fn response(&self) -> Result<reqwest::Response, S3Error> {
        let headers = match self.headers().await {
            Ok(headers) => headers,
            Err(e) => return Err(e),
        };

        let client = CLIENT.get_or_init(|| {
            let mut builder = reqwest::Client::builder()
                .brotli(true)
                .connect_timeout(Duration::from_secs(10))
                .http2_keep_alive_interval(Duration::from_secs(30))
                .http2_keep_alive_while_idle(true)
                .pool_idle_timeout(Duration::from_secs(600))
                .use_rustls_tls();
            if cfg!(feature = "no-verify-ssl") {
                builder = builder.danger_accept_invalid_certs(true);
            }
            builder.build().unwrap()
        });

        let url = self.url()?;
        let url = url.as_str();
        let req_builder = match self.command.http_verb() {
            HttpMethod::Delete => client.delete(url),
            HttpMethod::Get => client.get(url),
            HttpMethod::Post => client.post(url),
            HttpMethod::Put => client.put(url),
            HttpMethod::Head => client.head(url),
        };

        let response = req_builder
            .headers(headers)
            .body(Body::from(self.request_body()))
            .send()
            .await?;

        if cfg!(feature = "fail-on-err") && !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(S3Error::HttpFailWithBody(status, text));
        }

        Ok(response)
    }

    async fn response_data(&self, etag: bool) -> Result<ResponseData, S3Error> {
        let response = self.response().await?;
        let status_code = response.status().as_u16();
        let headers = response.headers();
        let response_headers = headers
            .iter()
            .map(|(k, v)| {
                (
                    k.to_string(),
                    v.to_str()
                        .unwrap_or("could-not-decode-header-value")
                        .to_string(),
                )
            })
            .collect::<HashMap<String, String>>();
        let body_vec = if etag {
            if let Some(etag) = headers.get("ETag") {
                Bytes::from(etag.to_str()?.to_string())
            } else {
                Bytes::from("")
            }
        } else {
            response.bytes().await?
        };
        Ok(ResponseData::new(body_vec, status_code, response_headers))
    }

    async fn response_data_to_writer<T: tokio::io::AsyncWrite + Send + Unpin>(
        &self,
        writer: &mut T,
    ) -> Result<u16, S3Error> {
        use tokio::io::AsyncWriteExt;
        let response = self.response().await?;

        let status_code = response.status();
        let mut stream = response.bytes_stream().into_stream();

        while let Some(item) = stream.next().await {
            writer.write_all(&item?).await?;
        }

        Ok(status_code.as_u16())
    }

    async fn response_data_to_stream(&self) -> Result<ResponseDataStream, S3Error> {
        let response = self.response().await?;
        let status_code = response.status();
        let stream = response
            .bytes_stream()
            .into_stream()
            .map_err(S3Error::Reqwest);

        Ok(ResponseDataStream {
            bytes: Box::pin(stream),
            status_code: status_code.as_u16(),
        })
    }

    fn datetime(&self) -> OffsetDateTime {
        self.datetime
    }

    fn bucket(&self) -> &Bucket {
        self.bucket
    }

    fn command(&self) -> &Command {
        &self.command
    }

    fn path(&self) -> &str {
        self.path
    }
}

impl<'a> HttpRequest<'a> {
    pub async fn new(
        bucket: &'a Bucket,
        path: &'a str,
        command: Command<'a>,
    ) -> Result<HttpRequest<'a>, S3Error> {
        // bucket.credentials_refresh().await?;
        Ok(Self {
            bucket,
            path,
            command,
            datetime: OffsetDateTime::now_utc(),
        })
    }
}
