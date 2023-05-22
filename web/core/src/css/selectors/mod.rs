//! <https://drafts.csswg.org/selectors-4/>

mod any_value;
mod attribute_matcher;
mod attribute_modifier;
mod attribute_selector;
mod class_selector;
mod combinator;
mod complex_real_selector;
mod complex_selector;
mod complex_selector_unit;
mod compound_selector;
mod id_selector;
mod legacy_pseudo_element_selector;
mod ns_prefix;
mod pseudo_class_selector;
mod pseudo_compound_selector;
mod pseudo_element_selector;
mod relative_real_selector;
mod relative_selector;
mod simple_selector;
mod subclass_selector;
mod type_selector;
mod wq_name;

pub use any_value::AnyValue;
pub use attribute_matcher::AttributeMatcher;
pub use attribute_modifier::AttributeModifier;
pub use attribute_selector::AttributeSelector;
pub use class_selector::ClassSelector;
pub use combinator::Combinator;
pub use complex_real_selector::{ComplexRealSelector, ComplexRealSelectorList};
pub use complex_selector::{ComplexSelector, ComplexSelectorList, SelectorList};
pub use complex_selector_unit::{ComplexSelectorUnit, ComplexSelectorUnitPart};
pub use compound_selector::{CompoundSelector, CompoundSelectorList};
pub use id_selector::IDSelector;
pub use legacy_pseudo_element_selector::LegacyPseudoElementSelector;
pub use ns_prefix::NSPrefix;
pub use pseudo_class_selector::PseudoClassSelector;
pub use pseudo_compound_selector::PseudoCompoundSelector;
pub use pseudo_element_selector::PseudoElementSelector;
pub use relative_real_selector::{RelativeRealSelector, RelativeRealSelectorList};
pub use relative_selector::{RelativeSelector, RelativeSelectorList};
pub use simple_selector::{SimpleSelector, SimpleSelectorList};
pub use subclass_selector::SubClassSelector;
pub use type_selector::TypeSelector;
pub use wq_name::WQName;

use super::parser::{CSSParse, ParseError, Parser};

/// <https://drafts.csswg.org/selectors-4/#parse-selector>
pub fn parse_selector<'a>(parser: &mut Parser<'a>) -> Result<SelectorList<'a>, ParseError> {
    // 1. Let selector be the result of parsing source as a <selector-list>. If this returns failure,
    //    it’s an invalid selector; return failure.
    let selector = SelectorList::parse(parser)?;

    // 2. If selector is an invalid selector for any other reason (such as, for example,
    //    containing an undeclared namespace prefix), return failure.
    if false {
        return Err(ParseError);
    }

    // 3. Otherwise, return selector.
    Ok(selector)
}

pub trait CSSValidateSelector {
    fn is_valid(&self) -> bool {
        true
    }
}