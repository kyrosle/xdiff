use std::str::FromStr;

use anyhow::{anyhow, Error, Result};
use reqwest::{
    header::{self, HeaderMap, HeaderName, HeaderValue},
    Client, Method, Response, Url,
};
use serde_json::json;
use std::fmt::Write;

use crate::{ExtraArgs, RequestProfile, ResponseProfile};

#[derive(Debug)]
pub struct ResponseExt(Response);

impl RequestProfile {
    pub fn new(
        method: Method,
        url: Url,
        params: Option<serde_json::Value>,
        headers: HeaderMap,
        body: Option<serde_json::Value>,
    ) -> Self {
        Self {
            method,
            url,
            params,
            headers,
            body,
        }
    }
    pub async fn send(&self, args: &ExtraArgs) -> Result<ResponseExt> {
        // let req = Client::new().request(self.method.clone(), self.url.clone());
        let (headers, query, body) = self.generate(args)?;
        let client = Client::new();
        // let url = self.url.clone().set_query(query);
        // println!("{:?}", args);
        // println!("{:?}", self.url.clone());
        let req = client
            .request(self.method.clone(), self.url.clone())
            .query(&query)
            .headers(headers)
            .body(body)
            .build()?;
        let res = client.execute(req).await?;
        Ok(ResponseExt(res))
    }
    pub fn generate(&self, args: &ExtraArgs) -> Result<(HeaderMap, serde_json::Value, String)> {
        let mut headers = self.headers.clone();
        let mut query = self.params.clone().unwrap_or_else(|| json!({}));
        let mut body = self.body.clone().unwrap_or_else(|| json!({}));

        for (k, v) in &args.headers {
            headers.insert(HeaderName::from_str(k)?, HeaderValue::from_str(v)?);
        }
        if !headers.contains_key(header::CONTENT_TYPE) {
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
        }

        for (k, v) in &args.query {
            query[k] = v.parse()?;
        }

        for (k, v) in &args.body {
            body[k] = v.parse()?;
        }

        let content_type = get_content_type(&headers);

        match content_type.as_deref() {
            Some("application/json") => {
                let body = serde_json::to_string(&body)?;
                Ok((headers, query, body))
            }
            Some("application/x-ww-form-urlencoded" | "multipart/from-data") => {
                let body = serde_urlencoded::to_string(&body)?;
                Ok((headers, query, body))
            }
            _ => Err(anyhow::anyhow!("unsupported content-type")),
        }
    }
    pub(crate) fn validate(&self) -> Result<()> {
        if let Some(params) = self.params.as_ref() {
            if !params.is_object() {
                return Err(anyhow!(
                    "Params must be a object but got:\n{}",
                    serde_yaml::to_string(&params)?
                ));
            }
        }
        if let Some(body) = self.body.as_ref() {
            if !body.is_object() {
                return Err(anyhow!(
                    "Body must be a object but got:\n{}",
                    serde_yaml::to_string(&body)?
                ));
            }
        }
        Ok(())
    }
}

impl FromStr for RequestProfile {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let mut url = Url::parse(s)?;
        let qs = url.query_pairs();
        let mut params = json!({});
        for (k, v) in qs {
            params[&*k] = v.parse()?;
        }
        url.set_query(None);
        Ok(RequestProfile::new(
            Method::GET,
            url,
            Some(params),
            HeaderMap::new(),
            None,
        ))
    }
}

impl ResponseExt {
    pub async fn filter_text(self, profile: &ResponseProfile) -> Result<String> {
        let res = self.0;

        let mut output = get_header_text(&res, &profile.skip_headers)?;

        let content_type = get_content_type(res.headers());
        let text = res.text().await?;
        match content_type.as_deref() {
            Some("application/json") => {
                let text = filter_json(&text, &profile.skip_body)?;
                writeln!(&mut output, "{}", text)?;
            }
            _ => {
                writeln!(&mut output, "{}", text)?;
            }
        }
        Ok(output)
    }
    pub fn get_header_keys(&self) -> Vec<String> {
        let res = &self.0;
        let headers = res.headers();
        headers
            .iter()
            .map(|(k, _)| k.as_str().to_string())
            .collect()
    }
}

fn get_header_text(res: &Response, skip_headers: &[String]) -> Result<String> {
    let mut output = String::new();
    writeln!(&mut output, "{:?} {}", res.version(), res.status())?;
    let headers = res.headers();

    for (k, v) in headers.iter() {
        if !skip_headers.iter().any(|sh| sh == k.as_str()) {
            writeln!(&mut output, "{}: {:?}", k, v)?;
        }
    }

    writeln!(&mut output)?;
    Ok(output)
}

fn filter_json(text: &str, skip: &[String]) -> Result<String> {
    let mut json: serde_json::Value = serde_json::from_str(text)?;

    if let serde_json::Value::Object(ref mut obj) = json {
        for k in skip {
            obj.remove(k);
        }
    }

    Ok(serde_json::to_string_pretty(&json)?)
}

fn get_content_type(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().unwrap().split(';').next())
        .map(|v| v.to_string())
}
