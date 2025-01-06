mod cli;
mod copy;
mod progress;

use anyhow::Result;
use parking_lot::Mutex;
use progress::CopyProgress;
use std::sync::Arc;
use tokio::signal::ctrl_c;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::parse_args();
    let test_mode = args.get_test_mode();

    // Calculate total size
    let total_size = copy::get_total_size(&args.source, args.recursive).await?;
    let progress = Arc::new(Mutex::new(CopyProgress::new(total_size)?));

    // Set initial file/directory name
    let display_name = args
        .source
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    progress.lock().set_current_file(&display_name, total_size);

    // Create clones for callbacks with explicit type annotations
    let progress_for_inc: Arc<Mutex<CopyProgress>> = Arc::clone(&progress);
    let progress_for_file: Arc<Mutex<CopyProgress>> = Arc::clone(&progress);

    // 修改信号处理逻辑
    let progress_for_signal = Arc::clone(&progress);
    tokio::spawn(async move {
        if let Ok(()) = ctrl_c().await {
            let _ = progress_for_signal.lock().finish();
            std::process::exit(0);
        }
    });

    // Start the copy operation
    let result = copy::copy_path(
        &args.source,
        &args.destination,
        args.recursive,
        args.preserve,
        test_mode,
        move |n| progress_for_inc.lock().inc_current(n),
        move |name, size| progress_for_file.lock().set_current_file(name, size),
    )
    .await;

    // 确保在完成或出错时正确清理
    let mut progress = progress.lock();
    if let Err(e) = result {
        progress.finish()?;
        return Err(e);
    }
    progress.finish()?;

    // 给用户一些时间看到完成状态
    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok(())
}
