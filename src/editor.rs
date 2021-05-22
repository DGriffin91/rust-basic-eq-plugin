use crate::{
    editor_elements::*,
    eq_effect_parameters::{BandKind, BandMode},
    get_coefficients_iir1, get_coefficients_iir2,
};
use imgui::*;
use vst::util::AtomicFloat;

use crate::units::map_to_freq;
use imgui_baseview::{HiDpiMode, ImguiWindow, RenderSettings, Settings};

use crate::eq_effect_parameters::EQEffectParameters;
use crate::parameter::Parameter;

use vst::editor::Editor;

use baseview::{Size, WindowOpenOptions, WindowScalePolicy};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::sync::Arc;

const WINDOW_WIDTH: usize = 1300;
const WINDOW_HEIGHT: usize = 1300;
const WINDOW_WIDTH_F: f32 = WINDOW_WIDTH as f32;
const WINDOW_HEIGHT_F: f32 = WINDOW_HEIGHT as f32;

fn input_float(ui: &Ui, parameter: &Parameter, i: usize) {
    let knob_id = &ImString::new(format!("##{}_{}_KNOB_CONTORL_", parameter.get_name(), i));
    let mut val = parameter.get();

    let nor_val = parameter.get_normalized();

    let speed = (((parameter.transform_func)(nor_val) / (parameter.inv_transform_func)(nor_val))
        .abs()
        * (parameter.max - parameter.min))
        .max(0.00001) as f32;
    let cursor = ui.cursor_pos();
    if Drag::new(knob_id)
        .range(parameter.min..=parameter.max)
        .speed(speed * 0.001)
        .display_format(im_str!(""))
        .build(ui, &mut val)
    {
        //parameter.set(*knob.p_value)
        parameter.set(val)
    }

    let cursor2 = ui.cursor_pos();
    ui.set_cursor_pos(cursor);

    if !ui.is_item_active() || ui.is_mouse_down(MouseButton::Left) {
        ui.text(&ImString::new(format!("{}", parameter.get_display())));
    }
    ui.set_cursor_pos(cursor2);
}

fn popup_select<F: Fn(usize) -> bool>(
    ui: &Ui,
    parameter: &Parameter,
    i: usize,
    button_fn: F,
    qty_of_options: usize,
) {
    let popup_str = &ImString::new(format!("band_{}_popup{}", parameter.get_name(), i));
    if ui.button(
        &ImString::new(format!(
            "{}##_popupbtn{}_{}",
            parameter.get_display(),
            i,
            parameter.get_name()
        )),
        [0.0, 0.0],
    ) {
        ui.open_popup(popup_str);
    }
    ui.popup(popup_str, || {
        for j in 0..qty_of_options {
            if button_fn(j) {
                parameter.set(j as f32);
                ui.close_current_popup();
                break;
            }
        }
    });
}

pub struct EditorState {
    pub params: Arc<EQEffectParameters>,
    pub sample_rate: Arc<AtomicFloat>,
}

pub struct EQPluginEditor {
    pub is_open: bool,
    pub state: Arc<EditorState>,
}

impl Editor for EQPluginEditor {
    fn position(&self) -> (i32, i32) {
        (0, 0)
    }

    fn size(&self) -> (i32, i32) {
        (WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
    }

    fn open(&mut self, parent: *mut ::std::ffi::c_void) -> bool {
        //::log::info!("self.running {}", self.running);
        if self.is_open {
            return false;
        }

        self.is_open = true;

        let settings = Settings {
            window: WindowOpenOptions {
                title: String::from("imgui-baseview demo window"),
                size: Size::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64),
                scale: WindowScalePolicy::SystemScaleFactor,
            },
            clear_color: (0.0, 0.0, 0.0),
            hidpi_mode: HiDpiMode::Default,
            render_settings: RenderSettings::default(),
        };

        ImguiWindow::open_parented(
            &VstParent(parent),
            settings,
            self.state.clone(),
            |ctx: &mut Context, _state: &mut Arc<EditorState>| {
                ctx.fonts().add_font(&[FontSource::TtfData {
                    data: include_bytes!("../FiraCode-Regular.ttf"),
                    size_pixels: 20.0,
                    config: None,
                }]);
            },
            |_run: &mut bool, ui: &Ui, state: &mut Arc<EditorState>| {
                let w = Window::new(im_str!("Example 1: Basic sliders"))
                    .size([WINDOW_WIDTH_F, WINDOW_HEIGHT_F], Condition::Appearing)
                    .position([0.0, 0.0], Condition::Appearing)
                    .draw_background(false)
                    .no_decoration()
                    .movable(false);
                w.build(&ui, || {
                    let graph_width = 1200.0;
                    let graph_height = 500.0;

                    let db_px_step = graph_height / 40.0;

                    let [cx, cy] = ui.cursor_screen_pos();
                    let [mx, my] = ui.io().mouse_pos;
                    let [px, py] = [
                        map_to_freq((mx - cx) / graph_width),
                        -(my - cy - (graph_height * 0.5)) / 10.0,
                    ];
                    let [px, py] = [px.min(20000.0).max(10.0), py.min(24.0).max(-96.0)];

                    ui.get_window_draw_list().add_text(
                        [mx - 40.0, my - 25.0],
                        ui.style_color(StyleColor::Text),
                        &ImString::new(format!("{}hz {}dB", px as i32, py)),
                    );

                    let sample_rate = state.sample_rate.get();

                    let params = &state.params;

                    let mut graph_y_values = vec![0.0f32; graph_width as usize];

                    let mut bandcoeffs_iir2 = Vec::new();
                    let mut bandcoeffs_iir1 = Vec::new();

                    for band in state.params.bands.iter() {
                        //TODO reuse coeffs from DSP

                        let f0 = band.freq.get();
                        let gain = band.db_gain.get();
                        let q_value = band.q_value.get();
                        let fs = sample_rate;
                        bandcoeffs_iir2.push(get_coefficients_iir2(
                            band.get_kind(),
                            f0,
                            gain,
                            q_value,
                            fs,
                        ));
                        bandcoeffs_iir1.push(get_coefficients_iir1(band.get_kind(), f0, gain, fs));
                    }

                    for (i, graph_y) in graph_y_values.iter_mut().enumerate() {
                        let f_hz = map_to_freq((i as f32) / graph_width);
                        for (i, band) in state.params.bands.iter().enumerate() {
                            let iir2mode = band.mode.get().floor() == 1.0;
                            if iir2mode {
                                let y =
                                    bandcoeffs_iir2[i].get_bode_sample(f_hz, sample_rate).norm();
                                *graph_y += -(y.max(0.0).log(10.0) * 20.0) as f32;
                            } else {
                                let y =
                                    bandcoeffs_iir1[i].get_bode_sample(f_hz, sample_rate).norm();
                                *graph_y += -(y.max(0.0).log(10.0) * 20.0) as f32;
                            }

                            //let y = -new_band.get_bode_sample(z).arg().to_degrees() * 0.2;
                            //*graph_y += y as f32;
                        }
                    }

                    draw_eq_graph(
                        ui,
                        im_str!("test"),
                        [graph_width, graph_height],
                        db_px_step,
                        2.0,
                        graph_width as usize,
                        |i| graph_y_values[i],
                    );
                    ui.columns(4, im_str!("cols"), false);
                    for (i, band) in params.bands.iter().enumerate() {
                        popup_select(
                            ui,
                            &band.kind,
                            i,
                            |j| {
                                ui.radio_button_bool(
                                    &ImString::new(format!(
                                        "{}",
                                        BandKind::from_u8(j as u8).to_string()
                                    )),
                                    band.get_kind() as usize == j,
                                )
                            },
                            8,
                        );
                        input_float(&ui, &band.freq, i);
                        input_float(&ui, &band.db_gain, i);
                        input_float(&ui, &band.q_value, i);
                        popup_select(
                            ui,
                            &band.mode,
                            i,
                            |j| {
                                ui.radio_button_bool(
                                    &ImString::new(format!(
                                        "{}",
                                        BandMode::from_u8(j as u8).to_string()
                                    )),
                                    band.get_mode() as usize == j,
                                )
                            },
                            2,
                        );
                        ui.next_column();
                    }
                });
            },
        );

        true
    }

    fn is_open(&mut self) -> bool {
        self.is_open
    }

    fn close(&mut self) {
        self.is_open = false;
    }
}

struct VstParent(*mut ::std::ffi::c_void);

#[cfg(target_os = "macos")]
unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::macos::MacOSHandle;

        RawWindowHandle::MacOS(MacOSHandle {
            ns_view: self.0 as *mut ::std::ffi::c_void,
            ..MacOSHandle::empty()
        })
    }
}

#[cfg(target_os = "windows")]
unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::windows::WindowsHandle;

        RawWindowHandle::Windows(WindowsHandle {
            hwnd: self.0,
            ..WindowsHandle::empty()
        })
    }
}

#[cfg(target_os = "linux")]
unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::unix::XcbHandle;

        RawWindowHandle::Xcb(XcbHandle {
            window: self.0 as u32,
            ..XcbHandle::empty()
        })
    }
}
