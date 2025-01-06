mod cli;
mod copy;
mod progress;

use anyhow::Result;
use parking_lot::Mutex;
use progress::CopyProgress;
use std::io::{self, Write};
use std::sync::Arc;
use tokio::signal::ctrl_c;
use tokio::time::Duration;

async fn confirm_overwrite(files: &[copy::FileToOverwrite]) -> Result<bool> {
    println!("\nThe following items will be overwritten:");
    for file in files {
        println!(
            "  {} {}",
            if file.is_dir { "DIR:" } else { "FILE:" },
            file.path.display()
        );
    }

    print!("\nDo you want to proceed? [y/N] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_lowercase() == "y")
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::parse_args();
    let test_mode = args.get_test_mode();

    // 如果指定了force，检查将被覆盖的文件
    if args.force {
        let files_to_overwrite =
            copy::check_overwrites(&args.source, &args.destination, args.recursive, &args).await?;

        // 如果有文件要被覆盖，且需要确认
        if !files_to_overwrite.is_empty() && args.should_prompt_for_overwrite() {
            if !confirm_overwrite(&files_to_overwrite).await? {
                println!("Operation cancelled.");
                return Ok(());
            }
        }
    }

    // Calculate total size
    let total_size = copy::get_total_size(&args.source, args.recursive, &args).await?;
    let progress = Arc::new(Mutex::new(CopyProgress::new(total_size)?));

    // Set initial file/directory name
    let display_name = args
        .source
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    progress.lock().set_current_file(&display_name, total_size);

    // Create clones for callbacks
    let progress_for_inc = Arc::clone(&progress);
    let progress_for_file = Arc::clone(&progress);

    // 修改信号处理逻辑
    let progress_for_signal = Arc::clone(&progress);
    tokio::spawn(async move {
        if let Ok(()) = ctrl_c().await {
            let _ = progress_for_signal.lock().finish();
            std::process::exit(0);
        }
    });

    // Start the copy operation with exclude patterns
    let result = copy::copy_path(
        &args.source,
        &args.destination,
        args.recursive,
        args.preserve,
        test_mode,
        &args,
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
