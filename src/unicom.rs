use anyhow::Result;
use clap::Subcommand;
use md5::{Digest, Md5};
use tracing::{debug, trace};
const PID: &str = "2";
const CALG: &str = "12345678";

fn hash(pwd: &str) -> String {
    let raw = format!("{}{}{}", PID, pwd, CALG);
    let hashed = Md5::new().chain_update(raw).finalize();
    let result = format!(
        "{}{}{}",
        hashed.iter().fold(String::new(), |mut acc, x| {
            acc.push_str(&format!("{:02x}", x));
            acc
        }),
        CALG,
        PID
    );
    result
}

#[derive(Subcommand, Debug)]
pub enum Unicom {
    Login {
        host: Option<String>,
        username: Option<String>,
        password: Option<String>,
    },
    Logout {
        host: Option<String>,
    },
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Config {
    host: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

async fn login(host: &str, username: &str, password: &str) -> Result<()> {
    use reqwest::{header::HeaderValue, Client};
    debug!(
        "host: {}, username: {}, password: {}",
        host, username, password
    );
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        HeaderValue::from_static("curl/8.10.1"),
    );
    let url = format!("http://{}", host);
    let pwd = hash(password);
    let client = Client::builder().http1_title_case_headers().build()?;
    let res = client
        .post(format!("{}/0.htm", &url))
        .headers(headers)
        .form(&[
            ("DDDDD", username),
            ("upass", &pwd),
            ("R1", "0"),
            ("R2", "1"),
        ])
        .send()
        .await?;
    anyhow::ensure!(res.status().is_success(), "login failed");
    Ok(())
}

async fn logout(host: &str) -> Result<()> {
    debug!("host: {}", host);
    let resp = reqwest::get(&format!("http://{}/F.htm", &host)).await?;
    anyhow::ensure!(resp.status().is_success(), "logout failed");
    Ok(())
}

pub async fn dispatch(args: Unicom, cfg: Option<Config>) -> Result<()> {
    trace!("args: {:?}, cfg: {:?}", args, cfg);
    match (args, cfg) {
        (
            Unicom::Login {
                host,
                username,
                password,
            },
            Some(Config {
                host: cfg_host,
                username: cfg_username,
                password: cfg_password,
            }),
        ) => {
            let (host, username, password) = (
                host.unwrap_or_else(|| cfg_host.unwrap()),
                username.unwrap_or_else(|| cfg_username.unwrap()),
                password.unwrap_or_else(|| cfg_password.unwrap()),
            );
            login(&host, &username, &password).await?
        }
        (
            Unicom::Login {
                host: Some(host),
                username: Some(username),
                password: Some(password),
            },
            None,
        ) => login(&host, &username, &password).await?,
        (Unicom::Login { .. }, _) => {
            anyhow::bail!("missing required arguments for login")
        }
        (Unicom::Logout { host }, Some(Config { host: cfg_host, .. })) => {
            let host = host.unwrap_or_else(|| cfg_host.unwrap());
            logout(&host).await?
        }
        _ => anyhow::bail!("invalid arguments"),
    }
    Ok(())
}
