//! Android 移植实现:arboard 上游无 Android 后端(平台选择显式排除了 android)。
//! 这里提供一个进程内内存剪贴板 —— 文本 set/get 可回环(grok 内部复制/粘贴可用),
//! 图片/文件/HTML 降级为不可用错误。App 宿主另有系统级剪贴板桥接。
use crate::common::Error;
#[cfg(feature = "image-data")]
use crate::common::ImageData;
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

// 进程内文本剪贴板。
static MEM_TEXT: Mutex<Option<String>> = Mutex::new(None);

pub struct Clipboard {}

impl Clipboard {
    pub fn new() -> Result<Self, Error> {
        Ok(Clipboard {})
    }
}

pub struct Get<'clipboard> {
    _inner: &'clipboard mut Clipboard,
}

impl<'clipboard> Get<'clipboard> {
    pub fn new(clipboard: &'clipboard mut Clipboard) -> Self {
        Get { _inner: clipboard }
    }

    pub fn text(self) -> Result<String, Error> {
        MEM_TEXT
            .lock()
            .ok()
            .and_then(|g| g.clone())
            .ok_or(Error::ContentNotAvailable)
    }

    pub fn html(self) -> Result<String, Error> {
        Err(Error::ContentNotAvailable)
    }

    pub fn file_list(self) -> Result<Vec<PathBuf>, Error> {
        Err(Error::ContentNotAvailable)
    }

    #[cfg(feature = "image-data")]
    pub fn image(self) -> Result<ImageData<'static>, Error> {
        Err(Error::ContentNotAvailable)
    }
}

pub struct Set<'clipboard> {
    _inner: &'clipboard mut Clipboard,
}

impl<'clipboard> Set<'clipboard> {
    pub fn new(clipboard: &'clipboard mut Clipboard) -> Self {
        Set { _inner: clipboard }
    }

    pub fn text(self, text: Cow<'_, str>) -> Result<(), Error> {
        if let Ok(mut g) = MEM_TEXT.lock() {
            *g = Some(text.into_owned());
        }
        Ok(())
    }

    pub fn html(self, _html: Cow<'_, str>, alt_text: Option<Cow<'_, str>>) -> Result<(), Error> {
        // 无原生 HTML 剪贴板:退化为把纯文本替代内容写入。
        if let Some(alt) = alt_text {
            if let Ok(mut g) = MEM_TEXT.lock() {
                *g = Some(alt.into_owned());
            }
        }
        Ok(())
    }

    pub fn file_list(self, _paths: &[impl AsRef<Path>]) -> Result<(), Error> {
        Err(Error::ConversionFailure)
    }

    #[cfg(feature = "image-data")]
    pub fn image(self, _image: ImageData) -> Result<(), Error> {
        Err(Error::ConversionFailure)
    }
}

pub struct Clear<'clipboard> {
    _inner: &'clipboard mut Clipboard,
}

impl<'clipboard> Clear<'clipboard> {
    pub fn new(clipboard: &'clipboard mut Clipboard) -> Self {
        Clear { _inner: clipboard }
    }

    pub fn clear(self) -> Result<(), Error> {
        if let Ok(mut g) = MEM_TEXT.lock() {
            *g = None;
        }
        Ok(())
    }
}
