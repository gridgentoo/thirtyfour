use fantoccini::elements::{Element, ElementRef};
use fantoccini::error::CmdError;
use serde::ser::{Serialize, Serializer};
use serde_json::Value;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::error::WebDriverError;
use crate::session::handle::SessionHandle;
use crate::{common::types::ElementRect, error::WebDriverResult, By, ElementRefHelper};

/// The WebElement struct encapsulates a single element on a page.
///
/// WebElement structs are generally not constructed manually, but rather
/// they are returned from a 'find_element()' operation using a WebDriver.
///
/// # Example:
/// ```rust
/// # use thirtyfour::prelude::*;
/// # use thirtyfour::support::block_on;
/// #
/// # fn main() -> WebDriverResult<()> {
/// #     block_on(async {
/// #         let caps = DesiredCapabilities::chrome();
/// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
/// #         driver.get("http://webappdemo").await?;
/// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
/// let elem = driver.find_element(By::Id("input-result")).await?;
/// #         assert_eq!(elem.get_attribute("id").await?, Some("input-result".to_string()));
/// #         driver.quit().await?;
/// #         Ok(())
/// #     })
/// # }
/// ```
///
/// You can also search for a child element of another element as follows:
/// ```rust
/// # use thirtyfour::prelude::*;
/// # use thirtyfour::support::block_on;
/// #
/// # fn main() -> WebDriverResult<()> {
/// #     block_on(async {
/// #         let caps = DesiredCapabilities::chrome();
/// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
/// #         driver.get("http://webappdemo").await?;
/// let elem = driver.find_element(By::Css("div[data-section='section-buttons']")).await?;
/// let child_elem = elem.find_element(By::Tag("button")).await?;
/// #         child_elem.click().await?;
/// #         let result_elem = elem.find_element(By::Id("button-result")).await?;
/// #         assert_eq!(result_elem.text().await?, "Button 1 clicked");
/// #         driver.quit().await?;
/// #         Ok(())
/// #     })
/// # }
/// ```
///
/// Elements can be clicked using the `click()` method, and you can send
/// input to an element using the `send_keys()` method.
///
#[derive(Clone)]
pub struct WebElement {
    pub element: Element,
    pub handle: SessionHandle,
}

impl Deref for WebElement {
    type Target = Element;

    fn deref(&self) -> &Self::Target {
        &self.element
    }
}

impl DerefMut for WebElement {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.element
    }
}

impl Debug for WebElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebElement").field("element", &self.element).finish()
    }
}

impl WebElement {
    /// Create a new WebElement struct.
    ///
    /// Typically you would not call this directly. WebElement structs are
    /// usually constructed by calling one of the find_element*() methods
    /// either on WebDriver or another WebElement.
    pub(crate) fn new(element: Element, handle: SessionHandle) -> Self {
        Self {
            element,
            handle,
        }
    }

    pub fn from_json(value: Value, handle: SessionHandle) -> WebDriverResult<Self> {
        let element_ref: ElementRefHelper = serde_json::from_value(value)?;
        Ok(Self {
            element: Element::from_element_id(handle.client.clone(), element_ref.into()),
            handle,
        })
    }

    pub fn to_json(&self) -> WebDriverResult<Value> {
        Ok(serde_json::to_value(self.element.clone())?)
    }

    pub fn element_id(&self) -> ElementRef {
        self.element.element_id()
    }

    /// Get the bounding rectangle for this WebElement.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// let r = elem.rect().await?;
    /// #         assert!(r.x > 0.0f64);
    /// #         assert!(r.y > 0.0f64);
    /// #         assert!(r.width > 0.0f64);
    /// #         assert!(r.height > 0.0f64);
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn rect(&self) -> WebDriverResult<ElementRect> {
        let (x, y, w, h) = self.element.rectangle().await?;
        Ok(ElementRect {
            x,
            y,
            width: w,
            height: h,
        })
    }

    /// Get the tag name for this WebElement.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// assert_eq!(elem.tag_name().await?, "button");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn tag_name(&self) -> WebDriverResult<String> {
        Ok(self.element.tag_name().await?)
    }

    /// Get the class name for this WebElement.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// let class_name_option = elem.class_name().await?;  // Option<String>
    /// #         assert!(class_name_option.expect("Missing class name").contains("pure-button"));
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn class_name(&self) -> WebDriverResult<Option<String>> {
        self.get_attribute("class").await
    }

    /// Get the id for this WebElement.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// let id_option = elem.id().await?;  // Option<String>
    /// #         assert_eq!(id_option, Some("button1".to_string()));
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn id(&self) -> WebDriverResult<Option<String>> {
        self.get_attribute("id").await
    }

    /// Get the text contents for this WebElement.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("button1")).await?.click().await?;
    /// let elem = driver.find_element(By::Id("button-result")).await?;
    /// let text = elem.text().await?;
    /// #         assert_eq!(text, "Button 1 clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn text(&self) -> WebDriverResult<String> {
        Ok(self.element.text().await?)
    }

    /// Convenience method for getting the (optional) value attribute of this element.
    pub async fn value(&self) -> WebDriverResult<Option<String>> {
        self.get_attribute("value").await
    }

    /// Click the WebElement.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// elem.click().await?;
    /// #         let elem = driver.find_element(By::Id("button-result")).await?;
    /// #         assert_eq!(elem.text().await?, "Button 1 clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn click(&self) -> WebDriverResult<()> {
        self.element.click().await?;
        Ok(())
    }

    /// Clear the WebElement contents.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// let elem = driver.find_element(By::Name("input2")).await?;
    /// elem.clear().await?;
    /// #         let cleared_text = elem.text().await?;
    /// #         assert_eq!(cleared_text, "");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn clear(&self) -> WebDriverResult<()> {
        Ok(self.element.clear().await?)
    }

    /// Get the specified property.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #         let elem = driver.find_element(By::Name("input2")).await?;
    /// let property_value_option = elem.get_property("checked").await?; // Option<String>
    /// assert_eq!(property_value_option, Some("true".to_string()));
    /// #         assert_eq!(elem.get_property("invalid-property").await?, None);
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn get_property(&self, name: &str) -> WebDriverResult<Option<String>> {
        Ok(self.element.prop(name).await?)
    }

    /// Get the specified attribute.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #         let elem = driver.find_element(By::Name("input2")).await?;
    /// let attribute_option = elem.get_attribute("name").await?;  // Option<String>
    /// assert_eq!(attribute_option, Some("input2".to_string()));
    /// #         assert_eq!(elem.get_attribute("invalid-attribute").await?, None);
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn get_attribute(&self, name: &str) -> WebDriverResult<Option<String>> {
        Ok(self.element.attr(name).await?)
    }

    /// Get the specified CSS property.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #         let elem = driver.find_element(By::Name("input2")).await?;
    /// let css_color = elem.get_css_property("color").await?;
    /// assert_eq!(css_color, "rgba(0, 0, 0, 1)");
    /// #         assert_eq!(elem.get_css_property("invalid-css-property").await?, "");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn get_css_property(&self, name: &str) -> WebDriverResult<String> {
        Ok(self.element.css_value(name).await?)
    }

    /// Return true if the WebElement is currently selected, otherwise false.
    pub async fn is_selected(&self) -> WebDriverResult<bool> {
        Ok(self.element.is_selected().await?)
    }

    /// Return true if the WebElement is currently displayed, otherwise false.
    ///
    /// # Example
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         let elem = driver.find_element(By::Id("button1")).await?;
    /// let displayed = elem.is_displayed().await?;
    /// #         assert_eq!(displayed, true);
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn is_displayed(&self) -> WebDriverResult<bool> {
        Ok(self.element.is_displayed().await?)
    }

    /// Return true if the WebElement is currently enabled, otherwise false.
    ///
    /// # Example
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         let elem = driver.find_element(By::Id("button1")).await?;
    /// let enabled = elem.is_enabled().await?;
    /// #         assert_eq!(enabled, true);
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn is_enabled(&self) -> WebDriverResult<bool> {
        Ok(self.element.is_enabled().await?)
    }

    /// Return true if the WebElement is currently clickable (visible and enabled),
    /// otherwise false.
    ///
    /// # Example
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         let elem = driver.find_element(By::Id("button1")).await?;
    /// let clickable = elem.is_clickable().await?;
    /// #         assert_eq!(clickable, true);
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn is_clickable(&self) -> WebDriverResult<bool> {
        Ok(self.is_displayed().await? && self.is_enabled().await?)
    }

    /// Return true if the WebElement is currently (still) present
    /// and not stale.
    ///
    /// NOTE: This method simply queries the tag name in order to
    ///       determine whether the element is still present.
    ///
    /// IMPORTANT:
    /// If an element is re-rendered it may be considered stale even
    /// though to the user it looks like it is still there.
    ///
    /// The recommended way to check for the presence of an element is
    /// to simply search for the element again.
    ///
    /// # Example
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         let elem = driver.find_element(By::Id("button1")).await?;
    /// let present = elem.is_present().await?;
    /// #         assert_eq!(present, true);
    /// #         // Check negative case as well.
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #         assert_eq!(elem.is_present().await?, false);
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn is_present(&self) -> WebDriverResult<bool> {
        let present = match self.tag_name().await {
            Ok(..) => true,
            Err(WebDriverError::NoSuchElement(..)) => false,
            Err(e) => return Err(e),
        };
        Ok(present)
    }

    /// Search for a child element of this WebElement using the specified
    /// selector.
    ///
    /// **NOTE**: For more powerful element queries including polling and filters, see the
    ///  [WebElement::query()](struct.WebElement.html#impl-ElementQueryable) method instead.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::Css("div[data-section='section-buttons']")).await?;
    /// let child_elem = elem.find_element(By::Tag("button")).await?;
    /// #         child_elem.click().await?;
    /// #         let result_elem = elem.find_element(By::Id("button-result")).await?;
    /// #         assert_eq!(result_elem.text().await?, "Button 1 clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn find_element(&self, by: By) -> WebDriverResult<WebElement> {
        let elem = self.element.find(by.locator()).await.map_err(|e| match e {
            // It's generally only useful to know the element query that failed.
            CmdError::NoSuchElement(_) => WebDriverError::NoSuchElement(by.to_string()),
            x => WebDriverError::CmdError(x),
        })?;
        Ok(WebElement::new(elem, self.handle.clone()))
    }

    /// Search for all child elements of this WebElement that match the
    /// specified selector.
    ///
    /// **NOTE**: For more powerful element queries including polling and filters, see the
    /// [WebElement::query()](struct.WebElement.html#impl-ElementQueryable) method instead.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::Css("div[data-section='section-buttons']")).await?;
    /// let child_elems = elem.find_elements(By::Tag("button")).await?;
    /// #         assert_eq!(child_elems.len(), 2);
    /// for child_elem in child_elems {
    ///     assert_eq!(child_elem.tag_name().await?, "button");
    /// }
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn find_elements(&self, by: By) -> WebDriverResult<Vec<WebElement>> {
        let elems = self.element.find_all(by.locator()).await.map_err(|e| match e {
            // It's generally only useful to know the element query that failed.
            CmdError::NoSuchElement(_) => WebDriverError::NoSuchElement(by.to_string()),
            x => WebDriverError::CmdError(x),
        })?;
        Ok(elems.into_iter().map(|x| WebElement::new(x, self.handle.clone())).collect())
    }

    /// Send the specified input.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #         let elem = driver.find_element(By::Name("input1")).await?;
    /// elem.send_keys("selenium").await?;
    /// #         assert_eq!(elem.value().await?, Some("selenium".to_string()));
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    ///
    /// You can also send special key combinations like this:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #         let elem = driver.find_element(By::Name("input1")).await?;
    /// elem.send_keys("selenium").await?;
    /// elem.send_keys(Key::Control + "a".to_string()).await?;
    /// elem.send_keys("thirtyfour" + Key::Enter).await?;
    /// #         assert_eq!(elem.value().await?, Some("thirtyfour".to_string()));
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn send_keys(&self, keys: impl AsRef<str>) -> WebDriverResult<()> {
        Ok(self.element.send_keys(keys.as_ref()).await?)
    }

    /// Take a screenshot of this WebElement and return it as PNG bytes.
    pub async fn screenshot_as_png(&self) -> WebDriverResult<Vec<u8>> {
        Ok(self.element.screenshot().await?)
    }

    /// Take a screenshot of this WebElement and write it to the specified filename.
    pub async fn screenshot(&self, path: &Path) -> WebDriverResult<()> {
        let png = self.screenshot_as_png().await?;
        let mut file = File::create(path).await?;
        file.write_all(&png).await?;
        Ok(())
    }

    /// Focus this WebElement using JavaScript.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// let elem = driver.find_element(By::Name("input1")).await?;
    /// elem.focus().await?;
    /// #         driver.action_chain().send_keys("selenium").perform().await?;
    /// #         assert_eq!(elem.value().await?, Some("selenium".to_string()));
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn focus(&self) -> WebDriverResult<()> {
        self.handle.execute_script(r#"arguments[0].focus();"#, vec![self.to_json()?]).await?;
        Ok(())
    }

    /// Scroll this element into view using JavaScript.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// elem.scroll_into_view().await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn scroll_into_view(&self) -> WebDriverResult<()> {
        self.handle
            .execute_script(r#"arguments[0].scrollIntoView();"#, vec![self.to_json()?])
            .await?;
        Ok(())
    }

    /// Get the innerHtml property of this element.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::XPath(r##"//*[@id="button1"]/.."##)).await?;
    /// let html = elem.inner_html().await?;
    /// #         assert_eq!(html, r##"<button class="pure-button pure-button-primary" id="button1">BUTTON 1</button>"##);
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn inner_html(&self) -> WebDriverResult<String> {
        self.get_property("innerHTML").await.map(|x| x.unwrap_or_default())
    }

    /// Get the outerHtml property of this element.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::XPath(r##"//*[@id="button1"]/.."##)).await?;
    /// let html = elem.outer_html().await?;
    /// #         assert_eq!(html, r##"<div class="pure-u-1-6"><button class="pure-button pure-button-primary" id="button1">BUTTON 1</button></div>"##);
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn outer_html(&self) -> WebDriverResult<String> {
        self.get_property("outerHTML").await.map(|x| x.unwrap_or_default())
    }

    /// Get the shadowRoot property of the current element.
    ///
    /// Call this method on the element containing the `#shadowRoot` node.
    /// You can then use the returned `WebElement` to query elements within the shadowRoot node.
    pub async fn get_shadow_root(&self) -> WebDriverResult<WebElement> {
        let ret = self
            .handle
            .execute_script("return arguments[0].shadowRoot", vec![self.to_json()?])
            .await?;
        ret.get_element()
    }
}

impl fmt::Display for WebElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.element)
    }
}

impl Serialize for WebElement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.element.serialize(serializer)
    }
}
