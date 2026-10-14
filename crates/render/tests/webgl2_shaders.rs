//! Every shader of the core path must translate to GLSL ES 3.00 for WebGL2
//! (PLAN.md 4.1). This runs without a GPU or browser.

use naga::back::glsl;

fn translate_for_webgl2(source: &str, stage: naga::ShaderStage, entry_point: &str) -> String {
    let module = naga::front::wgsl::parse_str(source).expect("WGSL parses");
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .expect("module validates without optional capabilities");
    let options = glsl::Options {
        version: glsl::Version::Embedded {
            version: 300,
            is_webgl: true,
        },
        ..Default::default()
    };
    let pipeline = glsl::PipelineOptions {
        shader_stage: stage,
        entry_point: entry_point.to_owned(),
        multiview: None,
    };
    let mut out = String::new();
    glsl::Writer::new(
        &mut out,
        &module,
        &info,
        &options,
        &pipeline,
        naga::proc::BoundsCheckPolicies::default(),
    )
    .expect("GLSL ES 3.00 writer accepts the module")
    .write()
    .expect("entry point translates to GLSL ES 3.00");
    out
}

#[test]
fn palette_quad_translates_to_glsl_es_300() {
    let src = glasnost_render::palette_quad::SHADER_SOURCE;
    let vs = translate_for_webgl2(src, naga::ShaderStage::Vertex, "vs_main");
    let fs = translate_for_webgl2(src, naga::ShaderStage::Fragment, "fs_main");
    assert!(vs.starts_with("#version 300 es"));
    assert!(
        fs.contains("texelFetch"),
        "index lookups must be plain texel fetches"
    );
}

#[test]
fn map_shader_translates_to_glsl_es_300() {
    let src = glasnost_render::map_gpu::SHADER_SOURCE;
    let vs = translate_for_webgl2(src, naga::ShaderStage::Vertex, "vs_main");
    let fs = translate_for_webgl2(src, naga::ShaderStage::Fragment, "fs_main");
    assert!(vs.starts_with("#version 300 es"));
    assert!(fs.contains("gl_FragDepth"), "depth is written per fragment");
    assert!(fs.contains("texelFetch"));
}

#[test]
fn batch2d_translates_to_glsl_es_300() {
    let src = glasnost_render::batch2d::SHADER_SOURCE;
    let vs = translate_for_webgl2(src, naga::ShaderStage::Vertex, "vs_main");
    let fs = translate_for_webgl2(src, naga::ShaderStage::Fragment, "fs_main");
    assert!(vs.starts_with("#version 300 es"));
    assert!(fs.contains("texelFetch"));
    assert!(!vs.contains("gl_BaseVertex") && !vs.contains("gl_BaseInstance"));
}
