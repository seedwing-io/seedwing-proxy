/*
 Crates GIT module to handle cloning of an existing registry and then serving to the cargo client.

 We will fetch a remote repository, rebuilding the git repository with one modification to the index config.json file to use our "dl" and "api" links
*/

use std::{
    fmt::{Display, Formatter},
    fs::{self, File},
    io::{self, ErrorKind, Write},
    path::{Path, PathBuf},
    process::Stdio,
};

use actix_web::{
    body::BodyStream, dev::HttpServiceFactory, http::StatusCode, web, HttpMessage, HttpRequest,
    HttpResponse,
};
use bytes::Bytes;
use fs2::FileExt;
use git2::{
    build::CheckoutBuilder, Direction, FetchOptions, MergeOptions, Oid, Remote, RemoteCallbacks,
    Repository, Signature,
};
use substring::Substring;
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt as _},
    time,
    {
        process::{ChildStdout, Command},
        sync::mpsc,
    },
};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use url::Url;

use super::CratesState;

const CACHEDIR_TAG_FILE: &str = "CACHEDIR.TAG";
const CACHEDIR_TAG_CONTENTS: &str = "Signature: 8a477f597d28d172789f06886806bc55
# This is a seedwing_proxy cache directory for a remote cargo registry.";

const REMOTE_NAME: &str = "repository";
const GIT_DIR: &str = "repository";

const SEEDWING_BRANCH_FILE: &str = ".seedwing/branch";

const GITIGNORE_FILE: &str = ".gitignore";

const CONFIG_JSON_FILE: &str = "config.json";

const TAG_NAME: &str = "seedwing";

const GIT_HTTP_BACKEND: &str = "http-backend";

#[derive(Error, Debug)]
pub enum GitError {
    Io {
        #[from]
        source: io::Error,
    },
    Git2 {
        #[from]
        source: git2::Error,
    },
    Other {
        message: String,
    },
}

impl Display for GitError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        f.write_str(format!("{:?}", *self).as_str())
    }
}

#[derive(Clone)]
pub struct IndexRepository {
    repo: Url,
    local_repository_cache: PathBuf,
    dl: Url,
    api: Url,
    periodic_update: u64,
}

impl IndexRepository {
    pub fn new(
        repo: Url,
        local_repository_cache: PathBuf,
        dl: Url,
        api: Url,
        periodic_update: u64,
    ) -> IndexRepository {
        IndexRepository {
            repo,
            local_repository_cache,
            dl,
            api,
            periodic_update,
        }
    }

    pub fn get_repo(&self) -> &Url {
        &self.repo
    }

    pub fn get_local_repository_cache(&self) -> &PathBuf {
        &self.local_repository_cache
    }

    pub fn get_dl_url(&self) -> &Url {
        &self.dl
    }

    pub fn get_api_url(&self) -> &Url {
        &self.api
    }

    pub fn get_periodic_update(&self) -> u64 {
        self.periodic_update
    }

    fn write_config(&self, config_json_file: &Path) -> io::Result<()> {
        fs::write(
            config_json_file,
            format!(
                "{{\n  \"dl\": \"{}\",\n  \"api\": \"{}\"\n}}\n",
                self.get_dl_url(),
                self.get_api_url()
            ),
        )
    }

    fn get_seedwing_branch(seedwing_branch_file: &PathBuf) -> Result<String, GitError> {
        if let Ok(remote_branch_vec) = fs::read(seedwing_branch_file) {
            if let Ok(remote_branch) = String::from_utf8(remote_branch_vec) {
                Ok(remote_branch)
            } else {
                Err(GitError::Other {
                    message: String::from("Remote branch name is not valid UTF-8"),
                })
            }
        } else {
            Err(GitError::Other {
                message: String::from(
                    "Failed to read remote branch name from seedwing configuration",
                ),
            })
        }
    }

    fn get_branch_short_name(remote_branch: &str) -> &str {
        remote_branch.trim_start_matches("refs/heads/")
    }

    fn get_remote_branch_spec(remote_branch: &str) -> String {
        format!(
            "refs/remotes/{}/{}",
            REMOTE_NAME,
            Self::get_branch_short_name(remote_branch)
        )
    }

    fn fetch_repo(remote: &mut Remote, remote_branch: &String) -> Result<(), GitError> {
        log::info!("Fetching remote branch: {}", remote_branch);

        let mut cb = RemoteCallbacks::new();
        cb.sideband_progress(|data| {
            print!(
                "[sideband]: {}",
                String::from_utf8(Vec::from(data)).unwrap()
            );
            io::stdout().flush().unwrap();
            true
        });

        cb.update_tips(|refname, a, b| {
            if a.is_zero() {
                println!("[update new]     {b} {refname}");
            } else {
                println!("[update updated] {a}..{b} {refname}");
            }
            true
        });
        cb.transfer_progress(|stats| {
            if stats.received_objects() == stats.total_objects() {
                print!(
                    "[Transfer]: Resolving deltas {}/{}\r",
                    stats.indexed_deltas(),
                    stats.total_deltas()
                );
            } else if stats.total_objects() > 0 {
                print!(
                    "[Transfer]: Received {}/{} objects ({}) in {} bytes\r",
                    stats.received_objects(),
                    stats.total_objects(),
                    stats.indexed_objects(),
                    stats.received_bytes()
                );
            }
            io::stdout().flush().unwrap();
            true
        });
        let mut fo = FetchOptions::new();
        fo.remote_callbacks(cb);

        remote.fetch(&[&remote_branch], Some(&mut fo), None)?;
        Ok(())
    }

    pub fn init_local_branch(
        &self,
        repo: &Repository,
        remote_branch: &String,
        git_repository_dir: &Path,
    ) -> Result<(), GitError> {
        let seedwing_branch_file = git_repository_dir.join(SEEDWING_BRANCH_FILE);
        let gitignore_file = git_repository_dir.join(GITIGNORE_FILE);
        let config_json_file = git_repository_dir.join(CONFIG_JSON_FILE);

        let sig = Signature::now("Seedwing", "seedwing@example.com")?;

        let remote_branch_spec = Self::get_remote_branch_spec(remote_branch);
        let branch_name = Self::get_branch_short_name(remote_branch);
        if let Some(upstream_commit) = repo.revparse_single(&remote_branch_spec)?.as_commit() {
            repo.branch(branch_name, upstream_commit, true)?;
            log::info!("Tagging {}", &upstream_commit.id());
            repo.tag(
                TAG_NAME,
                upstream_commit.as_object(),
                &sig,
                "Latest merge",
                true,
            )?;
        } else {
            return Err(GitError::Other {
                message: format!("Could not locate commit for {}", &remote_branch_spec),
            });
        }
        log::info!("Checking out head");
        repo.checkout_head(None)?;

        log::info!("Initialising seedwing configuration");

        fs::write(gitignore_file, format!("/{SEEDWING_BRANCH_FILE}"))?;
        fs::create_dir_all(seedwing_branch_file.parent().unwrap())?;
        fs::write(&seedwing_branch_file, remote_branch)?;

        self.write_config(&config_json_file)?;

        let mut index = repo.index()?;
        index.add_path(Path::new(CONFIG_JSON_FILE))?;
        index.add_path(Path::new(GITIGNORE_FILE))?;
        index.write()?;
        let id = index.write_tree()?;

        let tree = repo.find_tree(id)?;

        let head_id = repo.head()?.target().unwrap();
        let head_commit = repo.find_commit(head_id)?;

        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Committing Initial config.json",
            &tree,
            &[&head_commit],
        )?;
        repo.checkout_head(None)?;
        Ok(())
    }

    pub fn update_local_cache(local_repository_cache: &PathBuf) -> Result<(), GitError> {
        let cache_dir = Path::new(local_repository_cache);
        let cache_dir_tag = cache_dir.join(CACHEDIR_TAG_FILE);
        let git_repository_dir = cache_dir.join(GIT_DIR);
        let seedwing_branch_file = git_repository_dir.join(SEEDWING_BRANCH_FILE);

        let cache_dir_tag_file = File::open(cache_dir_tag)?;
        if cache_dir_tag_file.try_lock_exclusive().is_err() {
            log::info!("Lock file currently held, waiting for lock");
            cache_dir_tag_file.lock_exclusive()?;
        }
        log::info!("Updating cache");

        let repo = Repository::open(&git_repository_dir)?;
        let mut remote = repo.find_remote(REMOTE_NAME)?;

        let remote_branch = Self::get_seedwing_branch(&seedwing_branch_file)?;
        Self::fetch_repo(&mut remote, &remote_branch)?;

        let tag_spec = format!("refs/tags/{TAG_NAME}");
        let tagged_id = match repo.revparse_single(&tag_spec) {
            Ok(obj) => obj.as_tag().unwrap().target()?.id(),
            Err(_) => Oid::zero(),
        };

        let remote_branch_spec = Self::get_remote_branch_spec(&remote_branch);
        let remote_id = repo
            .revparse_single(&remote_branch_spec)?
            .as_commit()
            .unwrap()
            .id();

        if tagged_id != remote_id {
            let sig = Signature::now("Seedwing", "seedwing@example.com")?;

            let head_id = repo.head()?.target().unwrap();
            let head_commit = repo.find_commit(head_id)?;

            let remote_commit = repo.find_commit(remote_id)?;

            log::info!("Merging {}", remote_id);
            let mut merge_opts = MergeOptions::new();
            merge_opts.target_limit(1);

            let mut index = repo.merge_commits(&head_commit, &remote_commit, Some(&merge_opts))?;

            if index.has_conflicts() {
                index.remove_all([CONFIG_JSON_FILE].iter(), None)?;
                if index.has_conflicts() {
                    return Err(GitError::Other {
                        message: String::from("Could not resolve all conflicts"),
                    });
                }
            }
            let id = index.write_tree_to(&repo)?;
            let tree = repo.find_tree(id)?;
            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                "Merge commit for remote repository",
                &tree,
                &[&head_commit, &remote_commit],
            )?;

            log::info!("Tagging {}", &remote_id);
            repo.tag(
                TAG_NAME,
                remote_commit.as_object(),
                &sig,
                "Latest merge",
                true,
            )?;

            let mut cb = CheckoutBuilder::new();
            let cb = cb.force();
            repo.checkout_head(Some(cb))?;
        } else {
            log::info!("No updated required");
        }
        cache_dir_tag_file.unlock()?;
        Ok(())
    }

    async fn periodic_update_of_cache(local_repository_cache: String, periodic_update: u64) {
        let path = PathBuf::from(local_repository_cache);
        let mut interval = time::interval(time::Duration::from_secs(periodic_update));
        loop {
            interval.tick().await;
            if let Err(error) = Self::update_local_cache(&path) {
                log::info!("Error updating cache: {error}");
            }
        }
    }

    pub fn prepare_local_cache(&self) -> Result<(), GitError> {
        let cache_dir = Path::new(&self.local_repository_cache);
        let cache_dir_tag = cache_dir.join(CACHEDIR_TAG_FILE);
        let git_repository_dir = cache_dir.join(GIT_DIR);
        let seedwing_branch_file = git_repository_dir.join(SEEDWING_BRANCH_FILE);

        fs::create_dir_all(cache_dir)?;

        {
            let mut cache_dir_tag_file;
            if !cache_dir_tag.exists() {
                cache_dir_tag_file = File::create(&cache_dir_tag)?;
                writeln!(&mut cache_dir_tag_file, "{CACHEDIR_TAG_CONTENTS}")?;
            } else {
                cache_dir_tag_file = File::open(&cache_dir_tag)?;
            }

            cache_dir_tag_file.try_lock_exclusive()?;

            let mut is_repository_valid = false;

            // Check the directory contains the git repository and the seedwing branch file
            if git_repository_dir.exists() && seedwing_branch_file.exists() {
                // Check repository has the same URL as the registry remote
                if let Ok(repo) = Repository::open(&git_repository_dir) {
                    if let Ok(remote) = repo.find_remote(REMOTE_NAME) {
                        if let Some(url) = remote.url() {
                            is_repository_valid = url == self.get_repo().as_str();
                        }
                    }
                }
            }

            if !is_repository_valid {
                log::info!("Creating cache");
                if git_repository_dir.exists() {
                    fs::remove_dir_all(&git_repository_dir)?;
                }

                let repo = Repository::init(&git_repository_dir)?;

                let mut remote = repo.remote(REMOTE_NAME, self.get_repo().as_str())?;
                remote.connect(Direction::Fetch)?;

                let remote_branch_buf = remote.default_branch()?;
                if let Some(remote_branch) = remote_branch_buf.as_str() {
                    let remote_branch = String::from(remote_branch);

                    Self::fetch_repo(&mut remote, &remote_branch)?;

                    self.init_local_branch(&repo, &remote_branch, &git_repository_dir)?;
                } else {
                    return Err(GitError::Other {
                        message: String::from("Remote branch name is not valid UTF-8"),
                    });
                }
            }
        }

        let path = String::from(self.local_repository_cache.to_str().unwrap());
        if self.get_periodic_update() > 0 {
            tokio::spawn(Self::periodic_update_of_cache(
                path,
                self.get_periodic_update(),
            ));
        }
        Ok(())
    }
}

async fn read_line(child: &mut ChildStdout) -> Result<String, io::Error> {
    let mut vec: Vec<u8> = Vec::new();

    // Need a better way of handling this
    loop {
        match child.read_u8().await {
            Ok(byte) => {
                if byte == 13 { // '\r'
                     // ignore
                } else if byte == 10 {
                    // '\n'
                    break;
                } else {
                    vec.push(byte);
                }
            }
            Err(error) => return Err(error),
        }
    }
    match String::from_utf8(vec) {
        Ok(result) => Ok(result),
        Err(error) => Err(io::Error::new(ErrorKind::Other, error)),
    }
}

async fn handle_backend_service(
    req: HttpRequest,
    mut payload: web::Payload,
    crates: web::Data<CratesState>,
) -> Result<HttpResponse, actix_web::Error> {
    let git_dir = crates
        .index_repository
        .get_local_repository_cache()
        .join(GIT_DIR);
    let git_dir = git_dir.to_str().unwrap();

    let req_path = req.uri().path();

    let scope = crates.scope.as_str();
    let new_path = format!("{git_dir}{}", req_path.strip_prefix(scope).unwrap());

    let mut cmd = Command::new(&crates.git_cmd)
        .arg(GIT_HTTP_BACKEND)
        .env("GIT_HTTP_EXPORT_ALL", "")
        .env("REQUEST_METHOD", req.method().to_string())
        .env("QUERY_STRING", req.query_string())
        .env("PATH_TRANSLATED", new_path)
        .env("CONTENT_TYPE", req.content_type())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = cmd.stdin.take().unwrap();
    let mut stdout = cmd.stdout.take().unwrap();

    actix_web::rt::spawn(async move {
        while let Some(chunk) = payload.next().await {
            let content = chunk.unwrap().to_vec();
            if let Err(error) = stdin.write_all(&content).await {
                log::info!("Unexpected error writing to child process input: {}", error);
                break;
            }
        }
    });

    let mut resp = HttpResponse::build(StatusCode::OK);

    loop {
        let read_line = read_line(&mut stdout).await;
        match read_line {
            Ok(line) => {
                if line.is_empty() {
                    break;
                } else if let Some(index) = line.find(':') {
                    let header_name = line.substring(0, index);
                    let header_value = line.substring(index + 1, line.len());
                    resp.insert_header((header_name, header_value));
                }
            }
            Err(_) => break,
        }
    }

    let (tx, rx) = mpsc::channel::<Result<Bytes, io::Error>>(100);

    actix_web::rt::spawn(async move {
        let mut buf: [u8; 8192] = [0; 8192];
        while let Ok(count) = stdout.read(&mut buf).await {
            if count > 0 {
                if (tx.send(Ok(Bytes::from(buf[0..count].to_vec()))).await).is_err() {
                    break;
                }
            } else {
                break;
            }
        }
    });

    let body_stream = BodyStream::new(ReceiverStream::new(rx));
    Ok(resp.body(body_stream))
}

pub fn git_backend_service(scope: &str) -> impl HttpServiceFactory {
    web::resource(scope).to(handle_backend_service)
}
