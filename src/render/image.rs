use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use headless_chrome::{Browser, LaunchOptions, protocol::cdp::Page};
use image::{
    ImageFormat,
    imageops::{BiLevel, dither},
    load_from_memory_with_format,
};
use log::debug;
use tracing::info;

use crate::render::template::basic_template;

#[derive(Debug, Clone)]
pub struct RenderedImage {
    pub png_path: PathBuf,
    pub bmp_path: PathBuf,
}

impl RenderedImage {
    pub fn default() -> Self {
        let id = uuid::Uuid::new_v4();
        let generated_dir = Path::new("assets/images/generated");
        let png_path = generated_dir.join(id.to_string()).with_extension("png");
        let bmp_path = generated_dir.join(id.to_string()).with_extension("bmp");
        Self { png_path, bmp_path }
    }

    pub async fn render(&self, url: &str) -> Result<(), anyhow::Error> {
        // self.render_images(url).await?;
        info!(url);
        self.render_html().await?;
        Ok(())
    }

    async fn render_html(&self) -> Result<(), anyhow::Error> {
        let rendered = basic_template()?;
        println!("{}", rendered);
        let base = OsStr::new("--hide-scrollbars");
        let args = vec![base];

        // Create browser with custom window size
        let launch_options = LaunchOptions::default_builder()
            .window_size(Some((800, 480)))
            .headless(true)
            .sandbox(false)
            .args(args)
            .build()?;

        debug!("starting browser");
        let browser = Browser::new(launch_options);
        let browser = match browser {
            Err(err) => panic!("{}", err),
            Ok(browser) => browser,
        };

        // Navigate to the URL and take screenshot
        let tab = browser.new_tab()?;
        let data_url = format!(
            "data:text/html;charset=utf-8,{}",
            urlencoding::encode(rendered.as_str())
        );

        tab.call_method(Page::SetDeviceMetricsOverride {
            width: 800,
            height: 480,
            device_scale_factor: 1.0,
            mobile: false,
            scale: None,
            screen_width: Some(800),
            screen_height: Some(480),
            position_x: None,
            position_y: None,
            dont_set_visible_size: None,
            screen_orientation: None,
            viewport: None,
        })?;

        tab.navigate_to(&data_url)?;
        tab.wait_until_navigated()?;

        debug!("capturing screenshot");
        // Take screenshot and save to filesystem
        // Replace this with a Pathbuf
        let screenshot_data = tab.capture_screenshot(
            headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
            None,
            None,
            true,
        )?;

        let png = load_from_memory_with_format(&screenshot_data, ImageFormat::Png);
        match png {
            Ok(png) => {
                let mut grayscale = png.grayscale();
                let mut grayscale = grayscale.as_mut_luma8().unwrap();
                dither(&mut grayscale, &BiLevel);
                grayscale.save(&self.png_path).unwrap();
                grayscale.save(&self.bmp_path).unwrap();
            }
            Err(err) => return Err(err.into()),
        }
        debug!("wrote PNG");
        Ok(())
    }

    async fn render_images(&self, url: &str) -> Result<(), anyhow::Error> {
        // Not sure why we have to do this but it works
        let base = OsStr::new("--hide-scrollbars");
        let args = vec![base];

        // Create browser with custom window size
        let launch_options = LaunchOptions::default_builder()
            .window_size(Some((800, 480)))
            .headless(true)
            .sandbox(false)
            .args(args)
            .build()?;

        debug!("starting browser");
        let browser = Browser::new(launch_options);
        let browser = match browser {
            Err(err) => panic!("{}", err),
            Ok(browser) => browser,
        };

        // Navigate to the URL and take screenshot
        let tab = browser.new_tab()?;

        tab.call_method(Page::SetDeviceMetricsOverride {
            width: 800,
            height: 480,
            device_scale_factor: 1.0,
            mobile: false,
            scale: None,
            screen_width: Some(800),
            screen_height: Some(480),
            position_x: None,
            position_y: None,
            dont_set_visible_size: None,
            screen_orientation: None,
            viewport: None,
        })?;

        tab.navigate_to(&url)?;
        tab.wait_until_navigated()?;

        debug!("capturing screenshot");
        // Take screenshot and save to filesystem
        // Replace this with a Pathbuf
        let screenshot_data = tab.capture_screenshot(
            headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
            None,
            None,
            true,
        )?;

        let png = load_from_memory_with_format(&screenshot_data, ImageFormat::Png);
        match png {
            Ok(png) => {
                let mut grayscale = png.grayscale();
                let mut grayscale = grayscale.as_mut_luma8().unwrap();
                dither(&mut grayscale, &BiLevel);
                grayscale.save(&self.png_path).unwrap();
                grayscale.save(&self.bmp_path).unwrap();
            }
            Err(err) => return Err(err.into()),
        }
        debug!("wrote PNG");
        Ok(())
    }
}
