use anyhow::{anyhow, Result};

use clap::{Parser, Subcommand};
use crate::ExtraArgs;


/// Diff two http requests and compare the difference between the response
#[derive(Parser, Debug)]
#[clap(version, author, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Subcommand, Debug, Clone)]
#[non_exhaustive]
pub enum Action {
    /// Diff two Api response based on given profile
    Run(RunArgs),
    /// Parse URLS to generate a Profile
    Parse,
}

#[derive(Parser, Debug, Clone)]
pub struct RunArgs {
    /// Profile name
    #[clap(short, long, value_parser)]
    pub profile: String,

    /// Override args. Could be used to override the query, headers and body of the request
    ///
    /// for query params, use `-e key=value`.
    ///
    /// for headers, use `-e %key:value`.
    ///
    /// for body, use `-e @key=value`.
    #[clap(short, value_parser = parse_key_val, number_of_values = 1)]
    pub extra_params: Vec<KeyVal>,

    /// Configuration to use.
    #[clap(short, long, value_parser)]
    pub config: Option<String>,
}

#[derive(Debug, Clone)]
pub enum KeyValType {
    Query,
    Header,
    Body,
}

#[derive(Debug, Clone)]
pub struct KeyVal {
    key_type: KeyValType,
    key: String,
    value: String,
}

pub fn parse_key_val(s: &str) -> Result<KeyVal> {
    let mut parts = s.splitn(2, '=');

    let key = parts
        .next()
        .ok_or_else(|| anyhow!("Invalid key value pair: {}", s))?
        .trim();
    let value = parts
        .next()
        .ok_or_else(|| anyhow!("Invalid key value pair: {}", s))?
        .trim();

    let (key_type, key) = match key.chars().next() {
        Some('%') => (KeyValType::Header, &key[1..]),
        Some('@') => (KeyValType::Body, &key[1..]),
        Some(v) if v.is_alphabetic() => (KeyValType::Query, key),
        _ => return Err(anyhow!("Invalid key value pair")),
    };

    Ok(KeyVal {
        key_type,
        key: key.to_string(),
        value: value.to_string(),
    })
}

impl From<Vec<KeyVal>> for ExtraArgs {
    fn from(args: Vec<KeyVal>) -> Self {
        let mut headers = vec![];
        let mut query = vec![];
        let mut body = vec![];

        for arg in args {
            match arg.key_type {
                KeyValType::Header => headers.push((arg.key, arg.value)),
                KeyValType::Query => query.push((arg.key, arg.value)),
                KeyValType::Body => body.push((arg.key, arg.value)),
            }
        }
        ExtraArgs {
            headers,
            query,
            body,
        }
    }
}
