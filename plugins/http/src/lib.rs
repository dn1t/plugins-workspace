// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [![](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/http/banner.png)](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/http)
//!
//! Access the HTTP client written in Rust.

use std::{
    io::Write,
    path::{self, PathBuf},
};

pub use reqwest;
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};
use tauri::{
    path::PathResolver,
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use error::{Error, Result};

mod commands;
mod error;
mod scope;

pub(crate) struct Http {
    #[cfg(feature = "cookies")]
    cookies_jar: std::sync::Arc<reqwest_cookie_store::CookieStoreMutex>,
    #[cfg(feature = "cookies")]
    path: PathBuf,
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::<R>::new("http")
        .setup(|app, _| {
            #[cfg(feature = "cookies")]
            let appDataDir = {
                let tmp = app.path().app_data_dir().unwrap();
                if !tmp.exists() {
                    std::fs::create_dir_all(tmp.clone());
                }
                tmp
            };
            #[cfg(feature = "cookies")]
            let cookiesPath = appDataDir.join("cookies.json");
            let state = Http {
                #[cfg(feature = "cookies")]
                cookies_jar: std::sync::Arc::new(
                    match std::fs::File::open(&cookiesPath).map(std::io::BufReader::new) {
                        Ok(reader) => {
                            CookieStoreMutex::new(CookieStore::load_json(reader).unwrap())
                        }
                        Err(e) => CookieStoreMutex::new(CookieStore::default()),
                    },
                ),
                #[cfg(feature = "cookies")]
                path: cookiesPath,
            };

            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::fetch,
            commands::fetch_cancel,
            commands::fetch_send,
            commands::fetch_read_body,
        ])
        .build()
}
