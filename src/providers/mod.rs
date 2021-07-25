mod cloudflare;
mod namesilo;

pub async fn update_dns_record(
    current_ip: &mut String,
    recorded_ip: &mut String,
    domain: &str,
    host: &str,
    api_key: &str,
    backend: &str,
) -> Result<bool, &'static str> {
    match backend {
        "cloudflare" => {
            cloudflare::update_dns_record(current_ip, recorded_ip, domain, host, api_key).await
        }
        "namesilo" => {
            namesilo::update_dns_record(current_ip, recorded_ip, domain, host, api_key).await
        }
        _ => Err("Unknown backend"),
    }
}
