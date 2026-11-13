//! Loop the local exploration WAV for the current map.
use super::*;
use std::path::Path;

pub(super) const SUPPORTED: bool = cfg!(any(windows, target_os = "macos"));

#[cfg(windows)]
#[link(name = "winmm")]
unsafe extern "system" {
    fn PlaySoundW(sound: *const u16, module: *mut std::ffi::c_void, flags: u32) -> i32;
}
#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn firered_music_play(path: *const std::ffi::c_char) -> *mut std::ffi::c_void;
    fn firered_music_stop(handle: *mut std::ffi::c_void);
}

pub(super) struct Music {
    silent: bool,
    base: PathBuf,
    current: Option<&'static str>,
    #[cfg(target_os = "macos")]
    player: Option<std::ptr::NonNull<std::ffi::c_void>>,
}
impl Music {
    pub fn new(base: PathBuf, silent: bool) -> Self {
        Self {
            silent,
            base,
            current: None,
            #[cfg(target_os = "macos")]
            player: None,
        }
    }
    fn stop(&mut self) {
        #[cfg(windows)]
        unsafe {
            PlaySoundW(std::ptr::null(), std::ptr::null_mut(), 0);
        }
        #[cfg(target_os = "macos")]
        if let Some(player) = self.player.take() {
            // The handle came from firered_music_play and is released once.
            unsafe { firered_music_stop(player.as_ptr()) };
        }
        self.current = None;
    }

    #[cfg(windows)]
    fn play(&mut self, path: &Path) -> bool {
        use std::os::windows::ffi::OsStrExt;
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        // ASYNC | NODEFAULT | LOOP | FILENAME.
        unsafe { PlaySoundW(wide.as_ptr(), std::ptr::null_mut(), 0x0002000b) != 0 }
    }

    #[cfg(target_os = "macos")]
    fn play(&mut self, path: &Path) -> bool {
        use std::os::unix::ffi::OsStrExt;
        let Ok(path) = std::ffi::CString::new(path.as_os_str().as_bytes()) else {
            return false;
        };
        // The string lives through the call. AVAudioPlayer owns its URL afterward.
        self.player = std::ptr::NonNull::new(unsafe { firered_music_play(path.as_ptr()) });
        self.player.is_some()
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    fn play(&mut self, _: &Path) -> bool {
        false
    }
}
impl Drop for Music {
    fn drop(&mut self) {
        self.stop();
    }
}
pub(super) fn update(
    s: Res<Simulation>,
    p: Res<Preferences>,
    mut music: NonSendMut<Music>,
    window: Single<&Window, With<PrimaryWindow>>,
    battle: Res<super::battle_screen::Screen>,
) {
    if !SUPPORTED || music.silent {
        return;
    }
    if !p.music || !window.focused || battle.active {
        if music.current.is_some() {
            music.stop();
        }
        return;
    }
    let theme = if s.game.map == "route1" {
        "route1"
    } else {
        "pallet"
    };
    if music.current.as_deref() == Some(theme) {
        return;
    }
    music.stop();
    let tracks = music.base.join("music");
    let name = format!("{theme}.wav");
    let original = tracks.join(&name);
    let path = if original.is_file() {
        original
    } else {
        tracks.join("demo").join(name)
    };
    if music.play(&path) {
        info!("Playing exploration music {}", path.display());
    } else {
        warn!("Cannot play exploration music {}", path.display());
    }
    music.current = Some(theme);
}
