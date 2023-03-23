use serde::Serialize;

#[derive(Serialize)]
pub struct Context {
    purl: String,
    url: String,
    hash: String,
}

impl Context {
    pub fn new(purl: String, url: String, hash: String) -> Context {
        Context { purl, url, hash }
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_context_serialization() {
        let context = Context {
            purl: "pkg:cargo/crate@0.1.0".into(),
            url: "http://crates.io/not/a/real/crate.crate".into(),
            hash: "8675309".into(),
        };

        let json = serde_json::to_string(&context).unwrap();

        println!("{json}");
    }
}
