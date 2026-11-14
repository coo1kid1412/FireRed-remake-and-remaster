//! Loop the local soundtrack for the current map or battle.
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
    if !p.music || !window.focused {
        if music.current.is_some() {
            music.stop();
        }
        return;
    }
    let map = s.game.map.as_str();
    let theme = if battle.active {
        if battle.battle.is_wild { "wild" } else { "trainer" }
    } else if map == "lab" {
        "lab"
    } else if map.ends_with("_center") && map != "safari_center" {
        "center"
    } else if map == "viridian_forest" {
        "forest"
    } else if map == "route1" || map == "route2" {
        "route1"
    } else if map.starts_with("viridian") || map.starts_with("pewter") || map.starts_with("route2_") {
        "pewter"
    } else {
        "pallet"
    };
    if music.current.as_deref() == Some(theme) {
        return;
    }
    music.stop();
    let tracks = music.base.join("music");
    let name = format!("{theme}.wav");
    let original = tracks.join("letsgo").join(&name);
    let fallback = if theme == "route1" { "route1.wav" } else { "pallet.wav" };
    let path = if original.is_file() {
        Some(original)
    } else if battle.active {
        None
    } else {
        let imported = tracks.join(fallback);
        Some(if imported.is_file() { imported } else { tracks.join("demo").join(fallback) })
    };
    if let Some(path) = path {
        if music.play(&path) {
            info!("Playing music {}", path.display());
        } else {
            warn!("Cannot play music {}", path.display());
        }
    }
    music.current = Some(theme);
}
