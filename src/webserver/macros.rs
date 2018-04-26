#[macro_export]
macro_rules! get_client {
    () => {{
        use hyper::{client::Client as HyperClient, net::HttpsConnector};
        use hyper_native_tls::NativeTlsClient;

        let tc = NativeTlsClient::new()?;
        let connector = HttpsConnector::new(tc);
        HyperClient::with_connector(connector)
    }};
}

#[macro_export]
macro_rules! api_path {
    ($($path:expr),*) => {
        concat!(discord_api_url!(), $($path,)+)
    };

    ($($path:expr),*; $params:expr) => {{

        use serde_urlencoded::to_string;

        format!("{}{}", concat!(discord_api_url!(), $($path,)+ "?"), to_string($params)?)
    }};
}

#[macro_export]
macro_rules! make_auth {
    ($token:expr) => {{
        use hyper::header::Authorization;

        Authorization(format!("Bearer {}", $token))
    }};
}
