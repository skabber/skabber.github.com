use wasm_bindgen::JsCast;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, WebGl2RenderingContext, WebGlProgram, WebGlShader,
    WebGlTexture, WebGlUniformLocation, WebGlVertexArrayObject,
};

use crate::shaders::{FRAG, VERT};

// Density ramp from sparse to dense. Wide glyphs (CJK, emoji) are squeezed
// into the 10px cell via fill_text_with_max_width in build_atlas.
const CHARS: &[char] = &[
    // Void
    ' ', ' ', ' ', ' ', ' ',
    // Faint particles
    '.', '·', '˙', '⋅', '\'', '`', ',', '‚',
    // Thin marks
    ':', ';', '¨', '˘', '¯', '~', '-', '_', '+', '=', '≈', '∼', '±',
    // Thin letters (Latin / Cyrillic / Greek)
    'i', 'l', 'ı', 'т', 'и', 'й', 'ι', 'τ',
    't', 'r', 'f', 'j', 'v', 'c', 'z', 'x', 'n', 'u', 'o', 'e',
    'с', 'є', 'ј', 'α', 'β', 'γ', 'ε', 'η', 'ν',
    // Medium caps (Latin / Cyrillic / Greek)
    'I', 'T', 'Y', 'J', 'C', 'L', 'F', 'Z', 'V', 'U', 'X',
    'А', 'Б', 'Г', 'Д', 'Е', 'И', 'К', 'Л', 'Н', 'П', 'Р', 'С',
    'Γ', 'Δ', 'Λ', 'Π', 'Σ', 'Ω',
    // Denser letters
    '1', '7', 'S', 's', 'a', 'y', 'k', 'h', 'd', 'b', 'q', 'p',
    'O', '0', 'Q', 'D', 'm', 'w', 'g',
    'ь', 'ы', 'ф', 'ш', 'щ', 'Ж', 'Ф', 'Ю', 'Я',
    // Light CJK (few strokes)
    '一', '二', '三', '十', '人', '口', '日',
    // Medium CJK
    '月', '木', '水', '火', '土', '山', '川', '大', '中',
    '上', '下', '東', '西', '南', '北',
    // Heavy CJK
    '国', '森', '雨', '雪', '風', '海', '龍', '鬱', '魔',
    // Symbols / geometric (heavy)
    '*', '#', 'M', 'W', 'B', '8', '@',
    '※', '★', '☆', '◆', '◇', '●', '○', '■', '□', '▲', '▼',
    // Near-solid fills
    '░', '▒', '▓', '█',
    // Emoji peaks (silhouette only — shader samples alpha)
    '🔥', '✨', '💫', '⭐', '🌊', '💥',
];
const ATLAS_COLS: i32 = 13;

pub struct Renderer {
    gl: WebGl2RenderingContext,
    program: WebGlProgram,
    vao: WebGlVertexArrayObject,
    u_time: WebGlUniformLocation,
    u_resolution: WebGlUniformLocation,
    u_cell_size: WebGlUniformLocation,
    u_mouse: WebGlUniformLocation,
    u_mouse_strength: WebGlUniformLocation,
    u_pulses: WebGlUniformLocation,
    u_pulse_count: WebGlUniformLocation,
    u_trail: WebGlUniformLocation,
    u_trail_count: WebGlUniformLocation,
    u_atlas_cols: WebGlUniformLocation,
    u_char_count: WebGlUniformLocation,
}

impl Renderer {
    pub fn new(canvas: &HtmlCanvasElement, cell_w: f64, cell_h: f64) -> Self {
        let gl = canvas
            .get_context("webgl2")
            .expect("get_context webgl2")
            .expect("webgl2 unavailable")
            .dyn_into::<WebGl2RenderingContext>()
            .expect("cast webgl2");

        let program = link_program(&gl, VERT, FRAG);
        gl.use_program(Some(&program));

        let vao = gl.create_vertex_array().expect("create_vertex_array");
        gl.bind_vertex_array(Some(&vao));

        let vbo = gl.create_buffer().expect("create_buffer");
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));
        let quad: [f32; 12] = [
            -1.0, -1.0,  1.0, -1.0, -1.0,  1.0,
            -1.0,  1.0,  1.0, -1.0,  1.0,  1.0,
        ];
        unsafe {
            let view = js_sys::Float32Array::view(&quad);
            gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }
        let pos_attr = gl.get_attrib_location(&program, "a_pos") as u32;
        gl.enable_vertex_attrib_array(pos_attr);
        gl.vertex_attrib_pointer_with_i32(
            pos_attr,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );

        let atlas = build_atlas(&gl, cell_w, cell_h);
        gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&atlas));
        let u_atlas = gl
            .get_uniform_location(&program, "u_atlas")
            .expect("u_atlas");
        gl.uniform1i(Some(&u_atlas), 0);

        let u = |name: &str| {
            gl.get_uniform_location(&program, name)
                .unwrap_or_else(|| panic!("uniform {name} missing"))
        };

        Self {
            u_time: u("u_time"),
            u_resolution: u("u_resolution"),
            u_cell_size: u("u_cellSize"),
            u_mouse: u("u_mouse"),
            u_mouse_strength: u("u_mouseStrength"),
            u_pulses: u("u_pulses"),
            u_pulse_count: u("u_pulseCount"),
            u_trail: u("u_trail"),
            u_trail_count: u("u_trailCount"),
            u_atlas_cols: u("u_atlasCols"),
            u_char_count: u("u_charCount"),
            program,
            vao,
            gl,
        }
    }

    pub fn set_viewport(&self, w: i32, h: i32) {
        self.gl.viewport(0, 0, w, h);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        t: f32,
        canvas_w: f32,
        canvas_h: f32,
        cell_w: f32,
        cell_h: f32,
        mouse: (f32, f32),
        mouse_strength: f32,
        pulses: &[(f32, f32, f32, f32)],
        trail: &[(f32, f32, f32)],
    ) {
        let gl = &self.gl;
        gl.use_program(Some(&self.program));
        gl.bind_vertex_array(Some(&self.vao));

        gl.uniform1f(Some(&self.u_time), t);
        gl.uniform2f(Some(&self.u_resolution), canvas_w, canvas_h);
        gl.uniform2f(Some(&self.u_cell_size), cell_w, cell_h);
        gl.uniform2f(Some(&self.u_mouse), mouse.0, mouse.1);
        gl.uniform1f(Some(&self.u_mouse_strength), mouse_strength);

        let mut pulse_flat = [0.0f32; 40];
        let pcount = pulses.len().min(10);
        for (i, p) in pulses.iter().take(10).enumerate() {
            pulse_flat[i * 4]     = p.0;
            pulse_flat[i * 4 + 1] = p.1;
            pulse_flat[i * 4 + 2] = p.2;
            pulse_flat[i * 4 + 3] = p.3;
        }
        gl.uniform4fv_with_f32_array(Some(&self.u_pulses), &pulse_flat);
        gl.uniform1i(Some(&self.u_pulse_count), pcount as i32);

        let mut trail_flat = [0.0f32; 48];
        let tcount = trail.len().min(16);
        for (i, p) in trail.iter().take(16).enumerate() {
            trail_flat[i * 3]     = p.0;
            trail_flat[i * 3 + 1] = p.1;
            trail_flat[i * 3 + 2] = p.2;
        }
        gl.uniform3fv_with_f32_array(Some(&self.u_trail), &trail_flat);
        gl.uniform1i(Some(&self.u_trail_count), tcount as i32);

        gl.uniform1i(Some(&self.u_atlas_cols), ATLAS_COLS);
        gl.uniform1i(Some(&self.u_char_count), CHARS.len() as i32);

        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
    }
}

fn compile_shader(gl: &WebGl2RenderingContext, kind: u32, source: &str) -> WebGlShader {
    let shader = gl.create_shader(kind).expect("create_shader");
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    let ok = gl
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false);
    if !ok {
        let log = gl.get_shader_info_log(&shader).unwrap_or_default();
        panic!("shader compile failed: {log}");
    }
    shader
}

fn link_program(gl: &WebGl2RenderingContext, vert: &str, frag: &str) -> WebGlProgram {
    let vs = compile_shader(gl, WebGl2RenderingContext::VERTEX_SHADER, vert);
    let fs = compile_shader(gl, WebGl2RenderingContext::FRAGMENT_SHADER, frag);
    let program = gl.create_program().expect("create_program");
    gl.attach_shader(&program, &vs);
    gl.attach_shader(&program, &fs);
    gl.link_program(&program);
    let ok = gl
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false);
    if !ok {
        let log = gl.get_program_info_log(&program).unwrap_or_default();
        panic!("program link failed: {log}");
    }
    program
}

fn build_atlas(gl: &WebGl2RenderingContext, cell_w: f64, cell_h: f64) -> WebGlTexture {
    let atlas_rows = (CHARS.len() as i32 + ATLAS_COLS - 1) / ATLAS_COLS;

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas: HtmlCanvasElement = document
        .create_element("canvas")
        .unwrap()
        .dyn_into()
        .unwrap();
    canvas.set_width((cell_w * ATLAS_COLS as f64) as u32);
    canvas.set_height((cell_h * atlas_rows as f64) as u32);

    let ctx: CanvasRenderingContext2d = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap();
    ctx.set_font("13px 'Courier New', Courier, monospace");
    ctx.set_text_baseline("alphabetic");
    ctx.set_fill_style_str("#fff");

    let baseline_y_offset = cell_h * 0.82;
    let mut buf = [0u8; 4];
    for (i, ch) in CHARS.iter().enumerate() {
        let col = (i as i32) % ATLAS_COLS;
        let row = (i as i32) / ATLAS_COLS;
        let cx = col as f64 * cell_w;
        let cy = row as f64 * cell_h + baseline_y_offset;
        // max_width horizontally squeezes wide glyphs (CJK, emoji) into the cell.
        let _ = ctx.fill_text_with_max_width(ch.encode_utf8(&mut buf), cx, cy, cell_w);
    }

    let tex = gl.create_texture().expect("create_texture");
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&tex));
    // Flip Y so canvas-row-0 (top) lands at V=1 (texture top).
    gl.pixel_storei(WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL, 1);
    gl.tex_image_2d_with_u32_and_u32_and_html_canvas_element(
        WebGl2RenderingContext::TEXTURE_2D,
        0,
        WebGl2RenderingContext::RGBA as i32,
        WebGl2RenderingContext::RGBA,
        WebGl2RenderingContext::UNSIGNED_BYTE,
        &canvas,
    )
    .expect("tex_image_2d");
    gl.pixel_storei(WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL, 0);

    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_MIN_FILTER,
        WebGl2RenderingContext::LINEAR as i32,
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_MAG_FILTER,
        WebGl2RenderingContext::LINEAR as i32,
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_WRAP_S,
        WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_WRAP_T,
        WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
    );

    tex
}
