//! Start-up status, reported to the log and, on the web, to the page.
//!
//! The web build mirrors the status into `data-glasnost-*` attributes on
//! `<body>` so the page and the browser test lane can see it.

/// Lifecycle of the demo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    /// Window created, graphics initialising.
    Starting,
    /// The first frame was presented.
    Presented {
        /// Backend label, e.g. `"WebGL2"` or `"WebGPU"`.
        backend: String,
        /// Whether WebGPU-level extras could be enabled.
        webgpu_extras: bool,
    },
    /// Start-up failed.
    Error(String),
}

impl Status {
    /// Reports this status.
    pub fn publish(&self) {
        match self {
            Self::Starting => log::info!("starting"),
            Self::Presented {
                backend,
                webgpu_extras,
            } => log::info!("first frame presented ({backend}, WebGPU extras: {webgpu_extras})"),
            Self::Error(e) => log::error!("{e}"),
        }
        #[cfg(target_arch = "wasm32")]
        self.publish_to_page();
    }

    #[cfg(target_arch = "wasm32")]
    fn publish_to_page(&self) {
        let Some(body) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.body())
        else {
            return;
        };
        let data = body.dataset();
        let (status, backend, detail) = match self {
            Self::Starting => ("starting", None, None),
            Self::Presented {
                backend,
                webgpu_extras,
            } => (
                "presented",
                Some(backend.as_str()),
                Some(if *webgpu_extras {
                    "webgpu-extras"
                } else {
                    "core"
                }),
            ),
            Self::Error(e) => ("error", None, Some(e.as_str())),
        };
        let _ = data.set("glasnostStatus", status);
        if let Some(b) = backend {
            let _ = data.set("glasnostBackend", b);
        }
        if let Some(d) = detail {
            let _ = data.set("glasnostDetail", d);
        }
    }
}
