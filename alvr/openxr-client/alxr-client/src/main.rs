#![cfg_attr(target_vendor = "uwp", windows_subsystem = "windows")]

use alxr_common::{
    alxr_destroy, alxr_init, alxr_is_session_running, alxr_process_frame, battery_send,
    init_connections, input_send, path_string_to_hash, request_idr, set_waiting_next_idr, shutdown,
    time_sync_send, video_error_report_send, views_config_send, ALXRDecoderType, ALXRGraphicsApi,
    ALXRRustCtx, ALXRSystemProperties, APP_CONFIG,
};
use std::{thread, time};

const SLEEP_TIME: time::Duration = time::Duration::from_millis(250);

#[cfg(any(target_vendor = "uwp", target_os = "windows"))]
const DEFAULT_DECODER_TYPE: ALXRDecoderType = ALXRDecoderType::D311VA;

#[cfg(not(any(target_vendor = "uwp", target_os = "windows")))]
const DEFAULT_DECODER_TYPE: ALXRDecoderType = ALXRDecoderType::VAAPI;

#[cfg(target_vendor = "uwp")]
const DEFAULT_GRAPHICS_API: ALXRGraphicsApi = ALXRGraphicsApi::D3D12;

#[cfg(not(target_vendor = "uwp"))]
const DEFAULT_GRAPHICS_API: ALXRGraphicsApi = ALXRGraphicsApi::Auto;

#[cfg(not(target_os = "android"))]
fn main() {
    println!("{:?}", *APP_CONFIG);
    let selected_api = APP_CONFIG.graphics_api.unwrap_or(DEFAULT_GRAPHICS_API);
    let selected_decoder = APP_CONFIG.decoder_type.unwrap_or(DEFAULT_DECODER_TYPE);
    unsafe {
        loop {
            let ctx = ALXRRustCtx {
                inputSend: Some(input_send),
                viewsConfigSend: Some(views_config_send),
                pathStringToHash: Some(path_string_to_hash),
                timeSyncSend: Some(time_sync_send),
                videoErrorReportSend: Some(video_error_report_send),
                batterySend: Some(battery_send),
                setWaitingNextIDR: Some(set_waiting_next_idr),
                requestIDR: Some(request_idr),
                graphicsApi: selected_api,
                decoderType: selected_decoder,
                verbose: APP_CONFIG.verbose,
                disableLinearizeSrgb: APP_CONFIG.no_linearize_srgb,
            };
            let mut sys_properties = ALXRSystemProperties::new();
            if !alxr_init(&ctx, &mut sys_properties) {
                break;
            }
            init_connections(&sys_properties);

            let mut request_restart = false;
            loop {
                let mut exit_render_loop = false;
                alxr_process_frame(&mut exit_render_loop, &mut request_restart);
                if exit_render_loop {
                    break;
                }
                if !alxr_is_session_running() {
                    // Throttle loop since xrWaitFrame won't be called.
                    thread::sleep(SLEEP_TIME);
                }
            }

            shutdown();
            alxr_destroy();

            if !request_restart {
                break;
            }
        }
    }
    println!("successfully shutdown.");
}
