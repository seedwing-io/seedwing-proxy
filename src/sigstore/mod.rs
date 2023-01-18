use sigstore::rekor::apis::{
    configuration::Configuration,
    entries_api::get_log_entry_by_uuid,
    index_api::{search_index, SearchIndexError},
    Error,
};
use sigstore::rekor::models::SearchIndex;

pub async fn search(digest: String) -> Result<Vec<String>, Error<SearchIndexError>> {
    let query = SearchIndex {
        email: None,
        public_key: None,
        hash: Some(digest),
    };
    let configuration = Configuration::default();
    match search_index(&configuration, query).await {
        Ok(uuids) => {
            if log::log_enabled!(log::Level::Debug) {
                for uuid in uuids.iter() {
                    let entry = get_log_entry_by_uuid(&configuration, uuid).await;
                    if let Ok(entry) = entry {
                        log::debug!("{:?}", entry);
                    }
                }
            }
            Ok(uuids)
        }
        Err(e) => Err(e),
    }
}
