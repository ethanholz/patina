use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use headless_chrome::{Browser, LaunchOptions, protocol::cdp::Page};
use log::debug;
use tokio::{fs, process::Command};

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
        self.render_png(url).await?;
        self.convert_png_to_bmp().await?;
        Ok(())
    }

    async fn render_png(&self, url: &str) -> Result<(), anyhow::Error> {
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

        fs::write(&self.png_path, screenshot_data).await?;
        debug!("wrote PNG");
        Ok(())
    }

    pub async fn convert_png_to_bmp(&self) -> Result<(), anyhow::Error> {
        let bmp_output = format!("bmp3:{}", self.bmp_path.display());
        debug!("converting using magick");
        let _ = Command::new("magick")
            .args([
                self.png_path.display().to_string().as_str(),
                "-monochrome",
                "-depth",
                "1",
                "-strip",
                &bmp_output,
            ])
            .output()
            .await?;
        debug!("converted successfully");

        Ok(())
    }
}
