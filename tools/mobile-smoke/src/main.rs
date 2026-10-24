use filmcraft_mobile_core::{PreviewBudget, TICKS_PER_SECOND, Tick, TrimRange, layout_for_width};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let clip = TrimRange::new(
        Tick(TICKS_PER_SECOND),
        Tick(4 * TICKS_PER_SECOND),
        Tick(10 * TICKS_PER_SECOND),
    )?;
    println!("FilmCraft Mobile: host-side core smoke example (no GUI)");
    println!("Phone width 390: {:?}", layout_for_width(390));
    println!("Tablet width 1024: {:?}", layout_for_width(1024));
    println!("Trim duration: {} exact ticks", clip.duration().0);
    println!(
        "64 MiB RGBA cache: {} frames at 1080p",
        PreviewBudget::new(64 * 1024 * 1024).cached_rgba_frames(1920, 1080)?
    );
    Ok(())
}
