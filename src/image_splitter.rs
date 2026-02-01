use image::{DynamicImage, ImageReader};
use std::path::{Path, PathBuf};

/// 分割配置
#[derive(Clone, Debug)]
pub struct SplitConfig {
    pub rows: usize,
    pub cols: usize,
    pub h_lines: Vec<f32>, // 水平分割线位置 (0.0 - 1.0)
    pub v_lines: Vec<f32>, // 垂直分割线位置 (0.0 - 1.0)
}

impl Default for SplitConfig {
    fn default() -> Self {
        Self {
            rows: 1,
            cols: 1,
            h_lines: vec![],
            v_lines: vec![],
        }
    }
}

impl SplitConfig {
    /// 创建默认的平均分割配置
    pub fn new(rows: usize, cols: usize) -> Self {
        let mut config = Self {
            rows,
            cols,
            h_lines: vec![],
            v_lines: vec![],
        };
        config.reset_to_default();
        config
    }

    /// 重置为平均分割
    pub fn reset_to_default(&mut self) {
        self.h_lines = (1..self.rows)
            .map(|i| i as f32 / self.rows as f32)
            .collect();
        self.v_lines = (1..self.cols)
            .map(|i| i as f32 / self.cols as f32)
            .collect();
    }

    /// 验证配置是否有效
    pub fn is_valid(&self) -> bool {
        self.h_lines.len() == self.rows.saturating_sub(1)
            && self.v_lines.len() == self.cols.saturating_sub(1)
    }
}

/// 图片分割器
pub struct ImageSplitter;

impl ImageSplitter {
    /// 打开图片
    pub fn open_image<P: AsRef<Path>>(path: P) -> anyhow::Result<DynamicImage> {
        let img = ImageReader::open(path)?.decode()?;
        Ok(img)
    }

    /// 分割图片
    pub fn split_image(
        img: &DynamicImage,
        config: &SplitConfig,
    ) -> anyhow::Result<Vec<Vec<DynamicImage>>> {
        let (width, height) = (img.width(), img.height());

        // 计算分割边界（像素）- 使用截断方式与 Python 版本保持一致
        let h_positions: Vec<u32> = std::iter::once(0)
            .chain(config.h_lines.iter().map(|&p| (height as f32 * p) as u32))
            .chain(std::iter::once(height))
            .collect();

        let v_positions: Vec<u32> = std::iter::once(0)
            .chain(config.v_lines.iter().map(|&p| (width as f32 * p) as u32))
            .chain(std::iter::once(width))
            .collect();

        // 使用实际的线条数量来计算行列数（而不是依赖 config.rows/cols）
        let actual_rows = config.h_lines.len() + 1;
        let actual_cols = config.v_lines.len() + 1;

        let mut result = Vec::with_capacity(actual_rows);

        for row in 0..actual_rows {
            let mut row_images = Vec::with_capacity(actual_cols);
            let upper = h_positions[row];
            let lower = h_positions[row + 1];

            for col in 0..actual_cols {
                let left = v_positions[col];
                let right = v_positions[col + 1];

                // 使用 crop_imm 代替 crop（不需要可变引用）
                let cropped = img.crop_imm(left, upper, right - left, lower - upper);
                row_images.push(cropped);
            }
            result.push(row_images);
        }

        Ok(result)
    }

    /// 批量处理图片
    pub fn batch_process(
        image_paths: &[PathBuf],
        global_config: &SplitConfig,
        overrides: &std::collections::HashMap<usize, SplitConfig>,
        output_dir: &Path,
        progress_callback: impl Fn(usize, usize) + Sync,
    ) -> anyhow::Result<(usize, usize)> {
        use rayon::prelude::*;
        use std::fs;

        fs::create_dir_all(output_dir)?;

        let total = image_paths.len();
        let processed = std::sync::atomic::AtomicUsize::new(0);
        let failed = std::sync::atomic::AtomicUsize::new(0);

        image_paths.par_iter().enumerate().for_each(|(idx, path)| {
            let config = overrides.get(&idx).unwrap_or(global_config);
            let result = Self::process_single_image(path, config, output_dir);

            if result.is_ok() {
                processed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            } else {
                failed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                eprintln!("处理失败 {:?}: {:?}", path, result.err());
            }

            progress_callback(idx + 1, total);
        });

        Ok((processed.load(std::sync::atomic::Ordering::Relaxed),
            failed.load(std::sync::atomic::Ordering::Relaxed)))
    }

    fn process_single_image(
        path: &Path,
        config: &SplitConfig,
        output_dir: &Path,
    ) -> anyhow::Result<()> {
        let img = Self::open_image(path)?;
        let parts = Self::split_image(&img, config)?;

        let base_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("image");

        for (row_idx, row) in parts.iter().enumerate() {
            for (col_idx, part) in row.iter().enumerate() {
                let output_name = format!("{}_{}_{}.jpg", base_name, row_idx + 1, col_idx + 1);
                let output_path = output_dir.join(output_name);

                part.save_with_format(&output_path, image::ImageFormat::Jpeg)?;
            }
        }

        Ok(())
    }
}
