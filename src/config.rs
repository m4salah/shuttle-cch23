#[derive(clap::Parser, Clone, Debug)]
pub struct Config {
    #[clap(long, env)]
    pub database_url: String,

    #[clap(long, env)]
    pub geocoding_api_key: String,

    #[clap(long, env, default_value = "8000")]
    pub port: u16,
}
