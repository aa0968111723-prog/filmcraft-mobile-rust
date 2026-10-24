//! Compile-only check of candidate Android platform APIs.
//! Does not start an Activity, create a decoder, open audio, or build an APK.

#[cfg(target_os = "android")]
pub use android_activity::AndroidApp;
#[cfg(target_os = "android")]
pub use cpal::traits::HostTrait;
#[cfg(target_os = "android")]
pub use jni::JavaVM;
#[cfg(target_os = "android")]
pub use ndk::media::{media_codec::MediaCodec, media_format::MediaFormat};
