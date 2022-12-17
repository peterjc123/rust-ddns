mod providers;
mod utils;

use log::LevelFilter;
use providers::update_dns_record;
use simple_logger::SimpleLogger;
use time::UtcOffset;
use utils::{get_current_ip, get_viewable_api_key, parse_config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = parse_config();

    let (api_key, domain, host, backend) = (
        config.api_key.as_str(),
        config.domain.as_str(),
        config.host.as_str(),
        config.backend.as_str(),
    );

    let retry_period_on_err = 15;
    let retry_period_on_success = 15;
    let retry_period_on_skip = 15;

    let mut recorded_ip: String = String::new();
    let mut current_ip: String = String::new();

    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_utc_offset(UtcOffset::from_hms(8, 0, 0).unwrap())
        .init()
        .unwrap();

    log::info!(
        "Daemon started with following info: api_key: {}, domain: {}{}, backend: {}",
        get_viewable_api_key(api_key),
        host,
        domain,
        backend
    );

    loop {
        let need_update = get_current_ip().await.map(|s| current_ip = s).is_some();

        let update_result = if need_update {
            log::info!("current ip: {}", current_ip);
            update_dns_record(
                &mut current_ip,
                &mut recorded_ip,
                domain,
                host,
                api_key,
                backend,
            )
            .await
        } else {
            Err("Get current ip failed")
        };

        let retry_period = match update_result {
            Ok(res) => {
                if res {
                    log::info!(
                        "IP update success, will check again in {} min",
                        retry_period_on_success
                    );
                    retry_period_on_success
                } else {
                    log::info!(
                        "No need for an update, will retry in {} min",
                        retry_period_on_skip
                    );
                    retry_period_on_skip
                }
            }
            Err(e) => {
                log::error!("{}, will retry in {} min", e, retry_period_on_err);
                retry_period_on_err
            }
        };

        tokio::time::sleep(tokio::time::Duration::from_secs(retry_period * 60)).await;
    }
}
