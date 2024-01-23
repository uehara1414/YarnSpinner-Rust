use crate::prelude::*;
use anyhow::ensure;
use bevy::{prelude::*, reflect::TypePath};
use glob::glob;
use std::path::PathBuf;

/// Possible sources to load a [`YarnFile`] from.
#[derive(Debug, Clone, PartialEq, Eq, Hash, TypePath)]
pub enum YarnFileSource {
    /// A [`YarnFile`] that is already present in the asset server, addressed by its [`Handle`].
    Handle(Handle<YarnFile>),
    /// A [`YarnFile`] that is already present in memory, created with [`YarnFile::new`].
    InMemory(YarnFile),
    /// A [`YarnFile`] inside the `assets` folder. This will be loaded into the [`AssetServer`].
    /// Use [`YarnFileSource::file`] for convenience.
    File(PathBuf),
    /// A folder inside the `assets` folder which is searched for [`YarnFile`]s recursively, loading all files with the `.yarn` extension into the [`AssetServer`].
    /// Use [`YarnFileSource::folder`] for convenience.
    ///
    /// Not supported on Wasm and Android because Bevy cannot load folders on these platforms.
    Folder(PathBuf),
}

impl From<Handle<YarnFile>> for YarnFileSource {
    fn from(handle: Handle<YarnFile>) -> Self {
        Self::Handle(handle)
    }
}

impl From<YarnFile> for YarnFileSource {
    fn from(yarn_file: YarnFile) -> Self {
        Self::InMemory(yarn_file)
    }
}

impl YarnFileSource {
    /// Convenience function to create a [`YarnFileSource::File`] from a path.
    pub fn file(path: impl Into<PathBuf>) -> Self {
        Self::File(path.into())
    }

    /// Convenience function to create a [`YarnFileSource::folder`] from a path.
    /// Panics on Wasm and Android because Bevy cannot load folders on these platforms.
    pub fn folder(path: impl Into<PathBuf>) -> Self {
        #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
        {
            Self::Folder(path.into())
        }
        #[cfg(any(target_arch = "wasm32", target_os = "android"))]
        {
            let _ = path;
            panic!("YarnFileSource::folder is not supported on this platform. Help: Use YarnFileSource::file instead and specify all Yarn files you want to load.")
        }
    }

    pub(crate) fn load(
        &self,
        asset_server: &AssetServer,
        assets: &mut ResMut<Assets<YarnFile>>,
    ) -> Result<Vec<Handle<YarnFile>>> {
        match self {
            Self::Handle(handle) => Ok(vec![handle.clone()]),
            Self::InMemory(yarn_file) => Ok(vec![assets.add(yarn_file.clone())]),
            Self::File(path) => Ok(vec![asset_server.load(path.clone())]),
            Self::Folder(path) => {
                #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
                {
                    Self::load_folder(asset_server, path)
                }
                #[cfg(any(target_arch = "wasm32", target_os = "android"))]
                {
                    let _ = path;
                    panic!("YarnFileSource::Folder is not supported on this platform. Help: Use YarnFileSource::File instead and specify all Yarn files you want to load.")
                }
            }
        }
    }

    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    fn load_folder(
        asset_server: &AssetServer,
        path: &std::path::Path,
    ) -> Result<Vec<Handle<YarnFile>>> {
        // recursively glob
        let root = asset_server.get_assets_dir_path()?;
        let path = root.join(path);
        ensure!(path.is_dir(), "Failed to load Yarn file folder {path}.\nHelp: Does the folder exist under the assets directory?", path = path.display());
        let handles: Result<Vec<_>> =
            glob(path.join("**/*.yarn").to_str().with_context(|| {
                format!(
                    "Failed to create string from path: {path}",
                    path = path.display(),
                )
            })?)?
            .map(|entry| {
                let full_path = entry?;
                let path = full_path.strip_prefix(&root)?;
                let path = path.to_str().unwrap();
                Ok(asset_server.load(path.to_owned()))
            })
            .collect();
        let handles = handles?;

        if handles.is_empty() {
            warn!("No Yarn files found in the assets subdirectory {path}, so Yarn Slinger won't be able to do anything this run. \
                        Help: Add some Yarn files to get started.", path = path.display());
        }
        Ok(handles)
    }
}
