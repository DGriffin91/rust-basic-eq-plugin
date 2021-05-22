#[macro_use]
extern crate vst;

mod editor;
pub mod editor_elements;
mod eq_effect_parameters;
mod parameter;
pub mod units;

mod atomic_bool;

use basic_audio_filters::first_order_iir::IIR1Coefficients;
use basic_audio_filters::first_order_iir::IIR1;
use basic_audio_filters::second_order_iir::IIR2Coefficients;
use basic_audio_filters::second_order_iir::IIR2;

use editor::{EQPluginEditor, EditorState};
use eq_effect_parameters::{BandKind, BandParameters, EQEffectParameters};

use vst::buffer::AudioBuffer;
use vst::editor::Editor;
use vst::plugin::{Category, Info, Plugin, PluginParameters};
use vst::util::AtomicFloat;

use std::sync::Arc;

const FILTER_COUNT: usize = 4;

fn get_coefficients_iir2(
    kind: BandKind,
    f0: f32,
    db_gain: f32,
    q_value: f32,
    fs: f32,
) -> IIR2Coefficients {
    match kind {
        BandKind::Bell => IIR2Coefficients::bell(f0, db_gain, q_value, fs),
        BandKind::LowPass => IIR2Coefficients::lowpass(f0, db_gain, q_value, fs),
        BandKind::HighPass => IIR2Coefficients::highpass(f0, db_gain, q_value, fs),
        BandKind::LowShelf => IIR2Coefficients::lowshelf(f0, db_gain, q_value, fs),
        BandKind::HighShelf => IIR2Coefficients::highshelf(f0, db_gain, q_value, fs),
        BandKind::Notch => IIR2Coefficients::notch(f0, db_gain, q_value, fs),
        BandKind::BandPass => IIR2Coefficients::bandpass(f0, db_gain, q_value, fs),
        BandKind::AllPass => IIR2Coefficients::allpass(f0, db_gain, q_value, fs),
    }
}

fn get_coefficients_iir1(kind: BandKind, f0: f32, db_gain: f32, fs: f32) -> IIR1Coefficients {
    match kind {
        BandKind::LowPass => IIR1Coefficients::lowpass(f0, db_gain, fs),
        BandKind::HighPass => IIR1Coefficients::highpass(f0, db_gain, fs),
        BandKind::LowShelf => IIR1Coefficients::lowshelf(f0, db_gain, fs),
        BandKind::HighShelf => IIR1Coefficients::highshelf(f0, db_gain, fs),
        BandKind::AllPass => IIR1Coefficients::allpass(f0, db_gain, fs),
        _ => IIR1Coefficients::allpass(f0, db_gain, fs),
    }
}

pub struct EditorFilterData {
    pub params: Arc<BandParameters>,
}

struct EQPlugin {
    params: Arc<EQEffectParameters>,
    editor: Option<EQPluginEditor>,
    filter_iir2_l: Vec<IIR2>,
    filter_iir2_r: Vec<IIR2>,
    filter_iir1_l: Vec<IIR1>,
    filter_iir1_r: Vec<IIR1>,
    time: Arc<AtomicFloat>,
    sample_rate: Arc<AtomicFloat>,
    block_size: i64,
}

impl Default for EQPlugin {
    fn default() -> Self {
        let params = Arc::new(EQEffectParameters::default());
        let time = Arc::new(AtomicFloat::new(0.0));
        let sample_rate = Arc::new(AtomicFloat::new(48000.0));

        let coeffs = IIR2Coefficients::bell(1000.0, 0.0, 1.0, 48000.0);

        let filter_iir2_l = (0..FILTER_COUNT)
            .map(|_| IIR2::from(coeffs))
            .collect::<Vec<IIR2>>();
        let filter_iir2_r = (0..FILTER_COUNT)
            .map(|_| IIR2::from(coeffs))
            .collect::<Vec<IIR2>>();

        let coeffs = IIR1Coefficients::lowpass(1000.0, 0.0, 48000.0);

        let filter_iir1_l = (0..FILTER_COUNT)
            .map(|_| IIR1::from(coeffs))
            .collect::<Vec<IIR1>>();
        let filter_iir1_r = (0..FILTER_COUNT)
            .map(|_| IIR1::from(coeffs))
            .collect::<Vec<IIR1>>();

        Self {
            params: params.clone(),
            sample_rate: sample_rate.clone(),
            block_size: 128,
            time: time.clone(),
            editor: Some(EQPluginEditor {
                is_open: false,
                state: Arc::new(EditorState {
                    params: params.clone(),
                    sample_rate: sample_rate.clone(),
                }),
            }),
            filter_iir2_l,
            filter_iir2_r,
            filter_iir1_l,
            filter_iir1_r,
        }
    }
}

fn setup_logging() {
    let log_folder = ::dirs::home_dir().unwrap().join("tmp");

    let _ = ::std::fs::create_dir(log_folder.clone());

    let log_file = ::std::fs::File::create(log_folder.join("IMGUIBaseviewEQ.log")).unwrap();

    let log_config = ::simplelog::ConfigBuilder::new()
        .set_time_to_local(true)
        .build();

    let _ = ::simplelog::WriteLogger::init(simplelog::LevelFilter::max(), log_config, log_file);

    ::log_panics::init();

    ::log::info!("init");
}

impl Plugin for EQPlugin {
    fn get_info(&self) -> Info {
        Info {
            name: "Basic IMGUI EQ in Rust 0.1".to_string(),
            vendor: "DGriffin".to_string(),
            unique_id: 237955111,
            version: 2,
            inputs: 2,
            outputs: 2,
            // This `parameters` bit is important; without it, none of our
            // parameters will be shown!
            parameters: self.params.len() as i32,
            category: Category::Effect,
            ..Default::default()
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate.set(rate);
    }

    fn set_block_size(&mut self, block_size: i64) {
        self.block_size = block_size;
    }

    fn init(&mut self) {
        setup_logging();
        //setup_logger();
    }

    fn get_editor(&mut self) -> Option<Box<dyn Editor>> {
        if let Some(editor) = self.editor.take() {
            Some(Box::new(editor) as Box<dyn Editor>)
        } else {
            None
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        println!("{}", vec![1.0][0]);
        self.time
            .set(self.time.get() + (1.0 / self.sample_rate.get()) * self.block_size as f32);
        let sample_rate = self.sample_rate.get();

        let (inputs, outputs) = buffer.split();
        let (inputs_left, inputs_right) = inputs.split_at(1);
        let (mut outputs_left, mut outputs_right) = outputs.split_at_mut(1);

        let inputs_stereo = inputs_left[0].iter().zip(inputs_right[0].iter());
        let outputs_stereo = outputs_left[0].iter_mut().zip(outputs_right[0].iter_mut());

        for (input_pair, output_pair) in inputs_stereo.zip(outputs_stereo) {
            for (i, band) in self.params.bands.iter().enumerate() {
                if !band.dsp_update() {
                    continue;
                }
                let f0 = band.freq.get();
                let db_gain = band.db_gain.get();
                let q_value = band.q_value.get();
                let iir2mode = band.mode.get().floor() == 1.0;
                let fs = sample_rate;
                if iir2mode {
                    let coeffs = get_coefficients_iir2(band.get_kind(), f0, db_gain, q_value, fs);
                    self.filter_iir2_l[i].update(coeffs);
                    self.filter_iir2_r[i].update(coeffs);
                } else {
                    let coeffs = get_coefficients_iir1(band.get_kind(), f0, db_gain, fs);
                    self.filter_iir1_l[i].update(coeffs);
                    self.filter_iir1_r[i].update(coeffs);
                }
            }

            let (output_l, output_r) = output_pair;
            *output_l = *input_pair.0;
            *output_r = *input_pair.1;

            for i in 0..self.filter_iir2_l.len() {
                let iir2mode = self.params.bands[i].mode.get().floor() == 1.0;
                if iir2mode {
                    *output_l = self.filter_iir2_l[i].process(*output_l);
                    *output_r = self.filter_iir2_r[i].process(*output_r);
                } else {
                    *output_l = self.filter_iir1_l[i].process(*output_l);
                    *output_r = self.filter_iir1_r[i].process(*output_r);
                }
            }
        }
    }

    // Return the parameter object. This method can be omitted if the
    // plugin has no parameters.
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

impl PluginParameters for EQEffectParameters {
    // the `get_parameter` function reads the value of a parameter.
    fn get_parameter(&self, index: i32) -> f32 {
        if (index as usize) < self.len() {
            self[index as usize].get_normalized() as f32
        } else {
            0.0
        }
    }

    // the `set_parameter` function sets the value of a parameter.
    fn set_parameter(&self, index: i32, val: f32) {
        #[allow(clippy::single_match)]
        if (index as usize) < self.len() {
            self[index as usize].set_normalized(val);
        }
    }

    // This is what will display underneath our control.  We can
    // format it into a string that makes the most since.

    fn get_parameter_text(&self, index: i32) -> String {
        if (index as usize) < self.len() {
            self[index as usize].get_display()
        } else {
            "".to_string()
        }
    }

    // This shows the control's name.
    fn get_parameter_name(&self, index: i32) -> String {
        if (index as usize) < self.len() {
            self[index as usize].get_name()
        } else {
            "".to_string()
        }
    }
}

plugin_main!(EQPlugin);
