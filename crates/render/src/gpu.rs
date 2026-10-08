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
    let adapter_limits = adapter.limits();
    wgpu::Limits::downlevel_webgl2_defaults()
        .using_resolution(adapter_limits.clone())
        .using_alignment(adapter_limits)
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
}
