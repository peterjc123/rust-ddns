use clap::{App, Arg, ArgMatches};
use hyper::{body::Buf, header::HeaderValue, Body, Client, HeaderMap, Method, Request};
use hyper_tls::HttpsConnector;
use public_ip::ToResolver;

mod internal {
    pub struct DDNSConfig {
        pub domain: String,
        pub api_key: String,
        pub backend: String,
    }
}

pub trait HasSuccessField {
    fn success(&self) -> bool;
}

pub trait RecordParsable {
    fn name(&self) -> &String;
    fn id(&self) -> &String;
    fn ip(&self) -> &String;
}

pub struct DDNSConfig {
    pub api_key: String,
    pub domain: String,
    pub host: String,
    pub backend: String,
}

impl HasSuccessField for serde_json::Value {
    fn success(&self) -> bool {
        let opt = self["success"].as_bool();
        opt.unwrap_or(false)
    }
}

impl From<internal::DDNSConfig> for DDNSConfig {
    fn from(config: internal::DDNSConfig) -> Self {
        let api_key = config.api_key;
        let backend = config.backend;
        let domain = config.domain;

        let dot_pos: Vec<_> = domain.match_indices('.').collect();
        let (domain, host) = if dot_pos.len() > 1 {
            let pos = dot_pos.first().unwrap().0;
            let domain = domain.as_str();
            (&domain[pos + 1..], &domain[..pos + 1])
        } else {
            (domain.as_str(), "")
        };

        let domain = domain.to_owned();
        let host = host.to_owned();

        DDNSConfig {
            api_key,
            domain,
            host,
            backend,
        }
    }
}

pub fn parse_args<'a>(required: bool) -> ArgMatches<'a> {
    App::new("rust_ddns")
        .version("0.1.0")
        .about("DDNS daemon helper written in Rust")
        .arg(
            Arg::with_name("key")
                .short("k")
                .long("key")
                .value_name("APIKEY")
                .required(required)
                .help("API key for your account")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("domain")
                .short("d")
                .long("domain")
                .value_name("DOMAIN")
                .required(required)
                .help("full domain (e.g. www.example.com)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("backend")
                .short("b")
                .long("backend")
                .value_name("BACKEND")
                .required(required)
                .help("Backend of the DDNS service (cloudflare or ddns)")
                .takes_value(true),
        )
        .get_matches()
}

pub fn parse_config() -> DDNSConfig {
    if cfg!(any(feature = "default-config")) {
        let matches = parse_args(false);
        internal::DDNSConfig {
            api_key: matches
                .value_of("key")
                .unwrap_or(include_str!("../config/api_key_default.txt"))
                .to_owned(),
            domain: matches
                .value_of("domain")
                .unwrap_or(include_str!("../config/domain_default.txt"))
                .to_owned(),
            backend: matches
                .value_of("backend")
                .unwrap_or(include_str!("../config/backend_default.txt"))
                .to_owned(),
        }
    } else {
        let matches = parse_args(true);
        internal::DDNSConfig {
            api_key: matches.value_of("key").unwrap().to_owned(),
            domain: matches.value_of("domain").unwrap().to_owned(),
            backend: matches.value_of("backend").unwrap().to_owned(),
        }
    }
    .into()
}

pub async fn fetch_url_with_json<T>(
    url: &str,
    headers: &HeaderMap<HeaderValue>,
    method: Method,
    body: Body,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
where
    T: serde::de::DeserializeOwned + HasSuccessField,
{
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let mut req = Request::builder()
        .method(method)
        .uri(url)
        .body(body)
        .unwrap();

    req.headers_mut().extend((*headers).clone());

    let res = client.request(req).await?;

    let body = hyper::body::aggregate(res).await?;
    let resp: T = serde_json::from_reader(body.reader()).unwrap();

    log::trace!("Success: {}", resp.success());

    Ok(resp)
}

#[allow(unused_variables)]
pub async fn fetch_url_with_xml<T>(
    url: &str,
    headers: &HeaderMap<HeaderValue>,
    method: Method,
    body: Body,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
where
    T: serde::de::DeserializeOwned + HasSuccessField,
{
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let mut req = Request::builder()
        .method(method)
        .uri(url)
        .body(body)
        .unwrap();

    req.headers_mut().extend((*headers).clone());

    let res = client.request(req).await?;

    let body = hyper::body::aggregate(res).await?;
    let resp: T = serde_xml_rs::from_reader(body.reader()).unwrap();

    log::trace!("Success: {}", resp.success());

    Ok(resp)
}

pub async fn get_current_ip() -> Option<String> {
    let resolver = vec![
        public_ip::BoxToResolver::new(public_ip::dns::OPENDNS_RESOLVER),
        public_ip::BoxToResolver::new(public_ip::dns::GOOGLE_DNS_TXT_RESOLVER),
    ]
    .to_resolver();

    public_ip::resolve_address(resolver)
        .await
        .map(|s| s.to_string())
}

pub fn get_record_id_and_ip<'a, T>(
    resource_records: &'a [T],
    domain: &str,
    host: &str,
) -> Option<(&'a String, &'a String)>
where
    T: RecordParsable,
{
    let sub_domain = format!("{}{}", host, domain);
    for resource_record in resource_records {
        if *resource_record.name() == sub_domain {
            let record_id = resource_record.id();
            let ip = resource_record.ip();
            return Some((record_id, ip));
        }
    }
    None
}

pub fn get_viewable_api_key(api_key: &str) -> String {
    let len = api_key.len();
    if len > 10 {
        format!("{}***{}", &api_key[..2], &api_key[len - 2..])
    } else {
        "*".repeat(7)
    }
}
