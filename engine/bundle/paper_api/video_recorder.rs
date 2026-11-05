// The engine's own recorder: frames read back from the renderer, window or not, go straight to ffmpeg,
// and the sound mix is copied as it's played. `record PATH` and `record-stop` on the control socket, or
// `--record PATH` when the game starts. A video runs at a fixed rate whatever the game's frame rate is:
// a frame is written as many times as the clock says it was on the screen.
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{SyncSender, sync_channel};
use std::sync::{Mutex, OnceLock};
use std::thread::JoinHandle;
use std::time::Instant;

/// The screenshot request's path that means a frame of the recording.
pub const FRAME: &str = "<recording>";
/// Frames waiting for ffmpeg before one's dropped: a dropped frame shows as the next one held longer.
const QUEUE: usize = 8;

struct Sound {
    samples: Vec<i16>,
    channels: u16,
    rate: u32,
}

struct Encoder {
    child: Child,
    frames: SyncSender<Vec<u8>>,
    writer: JoinHandle<()>,
}

struct Recording {
    path: PathBuf,
    fps: u32,
    started: Instant,
    written: u64,
    encoder: Option<Encoder>,
    sound: Sound,
}

static ON: AtomicBool = AtomicBool::new(false);
static CURRENT: Mutex<Option<Recording>> = Mutex::new(None);

/// What a finished recording came to.
#[derive(Clone, Debug)]
pub struct Finished {
    pub path: PathBuf,
    pub frames: u64,
    pub seconds: f64,
    pub sound: bool,
}

fn silent(command: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    command.stdout(Stdio::null()).stderr(Stdio::null())
}

/// The GPU's encoder when it can open one, which leaves the game its frame rate.
fn encoder_args() -> &'static [&'static str] {
    static ARGS: OnceLock<&'static [&'static str]> = OnceLock::new();
    ARGS.get_or_init(|| {
        let nvenc = silent(Command::new("ffmpeg").args(["-hide_banner", "-f", "lavfi", "-i", "color=black:s=256x256:d=0.1", "-c:v", "h264_nvenc", "-f", "null", "-"])).status();
        if nvenc.is_ok_and(|status| status.success()) {
            &["-c:v", "h264_nvenc", "-preset", "p4", "-rc", "vbr", "-cq", "19", "-b:v", "0"]
        } else {
            &["-c:v", "libx264", "-preset", "veryfast", "-crf", "18"]
        }
    })
}

fn video_path(path: &Path) -> PathBuf {
    path.with_extension("video.mp4")
}

fn wav_path(path: &Path) -> PathBuf {
    path.with_extension("sound.wav")
}

/// Starts recording to `path`, at `fps`. Frames begin with the next one drawn.
pub fn start(path: &Path, fps: u32) -> Result<(), String> {
    let mut current = CURRENT.lock().unwrap_or_else(|e| e.into_inner());
    if current.is_some() {
        return Err("a recording is under way".to_owned());
    }
    if !(1..=240).contains(&fps) {
        return Err(format!("{fps} frames a second: 1 to 240"));
    }
    if let Some(folder) = path.parent().filter(|folder| !folder.as_os_str().is_empty()) {
        std::fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    }
    if silent(Command::new("ffmpeg").arg("-version")).status().is_err() {
        return Err("ffmpeg isn't on the PATH".to_owned());
    }
    encoder_args();
    *current = Some(Recording {
        path: path.to_owned(),
        fps,
        started: Instant::now(),
        written: 0,
        encoder: None,
        sound: Sound { samples: Vec::new(), channels: 2, rate: 48_000 },
    });
    ON.store(true, Ordering::Relaxed);
    Ok(())
}

pub fn recording() -> bool {
    ON.load(Ordering::Relaxed)
}

/// Whether the clock has gone past the next frame: the renderer reads one back only then.
pub fn frame_due() -> bool {
    if !recording() {
        return false;
    }
    let current = CURRENT.lock().unwrap_or_else(|e| e.into_inner());
    current.as_ref().is_some_and(|recording| {
        recording.started.elapsed().as_secs_f64() * f64::from(recording.fps) >= recording.written as f64
    })
}

pub fn is_frame(path: &Path) -> bool {
    path.as_os_str() == FRAME
}

fn open_encoder(path: &Path, fps: u32, [width, height]: [u32; 2], bgra: bool) -> Result<Encoder, String> {
    let size = format!("{width}x{height}");
    let rate = fps.to_string();
    let mut child = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-y", "-f", "rawvideo", "-pix_fmt", if bgra { "bgra" } else { "rgba" }, "-s", &size, "-r", &rate, "-i", "-", "-an"])
        // Encoders want even sides.
        .args(["-vf", "crop=trunc(iw/2)*2:trunc(ih/2)*2,format=yuv420p"])
        .args(encoder_args())
        .arg(video_path(path))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("ffmpeg: {e}"))?;
    let mut stdin: ChildStdin = child.stdin.take().ok_or("ffmpeg has no input")?;
    let (frames, queue) = sync_channel::<Vec<u8>>(QUEUE);
    let writer = std::thread::Builder::new()
        .name("horizon-recorder".to_owned())
        .spawn(move || {
            for frame in queue {
                if stdin.write_all(&frame).is_err() {
                    return;
                }
            }
        })
        .map_err(|e| e.to_string())?;
    Ok(Encoder { child, frames, writer })
}

/// A frame read back: the first sets the video's size, and a later one of another size is left out.
pub fn frame(pixels: &[[u8; 4]], width: u32, height: u32, bgra: bool) {
    let mut current = CURRENT.lock().unwrap_or_else(|e| e.into_inner());
    let Some(recording) = current.as_mut() else {
        return;
    };
    if pixels.len() != (width * height) as usize {
        return;
    }
    if recording.encoder.is_none() {
        match open_encoder(&recording.path, recording.fps, [width, height], bgra) {
            Ok(encoder) => {
                recording.encoder = Some(encoder);
                recording.started = Instant::now();
                recording.sound.samples.clear();
            }
            Err(error) => {
                log::warn!("Horizon: the recording can't start: {error}");
                return;
            }
        }
    }
    let wanted = (recording.started.elapsed().as_secs_f64() * f64::from(recording.fps)) as u64 + 1;
    let Some(encoder) = recording.encoder.as_ref() else {
        return;
    };
    let bytes: Vec<u8> = pixels.as_flattened().to_vec();
    while recording.written < wanted {
        if encoder.frames.try_send(bytes.clone()).is_err() {
            break;
        }
        recording.written += 1;
    }
}

/// The sound as it goes to the speakers: what's in the video is what was heard.
pub fn tee(samples: &[f32], channels: usize, rate: u32) {
    if !recording() {
        return;
    }
    let mut current = CURRENT.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(recording) = current.as_mut().filter(|recording| recording.encoder.is_some()) {
        recording.sound.channels = channels as u16;
        recording.sound.rate = rate;
        recording.sound.samples.extend(samples.iter().map(|sample| (sample.clamp(-1.0, 1.0) * 32767.0) as i16));
    }
}

fn write_wav(path: &Path, sound: &Sound) -> std::io::Result<()> {
    let data = sound.samples.len() as u32 * 2;
    let mut out = std::io::BufWriter::new(std::fs::File::create(path)?);
    out.write_all(b"RIFF")?;
    out.write_all(&(36 + data).to_le_bytes())?;
    out.write_all(b"WAVEfmt ")?;
    out.write_all(&16u32.to_le_bytes())?;
    out.write_all(&1u16.to_le_bytes())?;
    out.write_all(&sound.channels.to_le_bytes())?;
    out.write_all(&sound.rate.to_le_bytes())?;
    out.write_all(&(sound.rate * u32::from(sound.channels) * 2).to_le_bytes())?;
    out.write_all(&(sound.channels * 2).to_le_bytes())?;
    out.write_all(&16u16.to_le_bytes())?;
    out.write_all(b"data")?;
    out.write_all(&data.to_le_bytes())?;
    for sample in &sound.samples {
        out.write_all(&sample.to_le_bytes())?;
    }
    out.flush()
}

/// Ends the recording: the video closes, and the sound joins it.
pub fn stop() -> Result<Finished, String> {
    ON.store(false, Ordering::Relaxed);
    let recording = CURRENT.lock().unwrap_or_else(|e| e.into_inner()).take().ok_or("nothing is being recorded")?;
    let Some(encoder) = recording.encoder else {
        return Err("no frame was drawn: nothing was recorded".to_owned());
    };
    let Encoder { mut child, frames, writer } = encoder;
    drop(frames);
    let _ = writer.join();
    let status = child.wait().map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("ffmpeg ended with {status}"));
    }
    let seconds = recording.written as f64 / f64::from(recording.fps);
    let (video, wav) = (video_path(&recording.path), wav_path(&recording.path));
    let sound = !recording.sound.samples.is_empty();
    if sound {
        write_wav(&wav, &recording.sound).map_err(|e| e.to_string())?;
        let joined = silent(Command::new("ffmpeg").args(["-hide_banner", "-loglevel", "error", "-y", "-i"]).arg(&video).arg("-i").arg(&wav).args(["-c:v", "copy", "-c:a", "aac", "-b:a", "192k", "-shortest"]).arg(&recording.path)).status();
        if !joined.is_ok_and(|status| status.success()) {
            return Err("ffmpeg couldn't join the sound".to_owned());
        }
        let _ = std::fs::remove_file(&video);
        let _ = std::fs::remove_file(&wav);
    } else {
        std::fs::rename(&video, &recording.path).map_err(|e| e.to_string())?;
    }
    Ok(Finished { path: recording.path, frames: recording.written, seconds, sound })
}
