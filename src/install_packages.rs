use std::collections::HashMap;
use std::env::temp_dir;
use std::ffi::OsString;
use std::fs::File;
use std::io::ErrorKind;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ar::Archive as ArArchive;
use async_compression::tokio::bufread::{GzipDecoder, XzDecoder, ZstdDecoder};
use futures::io::AllowStdIo;
use futures::TryStreamExt;
use indexmap::IndexSet;
use libcnb::build::BuildContext;
use libcnb::data::layer::LayerNameError;
use libcnb::data::layer_name;
use libcnb::layer::{
    CachedLayerDefinition, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use libcnb::layer_env::{LayerEnv, ModificationBehavior, Scope};
use reqwest_middleware::ClientWithMiddleware;
use reqwest_middleware::Error::Reqwest;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::fs::{read_to_string as async_read_to_string, write as async_write, File as AsyncFile};
use tokio::io::{copy as async_copy, BufReader as AsyncBufReader, BufWriter as AsyncBufWriter};
use tokio::task::{JoinError, JoinSet};
use tokio_tar::Archive as TarArchive;
use tokio_util::compat::FuturesAsyncReadCompatExt;
use tokio_util::io::InspectReader;
use walkdir::{DirEntry, WalkDir};

use crate::debian::{Distro, MultiarchName, RepositoryPackage};
use crate::install_packages::InstallPackagesError::{
    ChecksumFailed, CreateLayer, DownloadPackage, NoFilename, OpenPackageArchive,
    OpenPackageArchiveEntry, ReadPackageConfig, TaskFailed, UnpackTarball, UnsupportedCompression,
    WriteLayerEnv, WriteLayerMetadata, WritePackage, WritePackageConfig,
};
use crate::{DebianPackagesBuildpack, DebianPackagesBuildpackError};

type Result<T> = std::result::Result<T, InstallPackagesError>;

pub(crate) async fn install_packages(
    context: &Arc<BuildContext<DebianPackagesBuildpack>>,
    client: &ClientWithMiddleware,
    distro: &Distro,
    packages_to_install: Vec<RepositoryPackage>,
) -> Result<()> {
    println!("## Installing packages");
    println!();

    let new_metadata = InstallationMetadata {
        package_checksums: packages_to_install
            .iter()
            .map(|package| (package.name.to_string(), package.sha256sum.to_string()))
            .collect(),
        distro: distro.clone(),
    };

    let install_layer = context
        .cached_layer(
            layer_name!("packages"),
            CachedLayerDefinition {
                build: true,
                launch: true,
                invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
                restored_layer_action: &|old_metadata: &InstallationMetadata, _| {
                    if old_metadata == &new_metadata {
                        RestoredLayerAction::KeepLayer
                    } else {
                        RestoredLayerAction::DeleteLayer
                    }
                },
            },
        )
        .map_err(|e| CreateLayer(Box::new(e)))?;

    match install_layer.state {
        LayerState::Restored { .. } => {
            for repository_package in packages_to_install {
                println!("  Restoring {} from cache", repository_package.name);
            }
        }
        LayerState::Empty { .. } => {
            install_layer
                .write_metadata(new_metadata)
                .map_err(|e| WriteLayerMetadata(Box::new(e)))?;

            let mut download_and_extract_handles = JoinSet::new();

            for repository_package in packages_to_install {
                download_and_extract_handles.spawn(download_and_extract(
                    client.clone(),
                    repository_package.clone(),
                    install_layer.path(),
                ));
            }

            while let Some(download_and_extract_handle) =
                download_and_extract_handles.join_next().await
            {
                download_and_extract_handle.map_err(TaskFailed)??;
            }
        }
    }

    let layer_env = configure_layer_environment(
        &install_layer.path(),
        &MultiarchName::from(&distro.architecture),
    );

    install_layer
        .write_env(layer_env)
        .map_err(|e| WriteLayerEnv(Box::new(e)))?;

    rewrite_package_configs(&install_layer.path()).await?;

    println!();
    Ok(())
}

async fn download_and_extract(
    client: ClientWithMiddleware,
    repository_package: RepositoryPackage,
    install_dir: PathBuf,
) -> Result<()> {
    println!("  Downloading and extracting {}", repository_package.name);
    let download_path = download(client, &repository_package).await?;
    extract(download_path, install_dir).await
}

async fn download(
    client: ClientWithMiddleware,
    repository_package: &RepositoryPackage,
) -> Result<PathBuf> {
    let download_url = format!(
        "{}/{}",
        repository_package.repository_uri.as_str(),
        repository_package.filename.as_str()
    );

    let download_file_name = PathBuf::from(repository_package.filename.as_str())
        .file_name()
        .map(ToOwned::to_owned)
        .ok_or(NoFilename)?;

    let download_path = temp_dir().join::<&Path>(download_file_name.as_ref());

    let response = client
        .get(&download_url)
        .send()
        .await
        .and_then(|res| res.error_for_status().map_err(Reqwest))
        .map_err(DownloadPackage)?;

    let mut hasher = Sha256::new();

    let mut writer = AsyncFile::create(&download_path)
        .await
        .map_err(WritePackage)
        .map(AsyncBufWriter::new)?;

    // the inspect reader lets us pipe the response to both the output file and the hash digest
    let mut reader = AsyncBufReader::new(InspectReader::new(
        // and we need to convert the http stream into an async reader
        FuturesAsyncReadCompatExt::compat(
            response
                .bytes_stream()
                .map_err(|e| std::io::Error::new(ErrorKind::Other, e))
                .into_async_read(),
        ),
        |bytes| hasher.update(bytes),
    ));

    async_copy(&mut reader, &mut writer)
        .await
        .map_err(WritePackage)?;

    let calculated_hash = format!("{:x}", hasher.finalize());

    if repository_package.sha256sum != calculated_hash {
        Err(ChecksumFailed(
            download_url,
            repository_package.sha256sum.to_string(),
            calculated_hash,
        ))?;
    }

    Ok(download_path)
}

async fn extract(download_path: PathBuf, output_dir: PathBuf) -> Result<()> {
    // a .deb file is an ar archive
    // https://manpages.ubuntu.com/manpages/jammy/en/man5/deb.5.html
    let mut debian_archive = File::open(download_path)
        .map_err(OpenPackageArchive)
        .map(ArArchive::new)?;

    while let Some(entry) = debian_archive.next_entry() {
        let entry = entry.map_err(OpenPackageArchiveEntry)?;
        let entry_path = PathBuf::from(OsString::from_vec(entry.header().identifier().to_vec()));
        let entry_reader =
            AsyncBufReader::new(FuturesAsyncReadCompatExt::compat(AllowStdIo::new(entry)));

        // https://manpages.ubuntu.com/manpages/noble/en/man5/deb.5.html
        match (
            entry_path.file_stem().and_then(|v| v.to_str()),
            entry_path.extension().and_then(|v| v.to_str()),
        ) {
            (Some("data.tar"), Some("gz")) => {
                let mut tar_archive = TarArchive::new(GzipDecoder::new(entry_reader));
                tar_archive
                    .unpack(&output_dir)
                    .await
                    .map_err(UnpackTarball)?;
            }
            (Some("data.tar"), Some("zstd" | "zst")) => {
                let mut tar_archive = TarArchive::new(ZstdDecoder::new(entry_reader));
                tar_archive
                    .unpack(&output_dir)
                    .await
                    .map_err(UnpackTarball)?;
            }
            (Some("data.tar"), Some("xz")) => {
                let mut tar_archive = TarArchive::new(XzDecoder::new(entry_reader));
                tar_archive
                    .unpack(&output_dir)
                    .await
                    .map_err(UnpackTarball)?;
            }
            (Some("data.tar"), Some(compression)) => {
                Err(UnsupportedCompression(compression.to_string()))?;
            }
            _ => {
                // ignore other potential file entries (e.g.; debian-binary, control.tar)
            }
        };
    }

    Ok(())
}

fn configure_layer_environment(install_path: &Path, multiarch_name: &MultiarchName) -> LayerEnv {
    let mut layer_env = LayerEnv::new();

    let bin_paths = [
        install_path.join("bin"),
        install_path.join("usr/bin"),
        install_path.join("usr/sbin"),
    ];
    prepend_to_env_var(&mut layer_env, "PATH", &bin_paths);

    // support multi-arch and legacy filesystem layouts for debian packages
    // https://wiki.ubuntu.com/MultiarchSpec
    let library_paths = [
        install_path.join(format!("usr/lib/{multiarch_name}")),
        install_path.join("usr/lib"),
        install_path.join(format!("lib/{multiarch_name}")),
        install_path.join("lib"),
    ]
    .iter()
    .fold(IndexSet::new(), |mut acc, lib_dir| {
        for dir in find_all_dirs_containing(lib_dir, shared_library_file) {
            acc.insert(dir);
        }
        acc.insert(lib_dir.clone());
        acc
    });
    prepend_to_env_var(&mut layer_env, "LD_LIBRARY_PATH", &library_paths);
    prepend_to_env_var(&mut layer_env, "LIBRARY_PATH", &library_paths);

    let include_paths = [
        install_path.join(format!("usr/include/{multiarch_name}")),
        install_path.join("usr/include"),
    ]
    .iter()
    .fold(IndexSet::new(), |mut acc, include_dir| {
        for dir in find_all_dirs_containing(include_dir, header_file) {
            acc.insert(dir);
        }
        acc.insert(include_dir.clone());
        acc
    });
    prepend_to_env_var(&mut layer_env, "INCLUDE_PATH", &include_paths);
    prepend_to_env_var(&mut layer_env, "CPATH", &include_paths);
    prepend_to_env_var(&mut layer_env, "CPPPATH", &include_paths);

    let pkg_config_paths = [
        install_path.join(format!("usr/lib/{multiarch_name}/pkgconfig")),
        install_path.join("usr/lib/pkgconfig"),
    ];
    prepend_to_env_var(&mut layer_env, "PKG_CONFIG_PATH", &pkg_config_paths);

    layer_env
}

fn find_all_dirs_containing(
    starting_dir: &Path,
    condition: impl Fn(&Path) -> bool,
) -> Vec<PathBuf> {
    let mut matches = vec![];
    if let Ok(true) = starting_dir.try_exists() {
        for entry in WalkDir::new(starting_dir).into_iter().flatten() {
            if let Some(parent_dir) = entry.path().parent() {
                if condition(entry.path()) {
                    matches.push(parent_dir.to_path_buf());
                }
            }
        }
    }
    // order the paths by longest to shortest
    matches.sort_by_key(|v| std::cmp::Reverse(v.as_os_str().len()));
    matches
}

fn shared_library_file(path: &Path) -> bool {
    let mut current_path = path.to_path_buf();
    let mut current_ext = current_path.extension();
    let mut current_name = current_path.file_stem();
    while let (Some(name), Some(ext)) = (current_name, current_ext) {
        if ext == "so" {
            return true;
        }
        current_path = PathBuf::from(name);
        current_ext = current_path.extension();
        current_name = current_path.file_stem();
    }
    false
}

fn header_file(path: &Path) -> bool {
    matches!(path.extension(), Some(ext) if ext == "h")
}

fn prepend_to_env_var<I, T>(layer_env: &mut LayerEnv, name: &str, paths: I)
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let separator = ":";
    layer_env.insert(Scope::All, ModificationBehavior::Delimiter, name, separator);
    layer_env.insert(
        Scope::All,
        ModificationBehavior::Prepend,
        name,
        paths
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>()
            .join(separator.as_ref()),
    );
}

async fn rewrite_package_configs(install_path: &Path) -> Result<()> {
    let package_configs = WalkDir::new(install_path)
        .into_iter()
        .flatten()
        .filter(is_package_config)
        .map(|entry| entry.path().to_path_buf());

    for package_config in package_configs {
        rewrite_package_config(&package_config, install_path).await?;
    }

    Ok(())
}

fn is_package_config(entry: &DirEntry) -> bool {
    matches!((
        entry.path().parent().and_then(|p| p.file_name()),
        entry.path().extension()
    ), (Some(parent), Some(ext)) if parent == "pkgconfig" && ext == "pc")
}

async fn rewrite_package_config(package_config: &Path, install_path: &Path) -> Result<()> {
    let contents = async_read_to_string(package_config)
        .await
        .map_err(ReadPackageConfig)?;

    let new_contents = contents
        .lines()
        .map(|line| {
            if let Some(prefix_value) = line.strip_prefix("prefix=") {
                format!(
                    "prefix={}",
                    install_path
                        .join(prefix_value.trim_start_matches('/'))
                        .to_string_lossy()
                )
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    async_write(package_config, new_contents)
        .await
        .map_err(WritePackageConfig)
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum InstallPackagesError {
    TaskFailed(JoinError),
    InvalidLayerName(LayerNameError),
    NoFilename,
    CreateLayer(Box<libcnb::Error<DebianPackagesBuildpackError>>),
    DownloadPackage(reqwest_middleware::Error),
    WritePackage(std::io::Error),
    ChecksumFailed(String, String, String),
    OpenPackageArchive(std::io::Error),
    OpenPackageArchiveEntry(std::io::Error),
    UnpackTarball(std::io::Error),
    WriteLayerMetadata(Box<libcnb::Error<DebianPackagesBuildpackError>>),
    WriteLayerEnv(Box<libcnb::Error<DebianPackagesBuildpackError>>),
    UnsupportedCompression(String),
    ReadPackageConfig(std::io::Error),
    WritePackageConfig(std::io::Error),
    ConfigurePaths(std::io::Error),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
struct InstallationMetadata {
    package_checksums: HashMap<String, String>,
    distro: Distro,
}

#[cfg(test)]
mod test {
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    use libcnb::layer_env::Scope;
    use tempfile::TempDir;

    use crate::debian::MultiarchName;
    use crate::install_packages::configure_layer_environment;

    #[test]
    fn configure_layer_environment_adds_nested_directories_with_shared_libraries_to_library_path() {
        let arch = MultiarchName::X86_64_LINUX_GNU;
        let install_dir = create_installation(bon::vec![
            format!("usr/lib/{arch}/nested-1/shared-library.so.2"),
            "usr/lib/nested-2/shared-library.so",
            format!("lib/{arch}/nested-3/shared-library.so.4"),
            format!("lib/{arch}/nested-3/deeply/shared-library.so.5"),
            "lib/nested-4/shared-library.so.3.0.0",
            "usr/lib/nested-but/does-not-contain-a-shared-library.txt",
            "usr/not-a-lib-dir/shared-library.so.6"
        ]);
        let install_path = install_dir.path();
        let layer_env = configure_layer_environment(install_path, &arch);
        assert_eq!(
            split_into_paths(layer_env.apply_to_empty(Scope::All).get("LD_LIBRARY_PATH")),
            vec![
                install_path.join(format!("usr/lib/{arch}/nested-1")),
                install_path.join(format!("usr/lib/{arch}")),
                install_path.join("usr/lib/nested-2"),
                install_path.join("usr/lib"),
                install_path.join(format!("lib/{arch}/nested-3/deeply")),
                install_path.join(format!("lib/{arch}/nested-3")),
                install_path.join(format!("lib/{arch}")),
                install_path.join("lib/nested-4"),
                install_path.join("lib"),
            ]
        );
    }

    #[test]
    fn configure_layer_environment_adds_nested_directories_with_headers_to_include_path() {
        let arch = MultiarchName::X86_64_LINUX_GNU;
        let install_dir = create_installation(bon::vec![
            format!("usr/include/{arch}/nested-1/header.h"),
            "usr/include/nested-2/header.h",
            "usr/include/nested-2/deeply/header.h",
            "usr/include/nested-but/does-not-contain-a-header.txt",
            "usr/not-an-include-dir/header.h"
        ]);
        let install_path = install_dir.path();
        let layer_env = configure_layer_environment(install_path, &arch);
        assert_eq!(
            split_into_paths(layer_env.apply_to_empty(Scope::All).get("INCLUDE_PATH")),
            vec![
                install_path.join(format!("usr/include/{arch}/nested-1")),
                install_path.join(format!("usr/include/{arch}")),
                install_path.join("usr/include/nested-2/deeply"),
                install_path.join("usr/include/nested-2"),
                install_path.join("usr/include"),
            ]
        );
    }

    fn create_installation(files: Vec<String>) -> TempDir {
        let install_dir = tempfile::tempdir().unwrap();
        for file in files {
            create_file(install_dir.path(), &file);
        }
        install_dir
    }

    fn create_file(install_dir: &Path, file: &str) {
        let file_path = install_dir.join(file);
        let dir = file_path.parent().unwrap();
        std::fs::create_dir_all(dir).unwrap();
        std::fs::File::create(file_path).unwrap();
    }

    fn split_into_paths(env_var: Option<&OsString>) -> Vec<PathBuf> {
        env_var
            .map(|v| {
                v.to_string_lossy()
                    .split(':')
                    .map(PathBuf::from)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }
}