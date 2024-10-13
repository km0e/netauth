use crate::unicom;
use xcfg::XCfg;

#[derive(serde::Deserialize, serde::Serialize, xcfg::XCfg, Debug)]
pub struct Config {
    pub unicom: Option<unicom::Config>,
}
