//! The alert when a timer ends: three beeps through Web Audio and a vibration where the browser has
//! one. The audio context is made during the tap that starts a timer, so the browser lets it sound.

use std::cell::RefCell;

use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::AudioContext;

thread_local! {
    static AUDIO: RefCell<Option<AudioContext>> = const { RefCell::new(None) };
}

pub(super) fn unlock() {
    AUDIO.with_borrow_mut(|audio| {
        if audio.is_none() {
            *audio = AudioContext::new().ok();
        }
    });
}

pub(super) fn alert() {
    AUDIO.with_borrow(|audio| audio.as_ref().and_then(beep));
    vibrate();
}

fn beep(context: &AudioContext) -> Option<()> {
    let now = context.current_time();
    for index in 0_u32..3 {
        let start = f64::mul_add(f64::from(index), 0.7, now);
        let oscillator = context.create_oscillator().ok()?;
        let gain = context.create_gain().ok()?;
        oscillator.connect_with_audio_node(&gain).ok()?;
        gain.connect_with_audio_node(&context.destination()).ok()?;
        gain.gain().set_value(0.2);
        oscillator.start_with_when(start).ok()?;
        oscillator.stop_with_when(start + 0.5).ok()?;
    }
    Some(())
}

#[wasm_bindgen(
    inline_js = "export function vibrate() { try { navigator.vibrate([300, 150, 300]); } catch (error) {} }"
)]
extern "C" {
    fn vibrate();
}
