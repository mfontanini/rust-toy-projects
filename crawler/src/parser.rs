use url::Url;

use regex::Regex;

pub struct LinkParser {
    base_url: Url,
}

impl LinkParser {
    pub fn new(url: Url) -> LinkParser {
        LinkParser {
            base_url: url
        }
    }

    pub fn parse(self, data: &str) -> Vec<Url> {
        lazy_static! {
            static ref PATTERNS : String = vec![
                r#"href="([^"]+)""#,
                r#"src="([^"]+)""#
            ].join("|");
            static ref RE : Regex = Regex::new(
                &format!(r#"(?:{})"#, &PATTERNS[..])
            ).unwrap();
        }
        let mut output = Vec::new();
        for link in RE.captures_iter(data) {
            // Skip the whole match, keep valid entries and unwrap option
            for parsed_link in link.iter().skip(1).filter(|l| !l.is_none()).map(Option::unwrap) {
                let parsed_link = parsed_link.as_str();
                if let Ok(url) = Url::parse(parsed_link) {
                    output.push(url);
                }
                else if let Ok(url) = self.base_url.join(parsed_link) {
                    output.push(url);
                }
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::LinkParser;
    use url::Url;

    #[test]
    fn parse_urls() {
        let absolute_url = Url::parse("http://www.example.com/foo.html").unwrap();
        let image_url = Url::parse("http://www.images.com/image.jpg").unwrap();
        let base_url = Url::parse("http://something.com/test").unwrap();
        let relative_url = "../bar.html";
        let text = format!(
            r#"
                <a href="{}">foo</a>
                <a href="{}">bar</a>
                <a href="">zar</a>
                <img src="{}" />
            "#,
            absolute_url,
            relative_url,
            image_url
        );
        let parser = LinkParser::new(base_url);
        let urls = vec![
            absolute_url,
            Url::parse("http://something.com/bar.html").unwrap(),
            image_url
        ];
        assert_eq!(urls, parser.parse(&text));
    }

    #[test]
    fn invalid_data() {
        let base_url = Url::parse("http://www.example.com").unwrap();
        let text = "asd";
        let parser = LinkParser::new(base_url);
        assert_eq!(0, parser.parse(&text).len());
    }
}
