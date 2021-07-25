use crate::utils::{fetch_url_with_xml, get_record_id_and_ip, HasSuccessField, RecordParsable};
use hyper::{Body, HeaderMap, Method};
use serde::Deserialize;

#[derive(Deserialize)]
struct Content {
    pub request: Request,
    pub reply: Reply,
}

#[derive(Deserialize)]
struct Request {
    pub operation: String,
    pub ip: String,
}

#[derive(Deserialize)]
struct Reply {
    pub code: u32,
    pub detail: String,
    pub resource_record: Option<Vec<ResourceRecord>>,
    pub record_id: Option<String>,
}

#[derive(Deserialize)]
struct ResourceRecord {
    pub record_id: String,
    #[serde(rename = "type", default)]
    pub record_type: String,
    pub host: String,
    pub value: String,
    pub ttl: u32,
    pub distance: u32,
}

impl HasSuccessField for Content {
    fn success(&self) -> bool {
        self.reply.code == 200 || self.reply.code == 300
    }
}

impl RecordParsable for ResourceRecord {
    fn name(&self) -> &String {
        &self.host
    }

    fn id(&self) -> &String {
        &self.record_id
    }

    fn ip(&self) -> &String {
        &self.value
    }
}

fn dns_list_records_url(api_key: &str, domain: &str) -> String {
    return format!(
        "https://www.namesilo.com/api/dnsListRecords?version=1&type=xml&key={}&domain={}",
        api_key, domain
    );
}

fn dns_update_url(api_key: &str, domain: &str, record_id: &str, host: &str, ip: &str) -> String {
    let new_host = &host[..host.len() - 1];
    return format!("https://www.namesilo.com/api/dnsUpdateRecord?version=1&type=xml&key={}&domain={}&rrid={}&rrhost={}&rrvalue={}&rrttl=3600", api_key, domain, record_id, new_host, ip);
}

pub fn get_headers() -> HeaderMap {
    HeaderMap::new()
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

    let headers = get_headers();

    let record_err = "Error parsing record_id";
    let query_url = dns_list_records_url(api_key, domain);
    let dns_result: Content =
        fetch_url_with_xml(query_url.as_str(), &headers, Method::GET, Body::default())
            .await
            .ok()
            .ok_or(record_err)?;

    if !dns_result.success() {
        return Err(record_err);
    }

    let record_not_found_err = "Cannot find record id, the sub domain is a new item";
    let resource_records = dns_result.reply.resource_record.ok_or(record_err)?;
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
    let update_url = dns_update_url(
        api_key,
        domain,
        record_id.as_str(),
        host,
        current_ip.as_str(),
    );
    let update_result: Content =
        fetch_url_with_xml(update_url.as_str(), &headers, Method::GET, Body::default())
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
