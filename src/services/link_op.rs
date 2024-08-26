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

pub fn trim_link(link: &str) -> String {
    clean_url(
        match link
            .strip_prefix("http://")
            .or(link.strip_prefix("https://"))
        {
            Some(res) => res,
            None => link,
        },
    )
}

fn clean_url(input: &str) -> String {
    // Parse the input string as a URL
    let parsed_url = Url::parse(&format!("https://{}", input)).expect("Invalid URL");

    // Get the host (domain) part of the URL
    let host = parsed_url.host_str().unwrap_or("");

    // Remove the "www" or "www3" if present
    let cleaned_host = if host.starts_with("www") {
        let parts: Vec<&str> = host.split('.').collect();
        if parts.len() > 2 {
            parts[1..].join(".")
        } else {
            parts.join(".")
        }
    } else {
        host.to_string()
    };

    // Combine the cleaned host with the path
    let mut cleaned_url = cleaned_host;
    if let Some(path) = parsed_url.path_segments() {
        let parts = path.collect::<Vec<_>>();
        if parts.len() > 0 && parts[0] != "" {
            cleaned_url.push_str("/");
            cleaned_url.push_str(&parts.join("/"));
        }
    }
    cleaned_url.trim_end_matches('/').to_string()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i_can_clean_urls() {
        let urls = [
            ["www3.nhk.or.jp/news/easy", "nhk.or.jp/news/easy"],
            [
                "www.newscientist.com/subject/space/feed/",
                "newscientist.com/subject/space/feed",
            ],
            ["ground.news", "ground.news"],
        ];

        for url in urls {
            assert_eq!(clean_url(url[0]), url[1]);
        }
    }
}
