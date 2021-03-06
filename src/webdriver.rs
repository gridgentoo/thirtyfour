use crate::error::WebDriverResult;
use crate::session::handle::SessionHandle;
use std::ops::{Deref, DerefMut};

use crate::TimeoutConfiguration;
use fantoccini::wd::Capabilities;

/// The `WebDriver` struct encapsulates an async Selenium WebDriver browser
/// session.
///
/// # Example:
/// ```rust
/// use thirtyfour::prelude::*;
/// use thirtyfour::support::block_on;
///
/// fn main() -> WebDriverResult<()> {
///     block_on(async {
///         let caps = DesiredCapabilities::chrome();
///         let driver = WebDriver::new("http://localhost:4444", caps).await?;
///         driver.get("http://webappdemo").await?;
///         driver.quit().await?;
///         Ok(())
///     })
/// }
/// ```
#[derive(Debug)]
pub struct WebDriver {
    pub handle: SessionHandle,
}

impl WebDriver {
    /// Create a new WebDriver as follows:
    ///
    /// # Example
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// let caps = DesiredCapabilities::chrome();
    /// let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    ///
    /// **NOTE:** If the webdriver appears to hang or give no response, please check that the
    ///     capabilities object is of the correct type for that webdriver.
    pub async fn new<C>(server_url: &str, capabilities: C) -> WebDriverResult<Self>
    where
        C: Into<Capabilities>,
    {
        use fantoccini::ClientBuilder;
        let caps: Capabilities = capabilities.into();
        let client = ClientBuilder::native().capabilities(caps.clone()).connect(server_url).await?;

        // Set default timeouts.
        let timeouts = TimeoutConfiguration::default();
        client.update_timeouts(timeouts).await?;

        Ok(Self {
            handle: SessionHandle::new(client, caps).await?,
        })
    }

    // /// Creates a new WebDriver just like the `new` function. Allows a
    // /// configurable timeout for all HTTP requests including the session creation.
    // ///
    // /// Create a new WebDriver as follows:
    // ///
    // /// # Example
    // /// ```rust
    // /// # use thirtyfour::prelude::*;
    // /// # use thirtyfour::support::block_on;
    // /// # use std::time::Duration;
    // /// #
    // /// # fn main() -> WebDriverResult<()> {
    // /// #     block_on(async {
    // /// let caps = DesiredCapabilities::chrome();
    // /// let driver = WebDriver::new_with_timeout("http://localhost:4444", &caps, Some(Duration::from_secs(120))).await?;
    // /// #         driver.quit().await?;
    // /// #         Ok(())
    // /// #     })
    // /// # }
    // /// ```
    // pub async fn new_with_timeout<C>(
    //     _server_url: &str,
    //     _capabilities: C,
    //     _timeout: Option<Duration>,
    // ) -> WebDriverResult<Self>
    // where
    //     C: Into<Capabilities>,
    // {
    //     unimplemented!()
    // }

    /// End the webdriver session and close the browser.
    ///
    /// **NOTE:** The browser will not close automatically when `WebDriver` goes out of scope.
    ///           Thus if you intend for the browser to close once you are done with it, then
    ///           you must call this method at that point, and await it.
    pub async fn quit(self) -> WebDriverResult<()> {
        self.handle.client.close().await?;
        Ok(())
    }
}

/// The Deref implementation allows the WebDriver to "fall back" to SessionHandle and
/// exposes all of the methods there without requiring us to use an async_trait.
/// See documentation at the top of this module for more details on the design.
impl Deref for WebDriver {
    type Target = SessionHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl DerefMut for WebDriver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}
