use std::path::PathBuf;

use git2::Repository;

pub enum GitError {
    UnsavedFiles,
    UncommittedFiles,
}

pub fn check_unsaved_files(path: &PathBuf) -> Result<(), GitError> {
    let Ok(repo) = Repository::open(path) else {
        return Ok(());
    };

    let mut status_opts = git2::StatusOptions::new();

    status_opts.include_untracked(true);

    let statuses = repo
        .statuses(Some(&mut status_opts))
        .expect("Could not get statuses");

    let mut unsaved_files = false;
    let mut uncommitted_files = false;

    for status in statuses.iter() {
        let status = status.status();

        if status.is_wt_new() || status.is_wt_modified() || status.is_wt_deleted() {
            unsaved_files = true;
        }

        if status.is_index_new() || status.is_index_modified() || status.is_index_deleted() {
            uncommitted_files = true;
        }
    }

    if unsaved_files {
        return Err(GitError::UnsavedFiles);
    }

    if uncommitted_files {
        return Err(GitError::UncommittedFiles);
    }

    Ok(())
}
