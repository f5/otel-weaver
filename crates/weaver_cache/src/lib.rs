// SPDX-License-Identifier: Apache-2.0

//! A cache system for OTel Weaver.
//!
//! Semantic conventions, schemas and other assets are cached
//! locally to avoid fetching them from the network every time.

use std::fs::create_dir_all;
use std::path::PathBuf;
use std::default::Default;
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
}

/// A cache system for OTel Weaver.
#[derive(Default)]
pub struct Cache {
    path: PathBuf,
    git_repo_dirs: std::collections::HashMap<String, TempDir>,
}

impl Cache {
    /// Creates the `.otel-weaver/cache` directory in the home directory.
    /// This directory is used to store the semantic conventions, schemas
    /// and other assets that are fetched from the network.
    pub fn try_new() -> Result<Self, Error> {
        let home = dirs::home_dir().ok_or(Error::HomeDirNotFound)?;
        let cache_path = home.join(".otel-weaver/cache");

        create_dir_all(cache_path.as_path())
            .map_err(|e| Error::CacheDirNotCreated { message: e.to_string() })?;

        Ok(Self {
            path: cache_path,
            ..Default::default()
        })
    }

    /// The given repo_url is cloned into the cache and the path to the repo is returned.
    pub fn git_repo(&mut self, repo_url: &str) -> Result<PathBuf, Error> {
        if let Some(git_repo_dir) = self.git_repo_dirs.get(repo_url) {
            return Ok(git_repo_dir.path().to_path_buf());
        }

        let git_repo_dir = TempDir::new_in(self.path.as_path(), "git-repo")
            .map_err(|e| Error::GitRepoNotCreated { repo_url: repo_url.to_string(), message: e.to_string() })?;
        let git_repo_pathbuf = git_repo_dir.path().to_path_buf();
        self.git_repo_dirs.insert(repo_url.to_string(), git_repo_dir);

        // ToDo use gitoxide to clone the repo

        Ok(git_repo_pathbuf)
    }
}