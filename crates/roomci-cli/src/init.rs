use std::{
    fs,
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use same_file::is_same_file;
use thiserror::Error;

const SMOKE_YAML: &str = include_str!("../templates/smoke.yaml");
const SCAFFOLD_README: &str = include_str!("../templates/README.md");
const VSCODE_SETTINGS: &str = include_str!("../templates/settings.json");
const GITHUB_WORKFLOW: &str = include_str!("../templates/github-actions-roomci.yml");

#[derive(Debug, Error)]
pub enum InitError {
    #[error("refusing to overwrite existing generated file(s): {paths}. Re-run with --force to replace every generated file.")]
    ExistingTargets { paths: String },
    #[error("refusing to write through symbolic link: {path}")]
    SymbolicLink { path: String },
    #[error("unsafe generated path: {path}")]
    UnsafePath { path: String },
    #[error("failed to {operation} {path}: {source}")]
    Io {
        operation: &'static str,
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to install generated files: {primary}; recovery also failed: {recovery}; staging backup preserved at {backup_path}")]
    Recovery {
        primary: String,
        recovery: String,
        backup_path: String,
    },
}

struct TemplateFile {
    relative_path: &'static str,
    contents: &'static str,
}

/// Generate a small, versioned starting point without reading or merging user files.
///
/// The preflight is deliberately all-or-nothing: an existing target means no target is
/// changed unless `force` is explicit. Files are prepared in a private staging directory,
/// then installed with rollback, so a write or rename failure cannot leave a partially
/// generated file set behind.
pub fn scaffold(path: &Path, ci_github: bool, force: bool) -> Result<Vec<PathBuf>, InitError> {
    validate_output_root(path)?;
    let files = template_files(ci_github);
    let targets = files
        .iter()
        .map(|file| checked_target(path, file.relative_path))
        .collect::<Result<Vec<_>, _>>()?;

    for target in &targets {
        ensure_safe_parents(path, target)?;
        reject_symlink(target)?;
    }

    let existing = targets
        .iter()
        .filter_map(|target| existing_file(target).transpose())
        .collect::<Result<Vec<_>, _>>()?;
    if !force && !existing.is_empty() {
        return Err(InitError::ExistingTargets {
            paths: existing
                .iter()
                .map(|target| target.display().to_string())
                .collect::<Vec<_>>()
                .join(", "),
        });
    }

    fs::create_dir_all(path).map_err(|source| io_error("create output directory", path, source))?;
    let staging = create_staging_dir(path)?;
    match install_files(&staging, path, &files, &targets, force) {
        Ok(()) => {
            fs::remove_dir_all(&staging)
                .map_err(|source| io_error("remove staging directory", &staging, source))?;
            Ok(targets)
        }
        Err(failure) if failure.preserve_staging => Err(failure.error),
        Err(failure) => match fs::remove_dir_all(&staging) {
            Ok(()) => Err(failure.error),
            Err(source) => Err(InitError::Recovery {
                primary: failure.error.to_string(),
                recovery: format!("remove staging directory {}: {source}", staging.display()),
                backup_path: staging.display().to_string(),
            }),
        },
    }
}

pub fn next_steps(path: &Path) -> String {
    let scenario = if path == Path::new(".") {
        PathBuf::from("roomci/smoke.yaml")
    } else {
        path.join("roomci/smoke.yaml")
    };
    format!(
        "next steps:\n  roomci validate {}\n  roomci run {} --verbose",
        shell_quote(&scenario),
        shell_quote(&scenario)
    )
}

/// Quote only when necessary so the default output remains easy to read while
/// paths with spaces or quotes remain safe to paste into a POSIX shell.
fn shell_quote(path: &Path) -> String {
    let value = path.to_string_lossy();
    if !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'_' | b'@' | b'%' | b'+' | b'=' | b':' | b',' | b'.' | b'/' | b'-'
                )
        })
    {
        return value.into_owned();
    }

    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn template_files(ci_github: bool) -> Vec<TemplateFile> {
    let mut files = vec![
        TemplateFile {
            relative_path: "roomci/smoke.yaml",
            contents: SMOKE_YAML,
        },
        TemplateFile {
            relative_path: "roomci/README.md",
            contents: SCAFFOLD_README,
        },
        TemplateFile {
            relative_path: ".vscode/settings.json",
            contents: VSCODE_SETTINGS,
        },
    ];
    if ci_github {
        files.push(TemplateFile {
            relative_path: ".github/workflows/roomci.yml",
            contents: GITHUB_WORKFLOW,
        });
    }
    files
}

fn checked_target(root: &Path, relative: &'static str) -> Result<PathBuf, InitError> {
    let relative_path = Path::new(relative);
    if relative_path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(InitError::UnsafePath {
            path: relative.to_string(),
        });
    }
    Ok(root.join(relative_path))
}

fn validate_output_root(path: &Path) -> Result<(), InitError> {
    if path.is_absolute() {
        // System temporary directories on macOS can legitimately live below the
        // `/var -> /private/var` compatibility symlink. Reject an explicit root
        // symlink here; containment checks before mutation protect descendants.
        return reject_symlink(path);
    }

    let mut current = std::env::current_dir()
        .map_err(|source| io_error("read current directory", Path::new("."), source))?;
    reject_symlink(&current)?;
    for component in path.components() {
        match component {
            Component::Normal(segment) => {
                current.push(segment);
                reject_symlink(&current)?;
            }
            Component::ParentDir => {
                current.pop();
            }
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => unreachable!("relative path components"),
        }
    }
    Ok(())
}

fn ensure_safe_parents(root: &Path, target: &Path) -> Result<(), InitError> {
    let mut parent = target.parent();
    while let Some(current) = parent {
        if current == root {
            break;
        }
        match fs::symlink_metadata(current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(InitError::SymbolicLink {
                    path: current.display().to_string(),
                });
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err(InitError::Io {
                    operation: "use non-directory parent",
                    path: current.display().to_string(),
                    source: std::io::Error::from(std::io::ErrorKind::NotADirectory),
                });
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(source) => return Err(io_error("inspect parent directory", current, source)),
        }
        parent = current.parent();
    }
    Ok(())
}

fn reject_symlink(path: &Path) -> Result<(), InitError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(InitError::SymbolicLink {
            path: path.display().to_string(),
        }),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(io_error("inspect path", path, source)),
    }
}

fn existing_file(path: &Path) -> Result<Option<PathBuf>, InitError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() => Ok(Some(path.to_path_buf())),
        Ok(_) => Err(InitError::Io {
            operation: "replace non-file generated target",
            path: path.display().to_string(),
            source: std::io::Error::from(std::io::ErrorKind::InvalidInput),
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(io_error("inspect generated target", path, source)),
    }
}

fn create_staging_dir(root: &Path) -> Result<PathBuf, InitError> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let staging = root.join(format!(
        ".roomci-init-staging-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&staging)
        .map_err(|source| io_error("create staging directory", &staging, source))?;
    Ok(staging)
}

struct InstallFailure {
    error: InitError,
    preserve_staging: bool,
}

fn install_files(
    staging: &Path,
    root: &Path,
    files: &[TemplateFile],
    targets: &[PathBuf],
    force: bool,
) -> Result<(), InstallFailure> {
    install_files_with_hooks(
        staging,
        root,
        files,
        targets,
        force,
        InstallHooks {
            after_revalidation: |_: &Path| {},
            after_backup: |_: &Path| {},
            after_install: |_: &Path| {},
        },
    )
}

struct InstallHooks<F, G, H> {
    after_revalidation: F,
    after_backup: G,
    after_install: H,
}

fn install_files_with_hooks<F, G, H>(
    staging: &Path,
    root: &Path,
    files: &[TemplateFile],
    targets: &[PathBuf],
    force: bool,
    hooks: InstallHooks<F, G, H>,
) -> Result<(), InstallFailure>
where
    F: FnMut(&Path),
    G: FnMut(&Path),
    H: FnMut(&Path),
{
    let InstallHooks {
        mut after_revalidation,
        mut after_backup,
        mut after_install,
    } = hooks;
    for file in files {
        let staged = staging.join(file.relative_path);
        let parent = staged
            .parent()
            .expect("template paths always have a parent");
        fs::create_dir_all(parent).map_err(|source| {
            InstallFailure::recovered(io_error("create staging parent", parent, source))
        })?;
        fs::write(&staged, file.contents).map_err(|source| {
            InstallFailure::recovered(io_error("write staged file", &staged, source))
        })?;
    }

    prepare_output_parents(root, targets).map_err(InstallFailure::recovered)?;

    let backup_root = staging.join(".backup");
    fs::create_dir(&backup_root).map_err(|source| {
        InstallFailure::recovered(io_error("create backup directory", &backup_root, source))
    })?;
    let mut installed = Vec::new();
    for (index, (file, target)) in files.iter().zip(targets).enumerate() {
        let staged = staging.join(file.relative_path);
        if let Err(error) = revalidate_target_for_mutation(root, target, force) {
            return Err(recover_or_preserve(error, &installed, staging));
        }

        // Test-only callers use this deterministic seam to model a file arriving
        // after the preflight. Non-force must never turn that new file into a backup.
        after_revalidation(target);
        let backup = if force {
            existing_file(target)
                .map_err(|error| recover_or_preserve(error, &installed, staging))?
                .map(|_| backup_root.join(index.to_string()))
        } else {
            None
        };
        if let Some(backup) = &backup {
            if let Err(source) = fs::rename(target, backup) {
                return Err(recover_or_preserve(
                    io_error("back up generated file", target, source),
                    &installed,
                    staging,
                ));
            }
        }

        // This seam models the interval after a force backup but before exclusive
        // creation, so race tests can prove the original is never overwritten.
        after_backup(target);

        // hard_link creates the final name exclusively. Unlike rename/copy, it cannot
        // replace a file that appeared after the preflight and it never exposes a
        // partially written template because the staged source is already complete.
        if let Err(source) = fs::hard_link(&staged, target) {
            let primary = io_error("install generated file", target, source);
            let failure = match backup {
                Some(backup) => {
                    // The backup was already moved out of the final path. Include this
                    // current entry in rollback so an unrelated hard-link failure does
                    // not leave the user's original file absent from the repository.
                    let mut rollback_entries = installed;
                    rollback_entries.push(Installed {
                        target: target.clone(),
                        staged,
                        backup: Some(backup),
                    });
                    recover_or_preserve(primary, &rollback_entries, staging)
                }
                None => recover_or_preserve(primary, &installed, staging),
            };
            return Err(failure);
        }
        installed.push(Installed {
            target: target.clone(),
            staged,
            backup,
        });
        after_install(target);
    }
    Ok(())
}

impl InstallFailure {
    fn recovered(error: InitError) -> Self {
        Self {
            error,
            preserve_staging: false,
        }
    }

    fn preserved(primary: InitError, recovery: String, staging: &Path) -> Self {
        Self {
            error: InitError::Recovery {
                primary: primary.to_string(),
                recovery,
                backup_path: staging.display().to_string(),
            },
            preserve_staging: true,
        }
    }
}

struct Installed {
    target: PathBuf,
    staged: PathBuf,
    backup: Option<PathBuf>,
}

fn recover_or_preserve(
    primary: InitError,
    installed: &[Installed],
    staging: &Path,
) -> InstallFailure {
    match rollback(installed) {
        Ok(()) => InstallFailure::recovered(primary),
        Err(recovery) => InstallFailure::preserved(primary, recovery, staging),
    }
}

fn rollback(installed: &[Installed]) -> Result<(), String> {
    rollback_with_hook(installed, |_| {})
}

fn rollback_with_hook<F>(installed: &[Installed], mut before_restore: F) -> Result<(), String>
where
    F: FnMut(&Path),
{
    let mut errors = Vec::new();
    for entry in installed.iter().rev() {
        let removed_generated_file = match fs::symlink_metadata(&entry.target) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
            Err(error) => {
                errors.push(format!("inspect {}: {error}", entry.target.display()));
                false
            }
            Ok(metadata) if metadata.file_type().is_symlink() => {
                errors.push(format!(
                    "refuse to remove symbolic-link replacement {}",
                    entry.target.display()
                ));
                false
            }
            Ok(metadata) if !metadata.is_file() => {
                errors.push(format!(
                    "refuse to remove non-file replacement {}",
                    entry.target.display()
                ));
                false
            }
            Ok(_) => match is_same_file(&entry.target, &entry.staged) {
                Ok(true) => match fs::remove_file(&entry.target) {
                    Ok(()) => true,
                    Err(error) => {
                        errors.push(format!("remove {}: {error}", entry.target.display()));
                        false
                    }
                },
                Ok(false) => {
                    errors.push(format!(
                        "refuse to remove concurrent replacement {}",
                        entry.target.display()
                    ));
                    false
                }
                Err(error) => {
                    errors.push(format!(
                        "compare {} with staged file {}: {error}",
                        entry.target.display(),
                        entry.staged.display()
                    ));
                    false
                }
            },
        };
        if !removed_generated_file {
            continue;
        }
        if let Some(backup) = &entry.backup {
            // Match installation's no-replace guarantee: another process can create
            // the final name after removal, so rename would be an unsafe overwrite.
            before_restore(&entry.target);
            if let Err(error) = fs::hard_link(backup, &entry.target) {
                errors.push(format!(
                    "restore {} from {} without replacement: {error}",
                    entry.target.display(),
                    backup.display()
                ));
                continue;
            }
            if let Err(error) = fs::remove_file(backup) {
                errors.push(format!(
                    "remove restored backup {}: {error}",
                    backup.display()
                ));
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

fn prepare_output_parents(root: &Path, targets: &[PathBuf]) -> Result<(), InitError> {
    validate_output_root(root)?;
    for target in targets {
        let parent = target
            .parent()
            .expect("template paths always have a parent");
        // Inspect every existing parent before creating anything: create_dir_all
        // follows links, which would otherwise permit `.github` to escape root.
        ensure_safe_parents(root, target)?;
        fs::create_dir_all(parent)
            .map_err(|source| io_error("create output parent", parent, source))?;
        validate_output_root(root)?;
        ensure_safe_parents(root, target)?;
        ensure_target_stays_under_root(root, target)?;
    }
    Ok(())
}

fn revalidate_target_for_mutation(
    root: &Path,
    target: &Path,
    force: bool,
) -> Result<(), InitError> {
    // `init` is a local CLI, not a hostile shared-filesystem sandbox. Rechecking
    // immediately before each final-name mutation catches ordinary symlink races;
    // the exclusive hard link below prevents overwrite if a file appears later.
    // A complete adversarial TOCTOU defence would require platform-specific dirfd APIs.
    validate_output_root(root)?;
    ensure_safe_parents(root, target)?;
    ensure_target_stays_under_root(root, target)?;
    reject_symlink(target)?;
    let existing = existing_file(target)?;
    if !force && existing.is_some() {
        return Err(InitError::ExistingTargets {
            paths: target.display().to_string(),
        });
    }
    Ok(())
}

fn ensure_target_stays_under_root(root: &Path, target: &Path) -> Result<(), InitError> {
    let canonical_root = fs::canonicalize(root)
        .map_err(|source| io_error("canonicalize output root", root, source))?;
    let parent = target
        .parent()
        .expect("template paths always have a parent");
    let canonical_parent = fs::canonicalize(parent)
        .map_err(|source| io_error("canonicalize output parent", parent, source))?;
    if canonical_parent.starts_with(&canonical_root) {
        Ok(())
    } else {
        Err(InitError::UnsafePath {
            path: parent.display().to_string(),
        })
    }
}

fn io_error(operation: &'static str, path: &Path, source: std::io::Error) -> InitError {
    InitError::Io {
        operation,
        path: path.display().to_string(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_does_not_overwrite_a_target_created_after_preflight() {
        let tempdir = tempfile::tempdir().unwrap();
        let root = tempdir.path();
        let files = template_files(false);
        let targets = files
            .iter()
            .map(|file| checked_target(root, file.relative_path).unwrap())
            .collect::<Vec<_>>();
        let racing_target = &targets[0];
        fs::create_dir_all(racing_target.parent().unwrap()).unwrap();
        fs::write(racing_target, "created after preflight\n").unwrap();
        let staging = create_staging_dir(root).unwrap();

        assert!(install_files(&staging, root, &files, &targets, false).is_err());
        assert_eq!(
            fs::read_to_string(racing_target).unwrap(),
            "created after preflight\n"
        );
        assert!(!targets[1].exists());
        assert!(!targets[2].exists());
    }

    #[cfg(unix)]
    #[test]
    fn install_rechecks_a_parent_symlink_before_writing() {
        use std::os::unix::fs::symlink;

        let tempdir = tempfile::tempdir().unwrap();
        let root = tempdir.path();
        let outside = root.join("outside");
        fs::create_dir(&outside).unwrap();
        let files = template_files(false);
        let targets = files
            .iter()
            .map(|file| checked_target(root, file.relative_path).unwrap())
            .collect::<Vec<_>>();
        symlink(&outside, root.join(".vscode")).unwrap();
        let staging = create_staging_dir(root).unwrap();

        assert!(install_files(&staging, root, &files, &targets, false).is_err());
        assert!(!outside.join("settings.json").exists());
    }

    #[test]
    fn rollback_reports_restore_failure_and_preserves_the_backup() {
        let tempdir = tempfile::tempdir().unwrap();
        let target = tempdir.path().join("generated-target");
        fs::create_dir(&target).unwrap();
        let backup = tempdir.path().join("staging-backup");
        fs::write(&backup, "original generated file\n").unwrap();

        let error = rollback(&[Installed {
            target,
            staged: backup.clone(),
            backup: Some(backup.clone()),
        }])
        .unwrap_err();

        assert!(error.contains("remove"));
        assert!(backup.exists(), "a failed recovery must retain the backup");
    }

    #[test]
    fn rollback_never_removes_a_target_replaced_after_installation() {
        let tempdir = tempfile::tempdir().unwrap();
        let target = tempdir.path().join("generated-target");
        let staged = tempdir.path().join("staged-generated-target");
        fs::write(&staged, "generated file\n").unwrap();
        fs::write(&target, "concurrent replacement\n").unwrap();

        rollback(&[Installed {
            target: target.clone(),
            staged,
            backup: None,
        }])
        .unwrap_err();

        assert_eq!(
            fs::read_to_string(&target).unwrap(),
            "concurrent replacement\n"
        );
    }

    #[test]
    fn rollback_never_overwrites_a_target_created_before_backup_restore() {
        let tempdir = tempfile::tempdir().unwrap();
        let target = tempdir.path().join("generated-target");
        let staged = tempdir.path().join("staged-generated-target");
        let backup = tempdir.path().join("original-backup");
        fs::write(&staged, "generated file\n").unwrap();
        fs::hard_link(&staged, &target).unwrap();
        fs::write(&backup, "original file\n").unwrap();

        let error = rollback_with_hook(
            &[Installed {
                target: target.clone(),
                staged,
                backup: Some(backup.clone()),
            }],
            |target| fs::write(target, "concurrent replacement\n").unwrap(),
        )
        .unwrap_err();

        assert!(error.contains("without replacement"));
        assert_eq!(
            fs::read_to_string(&target).unwrap(),
            "concurrent replacement\n"
        );
        assert_eq!(fs::read_to_string(&backup).unwrap(), "original file\n");
    }

    #[cfg(unix)]
    #[test]
    fn parent_preflight_never_creates_workflows_through_an_existing_symlink() {
        use std::os::unix::fs::symlink;

        let tempdir = tempfile::tempdir().unwrap();
        let root = tempdir.path();
        let outside = root.join("outside");
        fs::create_dir(&outside).unwrap();
        symlink(&outside, root.join(".github")).unwrap();
        let files = template_files(true);
        let targets = files
            .iter()
            .map(|file| checked_target(root, file.relative_path).unwrap())
            .collect::<Vec<_>>();

        assert!(prepare_output_parents(root, &targets).is_err());
        assert!(!outside.join("workflows").exists());
    }

    #[test]
    fn force_race_preserves_the_original_file_in_staging_backup() {
        let tempdir = tempfile::tempdir().unwrap();
        let root = tempdir.path();
        let files = template_files(false);
        let targets = files
            .iter()
            .map(|file| checked_target(root, file.relative_path).unwrap())
            .collect::<Vec<_>>();
        fs::create_dir_all(targets[0].parent().unwrap()).unwrap();
        fs::write(&targets[0], "original file\n").unwrap();
        let staging = create_staging_dir(root).unwrap();

        let failure = install_files_with_hooks(
            &staging,
            root,
            &files,
            &targets,
            true,
            InstallHooks {
                after_revalidation: |_: &Path| {},
                after_backup: |target: &Path| {
                    fs::write(target, "concurrent replacement\n").unwrap()
                },
                after_install: |_: &Path| {},
            },
        )
        .unwrap_err();

        assert!(failure.preserve_staging, "{}", failure.error);
        assert!(failure
            .error
            .to_string()
            .contains(&staging.display().to_string()));
        assert_eq!(
            fs::read_to_string(staging.join(".backup/0")).unwrap(),
            "original file\n"
        );
        assert_eq!(
            fs::read_to_string(&targets[0]).unwrap(),
            "concurrent replacement\n"
        );
    }

    #[test]
    fn force_failure_after_backup_restores_the_original_target() {
        let tempdir = tempfile::tempdir().unwrap();
        let root = tempdir.path();
        let files = template_files(false);
        let targets = files
            .iter()
            .map(|file| checked_target(root, file.relative_path).unwrap())
            .collect::<Vec<_>>();
        fs::create_dir_all(targets[0].parent().unwrap()).unwrap();
        fs::write(&targets[0], "original file\n").unwrap();
        let staging = create_staging_dir(root).unwrap();
        let staged_smoke = staging.join("roomci/smoke.yaml");

        let failure = install_files_with_hooks(
            &staging,
            root,
            &files,
            &targets,
            true,
            InstallHooks {
                after_revalidation: |_: &Path| {},
                after_backup: move |target: &Path| {
                    if target.ends_with("roomci/smoke.yaml") {
                        fs::remove_file(&staged_smoke).unwrap();
                    }
                },
                after_install: |_: &Path| {},
            },
        )
        .unwrap_err();

        assert!(!failure.preserve_staging, "{}", failure.error);
        assert_eq!(fs::read_to_string(&targets[0]).unwrap(), "original file\n");
    }

    #[test]
    fn non_force_race_never_renames_or_overwrites_the_new_target() {
        let tempdir = tempfile::tempdir().unwrap();
        let root = tempdir.path();
        let files = template_files(false);
        let targets = files
            .iter()
            .map(|file| checked_target(root, file.relative_path).unwrap())
            .collect::<Vec<_>>();
        let staging = create_staging_dir(root).unwrap();

        let failure = install_files_with_hooks(
            &staging,
            root,
            &files,
            &targets,
            false,
            InstallHooks {
                after_revalidation: |target: &Path| {
                    if target.ends_with("roomci/smoke.yaml") {
                        fs::create_dir_all(target.parent().unwrap()).unwrap();
                        fs::write(target, "concurrent target\n").unwrap();
                    }
                },
                after_backup: |_: &Path| {},
                after_install: |_: &Path| {},
            },
        )
        .unwrap_err();

        assert!(!failure.preserve_staging);
        assert_eq!(
            fs::read_to_string(&targets[0]).unwrap(),
            "concurrent target\n"
        );
        assert!(!staging.join(".backup/0").exists());
    }

    #[test]
    fn rollback_preserves_replaced_a_when_b_conflicts() {
        let tempdir = tempfile::tempdir().unwrap();
        let root = tempdir.path();
        let files = template_files(false);
        let targets = files
            .iter()
            .map(|file| checked_target(root, file.relative_path).unwrap())
            .collect::<Vec<_>>();
        let installed_a = targets[0].clone();
        let conflicting_b = targets[1].clone();
        let staging = create_staging_dir(root).unwrap();

        let failure = install_files_with_hooks(
            &staging,
            root,
            &files,
            &targets,
            false,
            InstallHooks {
                after_revalidation: move |target: &Path| {
                    if target == conflicting_b {
                        fs::write(target, "B concurrent target\n").unwrap();
                    }
                },
                after_backup: |_: &Path| {},
                after_install: move |target: &Path| {
                    if target == installed_a {
                        fs::remove_file(target).unwrap();
                        fs::write(target, "A concurrent replacement\n").unwrap();
                    }
                },
            },
        )
        .unwrap_err();

        assert!(failure.preserve_staging, "{}", failure.error);
        assert_eq!(
            fs::read_to_string(&targets[0]).unwrap(),
            "A concurrent replacement\n"
        );
        assert_eq!(
            fs::read_to_string(&targets[1]).unwrap(),
            "B concurrent target\n"
        );
        assert!(staging.join("roomci/smoke.yaml").exists());
    }

    #[cfg(unix)]
    #[test]
    fn mutation_revalidation_rejects_a_root_replaced_with_a_symlink() {
        use std::os::unix::fs::symlink;

        let tempdir = tempfile::tempdir().unwrap();
        let root = tempdir.path().join("project");
        let target = root.join("roomci/smoke.yaml");
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        let outside = tempdir.path().join("outside");
        fs::create_dir(&outside).unwrap();
        fs::rename(&root, tempdir.path().join("project-original")).unwrap();
        symlink(&outside, &root).unwrap();

        let error = revalidate_target_for_mutation(&root, &target, false).unwrap_err();

        assert!(error.to_string().contains("symbolic link"));
        assert!(!outside.join("roomci/smoke.yaml").exists());
    }
}
