//! Instance, adapter and device setup with the backend policy from PLAN.md 4.1:
//! WebGL2 is the core feature set; WebGPU is selected at runtime on the web
//! where the browser offers it.

use std::fmt;
use std::str::FromStr;

/// Which graphics backend to ask for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BackendChoice {
    /// Web: WebGPU if the browser offers a working adapter, else WebGL2.
    /// Native: wgpu's default (`WGPU_BACKEND` overrides).
    #[default]
    Auto,
    /// Web: WebGL2 only. Native: the OpenGL/GLES backend, the closest native
    /// stand-in for the WebGL2 path.
    Gl,
    /// Web: WebGPU only (fails where the browser has none). Native: the
    /// platform's primary API (Vulkan, Metal or DX12).
    Primary,
}

impl FromStr for BackendChoice {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "" | "auto" => Ok(Self::Auto),
            "gl" | "webgl" | "webgl2" => Ok(Self::Gl),
            "primary" | "webgpu" | "vulkan" => Ok(Self::Primary),
            other => Err(format!(
                "unknown backend {other:?}; expected auto, webgl2 or webgpu (native: gl or vulkan)"
            )),
        }
    }
}

impl BackendChoice {
    fn backends(self) -> wgpu::Backends {
        let web = cfg!(target_arch = "wasm32");
        match (self, web) {
            (Self::Auto, true) => wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL,
            (Self::Auto, false) => wgpu::Backends::all().with_env(),
            (Self::Gl, _) => wgpu::Backends::GL,
            (Self::Primary, true) => wgpu::Backends::BROWSER_WEBGPU,
            (Self::Primary, false) => wgpu::Backends::PRIMARY,
        }
    }
}

/// Creates the wgpu instance for `choice`.
///
/// On the web with [`BackendChoice::Auto`] this probes for a working WebGPU
/// adapter first, because some browsers expose `navigator.gpu` without being
/// able to create one. `display` should be the windowing system's display
/// handle where one exists (required by the GL backend on Wayland).
pub async fn create_instance(
    choice: BackendChoice,
    display: Option<Box<dyn wgpu::wgt::WgpuHasDisplayHandle>>,
) -> wgpu::Instance {
    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle().with_env();
    desc.backends = choice.backends();
    desc.display = display;
    if desc.backends.contains(wgpu::Backends::BROWSER_WEBGPU) {
        wgpu::util::new_instance_with_webgpu_detection(desc).await
    } else {
        wgpu::Instance::new(desc)
    }
}

/// Errors from adapter or device setup.
#[derive(Debug)]
pub enum GpuError {
    /// No adapter matched.
    NoAdapter(wgpu::RequestAdapterError),
    /// The adapter refused the device request.
    NoDevice(wgpu::RequestDeviceError),
}

impl fmt::Display for GpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoAdapter(e) => write!(f, "no suitable graphics adapter: {e}"),
            Self::NoDevice(e) => write!(f, "could not open the graphics device: {e}"),
        }
    }
}

impl std::error::Error for GpuError {}

/// Requests an adapter. On WebGL2 a `surface` is mandatory.
pub async fn request_adapter(
    instance: &wgpu::Instance,
    surface: Option<&wgpu::Surface<'_>>,
) -> Result<wgpu::Adapter, GpuError> {
    instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::from_env().unwrap_or_default(),
            force_fallback_adapter: false,
            compatible_surface: surface,
            ..Default::default()
        })
        .await
        .map_err(GpuError::NoAdapter)
}

/// The limits the core renderer requests: the WebGL2 floor, with texture
/// sizes and buffer alignments taken from the adapter so atlases can grow to
/// what the hardware offers (PLAN.md 4.1).
pub fn core_limits(adapter: &wgpu::Adapter) -> wgpu::Limits {
    limits_for(&adapter.limits())
}

/// [`core_limits`] for given adapter limits. No limit is requested beyond what
/// the adapter reports: some browsers report less than the WebGL2 floor (a
/// privacy mode can report WebGL1 minimums, such as 8 vertex attributes), and
/// a device request above the adapter's limits fails outright. The renderer's
/// pipelines stay within those lower values.
pub fn limits_for(adapter_limits: &wgpu::Limits) -> wgpu::Limits {
    wgpu::Limits::downlevel_webgl2_defaults()
        .using_resolution(adapter_limits.clone())
        .using_alignment(adapter_limits.clone())
        .or_worse_values_from(adapter_limits)
}

/// Opens a device with [`core_limits`] and no optional features. WebGPU-only
/// extras must check [`extras_available`] and request their features
/// separately.
pub async fn request_device(
    adapter: &wgpu::Adapter,
) -> Result<(wgpu::Device, wgpu::Queue), GpuError> {
    adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("glasnost"),
            required_features: wgpu::Features::empty(),
            required_limits: core_limits(adapter),
            ..Default::default()
        })
        .await
        .map_err(GpuError::NoDevice)
}

/// Whether the adapter supports the full WebGPU feature level, i.e. whether
/// optional extras beyond the WebGL2 core path may be enabled.
pub fn extras_available(adapter: &wgpu::Adapter) -> bool {
    adapter.get_downlevel_capabilities().is_webgpu_compliant()
}

/// A short, user-facing name for the adapter's backend.
pub fn backend_label(backend: wgpu::Backend) -> &'static str {
    match backend {
        wgpu::Backend::BrowserWebGpu => "WebGPU",
        wgpu::Backend::Gl if cfg!(target_arch = "wasm32") => "WebGL2",
        wgpu::Backend::Gl => "OpenGL",
        wgpu::Backend::Vulkan => "Vulkan",
        wgpu::Backend::Metal => "Metal",
        wgpu::Backend::Dx12 => "DirectX 12",
        wgpu::Backend::Noop => "no-op",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_backend_names() {
        assert_eq!("".parse(), Ok(BackendChoice::Auto));
        assert_eq!("WebGL2".parse(), Ok(BackendChoice::Gl));
        assert_eq!("webgpu".parse(), Ok(BackendChoice::Primary));
        assert!("metal-ish".parse::<BackendChoice>().is_err());
    }

    #[test]
    fn webgl2_floor_is_kept() {
        let floor = wgpu::Limits::downlevel_webgl2_defaults();
        let big = wgpu::Limits {
            max_texture_dimension_2d: 16384,
            ..Default::default()
        };
        let merged = floor.clone().using_resolution(big);
        assert_eq!(merged.max_texture_dimension_2d, 16384);
        assert_eq!(
            merged.max_storage_buffers_per_shader_stage,
            floor.max_storage_buffers_per_shader_stage
        );
        assert_eq!(merged.max_compute_workgroups_per_dimension, 0);
    }

    #[test]
    fn no_limit_above_the_adapter_is_requested() {
        // a browser in a privacy mode reports WebGL1 minimums
        let adapter = wgpu::Limits {
            max_vertex_attributes: 8,
            ..wgpu::Limits::downlevel_webgl2_defaults()
        };
        let requested = limits_for(&adapter);
        assert_eq!(requested.max_vertex_attributes, 8);
        assert!(requested.check_limits(&adapter));
    }

    #[test]
    fn the_webgl2_floor_is_kept_on_a_capable_adapter() {
        let adapter = wgpu::Limits::default();
        let requested = limits_for(&adapter);
        let floor = wgpu::Limits::downlevel_webgl2_defaults();
        assert_eq!(requested.max_vertex_attributes, floor.max_vertex_attributes);
        assert_eq!(
            requested.max_texture_dimension_2d,
            adapter.max_texture_dimension_2d
        );
        assert!(requested.check_limits(&adapter));
    }
}

/// A device, its queue and the adapter's description, shared by every
/// caller in the process (see [`shared_device`]).
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug)]
pub struct SharedDevice {
    /// The device.
    pub device: wgpu::Device,
    /// Its queue.
    pub queue: wgpu::Queue,
    /// The adapter it was opened on.
    pub info: wgpu::AdapterInfo,
}

/// Why [`shared_device`] has no device; the original errors as text, so
/// every caller sees the same outcome.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug)]
pub enum SharedGpuError {
    /// No adapter matched (tests skip unless a GPU is required).
    NoAdapter(String),
    /// The adapter refused the device request.
    NoDevice(String),
}

#[cfg(not(target_arch = "wasm32"))]
impl fmt::Display for SharedGpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoAdapter(e) | Self::NoDevice(e) => f.write_str(e),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl std::error::Error for SharedGpuError {}

/// One device for the whole process, opened on first use with
/// [`BackendChoice::Auto`] (`WGPU_BACKEND` overrides) and never closed.
///
/// Offscreen tests and tools must use this instead of opening a device of
/// their own. The test harness runs tests on several threads, and opening
/// Vulkan devices on several threads at once crashed the process: the
/// Vulkan loader segfaulted naming a debug object on one thread while
/// another thread was inside device creation (M7b step 2c, 3 runs in 250
/// of `map_gpu`). With one device per process no device is ever created
/// while another is in use.
#[cfg(not(target_arch = "wasm32"))]
pub fn shared_device() -> Result<SharedDevice, SharedGpuError> {
    static SHARED: std::sync::OnceLock<Result<SharedDevice, SharedGpuError>> =
        std::sync::OnceLock::new();
    SHARED
        .get_or_init(|| {
            pollster::block_on(async {
                let instance = create_instance(BackendChoice::Auto, None).await;
                let adapter = request_adapter(&instance, None)
                    .await
                    .map_err(|e| SharedGpuError::NoAdapter(e.to_string()))?;
                let info = adapter.get_info();
                let (device, queue) = request_device(&adapter)
                    .await
                    .map_err(|e| SharedGpuError::NoDevice(e.to_string()))?;
                Ok(SharedDevice {
                    device,
                    queue,
                    info,
                })
            })
        })
        .clone()
}
