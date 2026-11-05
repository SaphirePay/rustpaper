# Rows that change text worlds_engine.py writes, so they have to apply after it: tables apply by file name.
EDITS = [
    # A plugin hearing chat holds the line back; the decorator still has its say on it.
    ('horizon-client/src/lan/hub/chat.rs', '''            crate::paper_hub::chat_heard(uuid, payload, Box::new((message, bound, line)));
            return;
''', '''            let mut message = message;
            if let Some(decorated) = crate::paper_hub::decorate(message.sender, &message.content) {
                message.unsigned_content = Some(decorated);
            }
            crate::paper_hub::chat_heard(uuid, payload, Box::new((message, bound, line)));
            return;
'''),
    # The world's table: a pack's enchantment isn't in the game's own.
    ('horizon-client/src/game/commands/effects.rs', '''        let (name, curse) = self
            .res
            .stations()
            .enchantments
            .full_name(&id, level.max(0) as u32);
''', '''        let (name, curse) = self
            .stations()
            .enchantments
            .full_name(&id, level.max(0) as u32);
'''),
    # ---- The engine's recorder.
    ('horizon-client-render/src/gfx.rs', '''    pub fn render(&mut self, frame: &Frame) -> Result<bool> {
''', '''    pub fn render(&mut self, frame: &Frame) -> Result<bool> {
        if self.screenshot.is_none() && crate::video_recorder::frame_due() {
            self.screenshot = Some(ScreenshotRequest { path: crate::video_recorder::FRAME.into(), size: None, scaled: false });
        }
'''),
    ('horizon-client-render/src/gfx.rs', '''                self.screenshot_result = Some(
                    self.write_screenshot(&request, &buffer).map_err(|e| e.to_string()),
                );
            }
        }
        let present = Instant::now();
        if let Some(surface_frame) = surface_frame {
            if self.config.present_mode == wgpu::PresentMode::Fifo {
''', '''                if crate::video_recorder::is_frame(&request.path) {
                    let bgra = matches!(self.config.format.remove_srgb_suffix(), wgpu::TextureFormat::Bgra8Unorm);
                    match self.read_frame(&buffer) {
                        Ok(pixels) => crate::video_recorder::frame(&pixels, self.config.width, self.config.height, bgra),
                        Err(e) => log::warn!("Horizon: a recorded frame: {e}"),
                    }
                } else {
                    self.screenshot_result = Some(
                        self.write_screenshot(&request, &buffer).map_err(|e| e.to_string()),
                    );
                }
            }
        }
        let present = Instant::now();
        if let Some(surface_frame) = surface_frame {
            if self.config.present_mode == wgpu::PresentMode::Fifo {
'''),
    ('horizon-client-render/src/audio.rs', '''            if let Ok(mut m) = mixer.lock() {
                mix(&mut m, &mut scratch, channels, rate);
            } else {
                scratch.fill(0.0);
            }
            for (out, s) in data.iter_mut().zip(&scratch) {
''', '''            if let Ok(mut m) = mixer.lock() {
                mix(&mut m, &mut scratch, channels, rate);
            } else {
                scratch.fill(0.0);
            }
            crate::video_recorder::tee(&scratch, channels, rate);
            for (out, s) in data.iter_mut().zip(&scratch) {
'''),
    ('horizon-client/src/game/control.rs', '''            "screenshot" => {
                let renderer = self.renderer.as_mut().ok_or("no renderer yet")?;
''', '''            "record" => {
                let rest = line.trim_start().strip_prefix("record").unwrap_or("").trim();
                let (path, fps) = match rest.rsplit_once(" --fps ") {
                    Some((path, fps)) => (path.trim(), fps.trim().parse().map_err(|_| "--fps takes a number")?),
                    None => (rest, 60),
                };
                if path.is_empty() {
                    return Err("record PATH [--fps N]".into());
                }
                if self.renderer.is_none() {
                    return Err("no renderer: --headless draws nothing, so use --offscreen".into());
                }
                horizon_client_render::video_recorder::start(std::path::Path::new(path), fps)?;
                Ok(Some(json!({ "ok": true, "recording": path, "fps": fps })))
            }
            "record-stop" => {
                let done = horizon_client_render::video_recorder::stop()?;
                Ok(Some(json!({ "ok": true, "path": done.path, "frames": done.frames, "seconds": done.seconds, "sound": done.sound })))
            }
            "screenshot" => {
                let renderer = self.renderer.as_mut().ok_or("no renderer yet")?;
'''),
    ('horizon-client/src/game/control.rs', '''    "screenshot PATH                write the next frame as PNG",
''', '''    "screenshot PATH                write the next frame as PNG",
    "record PATH [--fps N]          record video and sound, with or without a window, until record-stop",
    "record-stop                    finish the recording and join its sound",
'''),
    ('horizon-client-render/src/gfx.rs', '''                    self.screenshot_result = Some(
                        self
                            .write_screenshot(&request, &buffer)
                            .map_err(|e| e.to_string()),
                    );
''', '''                    if crate::video_recorder::is_frame(&request.path) {
                        let bgra = matches!(self.config.format.remove_srgb_suffix(), wgpu::TextureFormat::Bgra8Unorm);
                        if let Ok(pixels) = self.read_frame(&buffer) {
                            crate::video_recorder::frame(&pixels, self.config.width, self.config.height, bgra);
                        }
                    } else {
                        self.screenshot_result = Some(
                            self
                                .write_screenshot(&request, &buffer)
                                .map_err(|e| e.to_string()),
                        );
                    }
'''),
    # With no output device, nothing pulls the mix: a recording does, so it has sound with no speaker.
    ('horizon-client-render/src/audio.rs', '''    pub fn silent(root: &Path) -> Self {
''', '''    /// For a recording: with no output, the mixer runs on a thread of its own until the recording ends.
    pub fn mix_for_recording(&mut self) {
        if self._stream.is_some() {
            return;
        }
        self.engine.loaded = true;
        self.backend.live = true;
        let mixer = self.backend.mixer.clone();
        let _ = std::thread::Builder::new().name("horizon-recorder-mix".to_owned()).spawn(move || {
            const RATE: u32 = 48_000;
            const CHANNELS: usize = 2;
            let began = std::time::Instant::now();
            let mut pulled = 0u64;
            let mut samples = vec![0.0f32; RATE as usize / 50 * CHANNELS];
            while crate::video_recorder::recording() {
                let due = (began.elapsed().as_secs_f64() * f64::from(RATE)) as u64;
                if due >= pulled + RATE as u64 / 50 {
                    if let Ok(mut held) = mixer.lock() {
                        mix(&mut held, &mut samples, CHANNELS, RATE);
                    } else {
                        samples.fill(0.0);
                    }
                    crate::video_recorder::tee(&samples, CHANNELS, RATE);
                    pulled += RATE as u64 / 50;
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
            }
        });
    }
    /// The recording's over: with no output again, nothing plays.
    pub fn recording_over(&mut self) {
        if self._stream.is_none() {
            self.engine.loaded = false;
            self.backend.live = false;
        }
    }
    pub fn silent(root: &Path) -> Self {
'''),
    ('horizon-client/src/game/control.rs', '''                horizon_client_render::video_recorder::start(std::path::Path::new(path), fps)?;
''', '''                horizon_client_render::video_recorder::start(std::path::Path::new(path), fps)?;
                self.audio.mix_for_recording();
'''),
    ('horizon-client/src/game/control.rs', '''                let done = horizon_client_render::video_recorder::stop()?;
''', '''                let done = horizon_client_render::video_recorder::stop();
                self.audio.recording_over();
                let done = done?;
'''),
    # A pack's enchantment names itself in its file, with no key in the language.
    ('horizon-items/src/enchant.rs', '''                description: lang
                    .get(key)
                    .cloned()
                    .unwrap_or_else(|| id.trim_start_matches("minecraft:").replace('_', " ")),
''', '''                description: v["description"]["text"].as_str().or_else(|| v["description"].as_str()).map(str::to_owned).unwrap_or_else(|| lang
                    .get(key)
                    .cloned()
                    .unwrap_or_else(|| id.trim_start_matches("minecraft:").replace('_', " "))),
'''),
    # `--record PATH`, with a window or `--offscreen`, from the first frame.
    ('horizon-client/src/game.rs', '''                "--offscreen" => o.offscreen = true,
''', '''                "--offscreen" => o.offscreen = true,
                "--record" => o.record = args.next().map(std::path::PathBuf::from),
                "--fps" => o.record_fps = args.next().and_then(|fps| fps.parse().ok()).unwrap_or(60),
'''),
    ('horizon-client/src/game.rs', '''    offscreen: bool,
    /// `--headless` (#4951)''', '''    offscreen: bool,
    /// `--record PATH`: the engine's video of the whole run.
    record: Option<std::path::PathBuf>,
    /// `--fps N`: its frame rate.
    record_fps: u32,
    /// `--headless` (#4951)'''),
    ('horizon-client/src/game.rs', '''            offscreen: false,
''', '''            offscreen: false,
            record: None,
            record_fps: 60,
'''),
    ('horizon-client/src/game.rs', '''        if item == "honeycomb" {
            let registry = self.pack.assets.registry;
            let old = registry.state(clicked).cloned();
''', '''        if item == "honeycomb" {
            let registry = self.pack.assets.registry;
            let old = registry.state(clicked).cloned();
'''),
    # /summon with data says why, as the one without does.
    ('horizon-client/src/game/commands.rs', '''                .summon_with_nbt(&format!("minecraft:{short}"), pos, &nbt)?
''', '''                .summon_with_nbt_why(&format!("minecraft:{short}"), pos, &nbt)?
'''),
    ('horizon-client/src/game/commands/data.rs', '''    pub(super) fn summon_with_nbt(
''', '''    pub(super) fn summon_with_nbt_why(&mut self, kind: &str, pos: [f64; 3], nbt: &str) -> R<Option<Target>> {
        horizon_client::mobs::ee::why("COMMAND");
        self.summon_with_nbt(kind, pos, nbt)
    }

    pub(super) fn summon_with_nbt(
'''),
    # ServerGamePacketListenerImpl.handleUseItemOn takes the block the client clicked: the server doesn't look again for an entity in the way.
    ('horizon-client/src/game.rs', '''        let block_hit = if matches!(guest, Some(GuestUse::Item(_)))
            || self.entity_targeted()
        {
''', '''        let block_hit = if matches!(guest, Some(GuestUse::Item(_)))
            || (!on_block && self.entity_targeted())
        {
'''),
]
