use std::sync::Mutex;

use retaia_agent::{
    AudioProxyFormat, AudioProxyRequest, CommandOutput, CommandRunner, FfmpegProxyGenerator,
    ProxyGenerationError, ProxyGenerator, VideoProxyRequest,
};

#[derive(Default)]
struct FlowRunner {
    calls: Mutex<Vec<(String, Vec<String>)>>,
}

impl FlowRunner {
    fn calls(&self) -> Vec<(String, Vec<String>)> {
        self.calls.lock().expect("calls").clone()
    }
}

impl CommandRunner for FlowRunner {
    fn run(&self, program: &str, args: &[String]) -> Result<CommandOutput, ProxyGenerationError> {
        self.calls
            .lock()
            .expect("calls")
            .push((program.to_string(), args.to_vec()));
        Ok(CommandOutput {
            status_code: Some(0),
            stderr: String::new(),
        })
    }
}

#[test]
fn e2e_ffmpeg_proxy_generator_flow_builds_video_and_audio_commands_with_configured_binary() {
    let runner = FlowRunner::default();
    let generator = FfmpegProxyGenerator::new("/usr/local/bin/ffmpeg".to_string(), runner);

    generator
        .generate_video_proxy(&VideoProxyRequest {
            input_path: "/tmp/source.mov".to_string(),
            output_path: "/tmp/proxy.mp4".to_string(),
            max_width: 1280,
            max_height: 720,
            video_bitrate_kbps: 3000,
            audio_bitrate_kbps: 128,
        })
        .expect("video generation should succeed");

    generator
        .generate_audio_proxy(&AudioProxyRequest {
            input_path: "/tmp/source.wav".to_string(),
            output_path: "/tmp/proxy.mp3".to_string(),
            format: AudioProxyFormat::Mpeg,
            audio_bitrate_kbps: 192,
            sample_rate_hz: 44100,
        })
        .expect("audio generation should succeed");

    let calls = generator.runner().calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].0, "/usr/local/bin/ffmpeg");
    assert_eq!(calls[1].0, "/usr/local/bin/ffmpeg");
    assert!(calls[0].1.join(" ").contains("-c:v libx264"));
    assert!(calls[1].1.join(" ").contains("-c:a libmp3lame"));
}
