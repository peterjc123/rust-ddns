# Rust-DDNS
A DDNS daemon written in Rust

## Supported providers
- Cloudflare
- Namesilo

## How to use
### Build
```bash
cargo build --release
```

### Command line options
```bash
USAGE:
    rust-ddns.exe [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --backend <BACKEND>    Backend of the DDNS service (cloudflare or ddns)
    -d, --domain <DOMAIN>      full domain (e.g. www.example.com)
    -k, --key <APIKEY>         API key for your account
```

## Working as a service
### Windows
1. Install [NSSM](https://nssm.cc/download)
2. Open an commmand prompt with elevated permission
3. Type in the following commands

    ```cmd
    cd %PROJECT_ROOT%
    mkdir %PROJECT_ROOT%/logs

    nssm.exe install rust-ddns %CD%\target\release\rust-ddns.exe
    nssm.exe set rust-ddns AppDirectory %CD%\target\release
    nssm.exe set rust-ddns AppExit Default Restart
    nssm.exe set rust-ddns AppStdout %CD%\logs\stdout.txt
    nssm.exe set rust-ddns AppStderr %CD%\logs\stderr.txt
    nssm.exe set rust-ddns AppRotateFiles 1
    nssm.exe set rust-ddns AppRotateOnline 1
    nssm.exe set rust-ddns AppRotateBytes 50000
    nssm.exe set rust-ddns Description "A DDNS service written in Rust"
    nssm.exe set rust-ddns DisplayName "Rust DDNS"
    nssm.exe set rust-ddns ObjectName LocalSystem
    nssm.exe set rust-ddns Start SERVICE_AUTO_START
    nssm.exe set rust-ddns Type SERVICE_WIN32_OWN_PROCESS
    ```

### Linux (Systemd)
Create a service description file `rust-ddns.service` under `/etc/systemd`.

```conf
[Unit]

Description=A DDNS daemon written in Rust

After=network.target

Wants=network-online.target

[Service]

Restart=always

Type=simple

ExecStart=/project/target/release/rust-ddns

Environment=

[Install]

WantedBy=multi-user.target
```

Then, `service rust-ddns start` could be used to start this application as a service. Finally, you may use `systemctl enable rust-ddns` to enable the service automatically on reboot.

## Advanced usage
### Build with the default configuration
This is enabled as a feature in rust. Following are the steps to enable it.
1. Writing down the configurations in the following files

    ```xxx
    APIKEY -> config/api_key_default.txt
    BACKEND -> config/backend_default.txt
    DOMAIN -> config/domain_default.txt
    ```

2. Build with the feature enabled

    ```bash
    cargo build --release --features=default-config
    ```

## Future work
1. IPV6 support
2. Multiple domains
3. Config file
