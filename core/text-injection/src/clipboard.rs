use std::mem::size_of;
use std::time::Duration;

use async_trait::async_trait;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_CONTROL,
};

use crate::error::InjectionError;
use crate::injector::{InjectorKind, TextInjector};

/// Inject text by setting the clipboard and synthesizing a Ctrl+V keystroke.
#[derive(Debug, Default, Clone, Copy)]
pub struct ClipboardInjector;

impl ClipboardInjector {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TextInjector for ClipboardInjector {
    fn kind(&self) -> InjectorKind {
        InjectorKind::Clipboard
    }

    async fn inject(&self, text: &str) -> Result<(), InjectionError> {
        if text.is_empty() {
            return Ok(());
        }
        let owned = text.to_owned();
        tokio::task::spawn_blocking(move || inject_blocking(&owned))
            .await
            .map_err(|e| InjectionError::Win32(format!("spawn_blocking: {e}")))?
    }
}

fn inject_blocking(text: &str) -> Result<(), InjectionError> {
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|e| InjectionError::Win32(format!("Failed to open clipboard: {e}")))?;

    // 1. Save old clipboard content (ignoring errors if it's empty or not text)
    let old_text = clipboard.get_text().unwrap_or_default();

    // 2. Set new text
    clipboard
        .set_text(text)
        .map_err(|e| InjectionError::Win32(format!("Failed to set clipboard: {e}")))?;

    // 3. Synthesize Ctrl+V
    send_ctrl_v()?;

    // 4. Wait for the host app to process the paste message before restoring.
    // If we restore too fast, the app pastes the old content. We use 250ms because
    // some apps (like Electron or browsers) can be slow to read the clipboard.
    std::thread::sleep(Duration::from_millis(250));

    // 5. Restore old text (if there was any)
    if old_text.is_empty() {
        let _ = clipboard.clear();
    } else {
        let _ = clipboard.set_text(old_text);
    }

    Ok(())
}

fn send_ctrl_v() -> Result<(), InjectionError> {
    let cb_size = i32::try_from(size_of::<INPUT>())
        .map_err(|_| InjectionError::Win32("INPUT size".to_owned()))?;

    // V key is 0x56
    let vk_v = VIRTUAL_KEY(0x56);

    let inputs = [
        // Ctrl Down
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_CONTROL,
                    wScan: 0,
                    dwFlags:
                        windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS::default(),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
        // V Down
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk_v,
                    wScan: 0,
                    dwFlags:
                        windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS::default(),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
        // V Up
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk_v,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
        // Ctrl Up
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_CONTROL,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
    ];

    // SAFETY: inputs is valid and correctly sized
    let dispatched = unsafe { SendInput(&inputs, cb_size) };
    if dispatched != inputs.len() as u32 {
        return Err(InjectionError::HookBlocked {
            requested: inputs.len() as u32,
            dispatched,
        });
    }

    Ok(())
}
