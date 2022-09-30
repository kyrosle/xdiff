use std::str::FromStr;

use anyhow::Result;
use reqwest::{
    header::{self, HeaderMap, HeaderName, HeaderValue},
    Client, Response,
};
use serde_json::json;

use crate::{ExtraArgs, RequestProfile, ResponseProfile};

#[derive(Debug)]
pub struct ResponseExt(Response);

impl RequestProfile {
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
}

impl ResponseExt {
    pub async fn filter_text(self, profile: &ResponseProfile) -> Result<String> {
        let res = self.0;
        let mut output = String::new();
        output.push_str(&format!("{:?} {}\r", res.version(), res.status()));
        let headers = res.headers();

        for (k, v) in headers.iter() {
            if !profile.skip_headers.iter().any(|sh| sh == k.as_str()) {
                output.push_str(&format!("{}: {:?}", k, v));
            }
        }
        output.push('\n');

        let content_type = get_content_type(&headers);
        let text = res.text().await?;
        match content_type.as_deref() {
            Some("application/json") => {
                let text = filter_json(&text, &profile.skip_body)?;
                output.push_str(&text);
            }
            _ => {
                output.push_str(&text);
            }
        }
        Ok(output)
    }
}

fn filter_json(text: &str, skip: &[String]) -> Result<String> {
    let mut json: serde_json::Value = serde_json::from_str(text)?;

    #[allow(clippy::single_match)]
    match json {
        serde_json::Value::Object(ref mut obj) => {
            for k in skip {
                obj.remove(k);
            }
        }
        _ =>
            // For now we just ignore non-object values, we don't know how to filter. In future, we might support array of objects
            {}
    }

    Ok(serde_json::to_string_pretty(&json)?)
}

fn get_content_type(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().unwrap().split(';').next())
        .map(|v| v.to_string())
}
