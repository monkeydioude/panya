use url::Url;

pub trait HostStr {
    fn get_host_string(&self) -> String;
}

impl HostStr for String {
    fn get_host_string(&self) -> String {
        match Url::parse(self) {
            Ok(res) => res.host_str().unwrap_or_default().to_string(),
            Err(_) => self.clone(),
        }
    }
}

