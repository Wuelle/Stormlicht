use dom_derive::inherit;

use super::HtmlElement;

/// <https://html.spec.whatwg.org/multipage/grouping-content.html#the-p-element>
#[inherit(HtmlElement)]
pub struct HtmlParagraphElement {}

impl HtmlParagraphElement {
    pub fn new(html_element: HtmlElement) -> Self {
        Self {
            __parent: html_element,
        }
    }
}
