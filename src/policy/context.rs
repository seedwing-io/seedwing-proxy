use serde::Serialize;
use url::Url;

#[derive(Serialize)]
pub struct Context {
    original_url: Url,
    hash: String,
    id: ArtifactIdentifier,
    license: Option<String>,
    repository_id: String,
}

#[derive(Serialize)]
#[serde(tag = "_type")]
pub enum ArtifactIdentifier {
    #[serde(rename = "crate")]
    Crate { name: String },
    #[serde(rename = "m2")]
    M2 {
        group_id: String,
        artifact_id: String,
    },
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_context_serialization() {
        let context = Context {
            original_url: Url::parse("http://crates.io/not/a/real/crate.crate").unwrap(),
            hash: "8675309".into(),
            id: ArtifactIdentifier::Crate {
                name: "rust_crate".into(),
            },
            license: None,
            repository_id: "crates-io".to_string(),
        };

        let json = serde_json::to_string(&context).unwrap();

        println!("{}", json);
    }
}
