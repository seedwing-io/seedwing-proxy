use serde::Serialize;

#[derive(Serialize)]
pub struct Context {
    purl: String,
    url: String,
    hash: String,
    id: ArtifactIdentifier,
    license: Option<String>,
    repository_id: String,
}

impl Context {
    pub fn new(
        purl: String,
        url: String,
        hash: String,
        id: ArtifactIdentifier,
        repository_id: String,
    ) -> Context {
        let license = None; // TODO: something
        Context {
            purl,
            url,
            hash,
            id,
            license,
            repository_id,
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }
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
            purl: "pkg:cargo/crate@0.1.0".into(),
            url: "http://crates.io/not/a/real/crate.crate".into(),
            hash: "8675309".into(),
            id: ArtifactIdentifier::Crate {
                name: "rust_crate".into(),
            },
            license: None,
            repository_id: "crates-io".to_string(),
        };

        let json = serde_json::to_string(&context).unwrap();

        println!("{json}");
    }
}
