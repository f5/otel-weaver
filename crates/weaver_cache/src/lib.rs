// SPDX-License-Identifier: Apache-2.0

//! A cache system for OTel Weaver.
//!
//! Semantic conventions, schemas and other assets are cached
//! locally to avoid fetching them from the network every time.

use std::default::Default;
use std::fs::create_dir_all;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use crate::Error::GitError;
use gix::clone::PrepareFetch;
use gix::create::Kind;
use gix::remote::fetch::Shallow;
use gix::{create, open, progress};
use tempdir::TempDir;

/// An error that can occur while creating or using a cache.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Home directory not found.
    #[error("Home directory not found")]
    HomeDirNotFound,

    /// Cache directory not created.
    #[error("Cache directory not created: {message}")]
    CacheDirNotCreated {
        /// The error message
        message: String,
    },

    /// Git repo not created.
    #[error("Git repo `{repo_url}` not created: {message}")]
    GitRepoNotCreated {
        /// The git repo URL
        repo_url: String,
        /// The error message
        message: String,
    },

    /// A git error occurred.
    #[error("Git error occurred while cloning `{repo_url}`: {message}")]
    GitError {
        /// The git repo URL
        repo_url: String,
        /// The error message
        message: String,
    },
}

/// A cache system for OTel Weaver.
#[derive(Default)]
pub struct Cache {
    path: PathBuf,
    git_repo_dirs: std::collections::HashMap<String, GitRepo>,
}

/// A git repo that is cloned into a tempdir.
struct GitRepo {
    temp_dir: TempDir,
    path: PathBuf,
}

impl Cache {
    /// Creates the `.otel-weaver/cache` directory in the home directory.
    /// This directory is used to store the semantic conventions, schemas
    /// and other assets that are fetched from the network.
    pub fn try_new() -> Result<Self, Error> {
        let home = dirs::home_dir().ok_or(Error::HomeDirNotFound)?;
        let cache_path = home.join(".otel-weaver/cache");

        create_dir_all(cache_path.as_path()).map_err(|e| Error::CacheDirNotCreated {
            message: e.to_string(),
        })?;

        Ok(Self {
            path: cache_path,
            ..Default::default()
        })
    }

    /// The given repo_url is cloned into the cache and the path to the repo is returned.
    pub fn git_repo(&mut self, repo_url: &str, path: &str) -> Result<PathBuf, Error> {
        // Checks if a tempdir already exists for this repo
        if let Some(git_repo_dir) = self.git_repo_dirs.get(repo_url) {
            return Ok(git_repo_dir.path.clone());
        }

        // Otherwise creates a tempdir for the repo and keeps track of it
        // in the git_repo_dirs hashmap.
        let git_repo_dir = TempDir::new_in(self.path.as_path(), "git-repo").map_err(|e| {
            Error::GitRepoNotCreated {
                repo_url: repo_url.to_string(),
                message: e.to_string(),
            }
        })?;
        let git_repo_pathbuf = git_repo_dir.path().to_path_buf();
        let git_repo_path = git_repo_pathbuf.as_path();

        // Clones the repo into the tempdir.
        // Use shallow clone to save time and space.
        let mut fetch = PrepareFetch::new(
            repo_url,
            git_repo_path,
            Kind::WithWorktree,
            create::Options {
                destination_must_be_empty: true,
                fs_capabilities: None,
            },
            open::Options::isolated(),
        )
        .map_err(|e| GitError {
            repo_url: repo_url.to_string(),
            message: e.to_string(),
        })?
        .with_shallow(Shallow::DepthAtRemote(NonZeroU32::new(1).unwrap()));

        let (mut prepare, _outcome) = fetch
            .fetch_then_checkout(progress::Discard, &AtomicBool::new(false))
            .map_err(|e| GitError {
                repo_url: repo_url.to_string(),
                message: e.to_string(),
            })?;

        let (_repo, _outcome) = prepare
            .main_worktree(progress::Discard, &AtomicBool::new(false))
            .map_err(|e| GitError {
                repo_url: repo_url.to_string(),
                message: e.to_string(),
            })?;

        // Checks the existence of the path in the repo.
        // If the path doesn't exist, returns an error.
        if !git_repo_path.join(path).exists() {
            return Err(Error::GitError {
                repo_url: repo_url.to_string(),
                message: format!("Path `{}` not found in repo", path),
            });
        }

        // Adds the repo to the git_repo_dirs hashmap.
        self.git_repo_dirs.insert(
            repo_url.to_string(),
            GitRepo {
                temp_dir: git_repo_dir,
                path: git_repo_path.join(path),
            },
        );

        Ok(git_repo_pathbuf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Marked as ignore because we don't want to clone the repo every
    /// time we run the tests in CI.
    #[test]
    #[ignore]
    fn test_cache() {
        let mut cache = Cache::try_new().unwrap();
        let result = cache.git_repo(
            "https://github.com/open-telemetry/semantic-conventions.git",
            "model",
        );
        assert!(result.is_ok());
        assert!(result.unwrap().exists());
    }
}
