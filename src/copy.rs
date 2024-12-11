use anyhow::Result;
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use walkdir::WalkDir;
use crate::cli::TestMode;

pub async fn get_total_size(path: &Path, recursive: bool) -> Result<u64> {
    let mut total_size = 0;
    
    if recursive && path.is_dir() {
        for entry in WalkDir::new(path).min_depth(1) {
            let entry = entry?;
            if entry.path().is_file() {
                total_size += entry.metadata()?.len();
            }
        }
    } else if path.is_file() {
        total_size = path.metadata()?.len();
    }
    
    Ok(total_size)
}

pub struct ProgressCallback<F> {
    callback: F,
    on_new_file: Box<dyn Fn(&str, u64) + Send + Sync>,
}

pub async fn copy_path<F>(
    src: &Path,
    dst: &Path,
    recursive: bool,
    test_mode: TestMode,
    progress_callback: F,
    on_new_file: impl Fn(&str, u64) + Send + Sync + 'static,
) -> Result<()>
where
    F: Fn(u64) + Send + Sync,
{
    let callback = ProgressCallback {
        callback: progress_callback,
        on_new_file: Box::new(on_new_file),
    };

    if src.is_file() {
        copy_file(src, dst, test_mode, &callback).await?;
    } else if recursive && src.is_dir() {
        // 创建目标目录
        if !dst.exists() {
            fs::create_dir_all(dst).await?;
        }

        // 收集所有需要复制的文件
        let mut files_to_copy = Vec::new();
        for entry in WalkDir::new(src).min_depth(1) {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(src)?;
            let target_path = dst.join(relative_path);

            if path.is_dir() {
                fs::create_dir_all(&target_path).await?;
            } else if path.is_file() {
                files_to_copy.push((path.to_path_buf(), target_path));
            }
        }

        // 逐个复制文件
        for (src_path, dst_path) in files_to_copy {
            if let Some(parent) = dst_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).await?;
                }
            }
            copy_file(&src_path, &dst_path, test_mode.clone(), &callback).await?;
        }
    } else {
        anyhow::bail!("Source path is a directory. Use -r flag for recursive copy.");
    }

    Ok(())
}

async fn copy_file<F>(src: &Path, dst: &Path, test_mode: TestMode, callback: &ProgressCallback<F>) -> Result<()>
where
    F: Fn(u64),
{
    let file_size = src.metadata()?.len();
    let file_name = src.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    
    // Update current file information
    (callback.on_new_file)(&file_name, file_size);

    let mut src_file = File::open(src).await?;
    let mut dst_file = File::create(dst).await?;
    
    let mut buffer = vec![0; 1024 * 1024]; // 1MB buffer
    
    match test_mode {
        TestMode::Delay(ms) => {
            loop {
                let n = src_file.read(&mut buffer).await?;
                if n == 0 {
                    break;
                }
                dst_file.write_all(&buffer[..n]).await?;
                (callback.callback)(n as u64);
                tokio::time::sleep(Duration::from_millis(ms)).await;
            }
        },
        TestMode::SpeedLimit(bps) => {
            let chunk_size = bps.min(buffer.len() as u64);
            let mut start_time = Instant::now();
            
            loop {
                let n = src_file.read(&mut buffer[..chunk_size as usize]).await?;
                if n == 0 { break; }
                
                dst_file.write_all(&buffer[..n]).await?;
                
                // 计算应该等待的时间以达到目标速度
                let elapsed = start_time.elapsed();
                let target_duration = Duration::from_secs_f64(n as f64 / bps as f64);
                if elapsed < target_duration {
                    tokio::time::sleep(target_duration - elapsed).await;
                    start_time = Instant::now();
                }
                
                (callback.callback)(n as u64);
            }
        },
        TestMode::None => {
            loop {
                let n = src_file.read(&mut buffer).await?;
                if n == 0 {
                    break;
                }
                dst_file.write_all(&buffer[..n]).await?;
                (callback.callback)(n as u64);
            }
        }
    }
    
    Ok(())
}