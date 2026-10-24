use filmcraft_mobile_core::{PreviewBudget, TICKS_PER_SECOND, Tick, TrimRange, layout_for_width};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let clip = TrimRange::new(
        Tick(TICKS_PER_SECOND),
        Tick(4 * TICKS_PER_SECOND),
        Tick(10 * TICKS_PER_SECOND),
    )?;
    println!("影片工坊行動版：主機端核心基本驗證範例（無圖形介面）");
    println!("手機寬度 390：{:?}", layout_for_width(390));
    println!("平板寬度 1024：{:?}", layout_for_width(1024));
    println!("裁切長度：{} 個精確時間刻度", clip.duration().0);
    println!(
        "64 MiB 色彩預覽快取：可容納 {} 格 1080p 影格",
        PreviewBudget::new(64 * 1024 * 1024).cached_rgba_frames(1920, 1080)?
    );
    Ok(())
}
