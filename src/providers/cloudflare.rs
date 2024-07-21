use crate::utils::{fetch_url_with_json, get_record_id_and_ip, HasSuccessField, RecordParsable};
use hyper::{header, HeaderMap};
use hyper::{Body, Method};
use serde::{Deserialize, Serialize};
use std::array::IntoIter;

#[derive(Serialize)]
struct RequestData {
    #[serde(rename = "type", default)]
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub ttl: u32,
    pub proxied: bool,
}

#[derive(Deserialize)]
struct ReplyData {
    pub success: bool,
    pub errors: Vec<ErrorRecord>,
    pub messages: Option<Vec<String>>,
    pub result_info: Option<ResultInfo>,
}

#[derive(Deserialize)]
struct Reply {
    pub result: Option<Vec<DnsRecord>>,
    #[serde(flatten)]
    pub reply_data: ReplyData,
}

#[derive(Deserialize)]
struct UpdateReply {
    pub result: Option<DnsRecord>,
    #[serde(flatten)]
    pub reply_data: ReplyData,
}

#[derive(Deserialize)]
struct DnsRecord {
    pub id: String,
    pub zone_id: String,
    pub zone_name: String,
    pub name: String,
    #[serde(rename = "type", default)]
    pub record_type: String,
    pub content: String,
    pub proxiable: bool,
    pub proxied: bool,
    pub ttl: u32,
    pub meta: DnsRecordMeta,
    pub created_on: String,
    pub modified_on: String,
}

#[derive(Deserialize)]
struct ErrorRecord {
    pub code: u32,
    pub message: String,
}

#[derive(Deserialize)]
struct ResultInfo {
    pub page: i32,
    pub per_page: i32,
    pub count: i32,
    pub total_count: i32,
    pub total_pages: i32,
}

#[derive(Deserialize)]
struct DnsRecordMeta {
    pub auto_added: bool,
    pub managed_by_apps: bool,
    pub managed_by_argo_tunnel: bool,
    pub source: Option<String>,
}

impl HasSuccessField for Reply {
    fn success(&self) -> bool {
        self.reply_data.success
    }
}

impl HasSuccessField for UpdateReply {
    fn success(&self) -> bool {
        self.reply_data.success
    }
}

impl RecordParsable for DnsRecord {
    fn name(&self) -> &String {
        &self.name
    }

    fn id(&self) -> &String {
        &self.id
    }

    fn ip(&self) -> &String {
        &self.content
    }
}

fn dns_list_zones_url(domain: &str) -> String {
    return format!(
        "https://api.cloudflare.com/client/v4/zones?name={}&status=active",
        domain
    );
}

fn dns_list_records_url(zone_id: &str) -> String {
    return format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records?type=A",
        zone_id
    );
}

fn dns_update_url(zone_id: &str, record_id: &str) -> String {
    return format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
        zone_id, record_id
    );
}

fn parse_zone_id(resp: serde_json::Value) -> Option<String> {
    let result = resp["result"].as_array()?;
    let record = result.first()?;
    let zone_id = record["id"].as_str()?;

    Some(String::from(zone_id))
}

fn update_dns_record_data(host: &str, domain: &str, ip: &str) -> String {
    let req = RequestData {
        record_type: String::from("A"),
        name: format!("{}{}", host, domain),
        content: String::from(ip),
        ttl: 1,
        proxied: false,
    };

    serde_json::to_string(&req).unwrap()
}

pub fn get_headers(api_key: &str) -> HeaderMap {
    IntoIter::new([
        (
            header::AUTHORIZATION,
            format!("Bearer {}", api_key).parse().unwrap(),
        ),
        (header::CONTENT_TYPE, "application/json".parse().unwrap()),
    ])
    .collect()
}

pub async fn update_dns_record(
    current_ip: &mut String,
    recorded_ip: &mut String,
    domain: &str,
    host: &str,
    api_key: &str,
) -> Result<bool, &'static str> {
    if current_ip == recorded_ip {
        return Ok(false);
    }

    let headers = get_headers(api_key);

    let zone_err = "Error parsing zone_id";
    let zone_url = dns_list_zones_url(domain);
    let zone_result: serde_json::Value =
        fetch_url_with_json(zone_url.as_str(), &headers, Method::GET, Body::default())
            .await
            .ok()
            .ok_or(zone_err)?;
    let zone_id = parse_zone_id(zone_result).ok_or(zone_err)?;

    let record_err = "Error parsing record_id";
    let query_url = dns_list_records_url(zone_id.as_str());
    let dns_result: Reply =
        fetch_url_with_json(query_url.as_str(), &headers, Method::GET, Body::default())
            .await
            .ok()
            .ok_or(record_err)?;

    if !dns_result.success() {
        return Err(record_err);
    }

    let record_not_found_err = "Cannot find record id, the sub domain is a new item";
    let resource_records = dns_result.result.ok_or(record_err)?;
    let record_id_and_ip =
        get_record_id_and_ip(&resource_records, domain, host).ok_or(record_not_found_err)?;

    let record_id = (record_id_and_ip.0).to_owned();
    *recorded_ip = (record_id_and_ip.1).to_owned();

    log::trace!("recorded ip: {}", recorded_ip);

    if current_ip == recorded_ip {
        return Ok(false);
    }

    log::info!("IP needs an update");
    let update_err = "IP update failed";
    let update_url = dns_update_url(&zone_id, &record_id);
    let update_data = update_dns_record_data(host, domain, current_ip.as_str());
    let update_result: UpdateReply = fetch_url_with_json(
        update_url.as_str(),
        &headers,
        Method::PUT,
        Body::from(update_data),
    )
    .await
    .ok()
    .ok_or(update_err)?;

    match update_result.success() {
        true => {
            *recorded_ip = current_ip.to_owned();
            Ok(true)
        }
        _ => Err(update_err),
    }
}
