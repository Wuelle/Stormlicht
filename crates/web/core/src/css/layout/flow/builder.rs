//! Contains [BoxTreeBuilder]
//!
//! Thanks to servo, the basic builder algorithm is the same as theirs
//! <https://github.com/servo/servo/blob/main/components/layout_2020/flow/construct.rs>

use crate::{
    css::{
        layout::{
            flow::{
                BlockContainer, BlockLevelBox, InFlowBlockBox, InlineBox, InlineFormattingContext,
                InlineLevelBox,
            },
            formatting_context::IndependentFormattingContext,
            replaced::ReplacedElement,
        },
        ComputedStyle, StyleComputer,
    },
    dom::{dom_objects, DomPtr},
};

use super::{float, positioning::AbsolutelyPositionedBox, TextRun};

#[derive(Clone)]
pub struct BlockContainerBuilder<'stylesheets, 'parent_style> {
    style_computer: StyleComputer<'stylesheets>,
    style: &'parent_style ComputedStyle,
    block_level_boxes: Vec<BlockLevelBox>,
    current_inline_formatting_context: InlineFormattingContext,
    inline_stack: Vec<InlineBox>,
}

impl<'stylesheets, 'parent_style> BlockContainerBuilder<'stylesheets, 'parent_style> {
    pub fn build(
        node: DomPtr<dom_objects::Node>,
        style_computer: StyleComputer<'stylesheets>,
        style: &'parent_style ComputedStyle,
    ) -> BlockContainer {
        let mut builder = Self {
            style_computer,
            style,
            block_level_boxes: Vec::new(),
            current_inline_formatting_context: InlineFormattingContext::default(),
            inline_stack: Vec::new(),
        };

        builder.traverse_subtree(node, style);

        if !builder.current_inline_formatting_context.is_empty() {
            if builder.block_level_boxes.is_empty() {
                return BlockContainer::InlineFormattingContext(
                    builder.current_inline_formatting_context,
                );
            }
            builder.end_inline_formatting_context();
        }

        BlockContainer::BlockLevelBoxes(builder.block_level_boxes)
    }

    fn traverse_subtree(&mut self, node: DomPtr<dom_objects::Node>, parent_style: &ComputedStyle) {
        for child in node.borrow().children() {
            if let Some(element) = child.try_into_type::<dom_objects::Element>() {
                let computed_style = self
                    .style_computer
                    .get_computed_style(element.clone(), parent_style);

                if computed_style.display().is_none() {
                    continue;
                }

                if computed_style.display().is_inline() {
                    self.push_inline_box(element, computed_style);
                } else {
                    self.push_block_box(element, computed_style);
                }
            } else if let Some(text) = child.try_into_type::<dom_objects::Text>() {
                // Content that would later be collapsed away according to the white-space property
                // does not generate inline boxes
                let text = text.borrow();
                if text.content().contains(|c: char| !c.is_whitespace()) {
                    let text_run = TextRun::new(text.content().to_owned(), parent_style.clone());
                    self.push_text(text_run);
                }
            }
        }
    }

    /// Wrap the current inline formatting context in a block level box and push
    fn end_inline_formatting_context(&mut self) {
        debug_assert!(!self.current_inline_formatting_context.is_empty());

        let formatting_context = std::mem::take(&mut self.current_inline_formatting_context);
        self.block_level_boxes.push(
            InFlowBlockBox::create_anonymous_box(
                BlockContainer::InlineFormattingContext(formatting_context),
                self.style,
            )
            .into(),
        );
    }

    fn push_text(&mut self, text_run: TextRun) {
        let text_box = InlineLevelBox::TextRun(text_run);

        if let Some(top_box) = self.inline_stack.last_mut() {
            top_box.push(text_box);
        } else {
            // inline box stack is empty
            self.current_inline_formatting_context.push(text_box);
        }
    }

    fn push_inline_box(&mut self, element: DomPtr<dom_objects::Element>, style: ComputedStyle) {
        let inline_box = if let Some(replaced_element) =
            ReplacedElement::try_from(element.clone(), style.clone())
        {
            InlineLevelBox::Replaced(replaced_element)
        } else {
            self.inline_stack
                .push(InlineBox::new(element.clone().upcast(), style.clone()));

            // Traverse all children, they will be appended to the inline box we just created
            self.traverse_subtree(element.upcast(), &style);

            // Pop the inline box from the stack and append it to its parents list of children
            // unless the stack of open inline boxes is empty, in which case this was a top level box
            // and we append it to the ongoing inline formatting context instead
            InlineLevelBox::InlineBox(
                self.inline_stack
                    .pop()
                    .expect("stack of open inline boxes should not be empty"),
            )
        };

        if let Some(top_box) = self.inline_stack.last_mut() {
            top_box.push(inline_box);
        } else {
            // inline box stack is empty
            self.current_inline_formatting_context.push(inline_box);
        }
    }

    fn push_block_box(&mut self, element: DomPtr<dom_objects::Element>, style: ComputedStyle) {
        // Split all currently open inline boxes around the block box
        if !self.inline_stack.is_empty() {
            // Split each inline box - these will end up on the "right side" of the block box
            let mut fragments = self.inline_stack.iter().map(InlineBox::split_off).collect();

            std::mem::swap(&mut self.inline_stack, &mut fragments);

            // Push the fragmented tree on the "left side" of the block box to the current
            // inline formatting context
            let left_side_box = fragments
                .into_iter()
                .reduce(|child_tree, mut parent| {
                    parent.push(InlineLevelBox::InlineBox(child_tree));
                    parent
                })
                .expect("inline box stack cannot be empty");
            self.current_inline_formatting_context
                .push(InlineLevelBox::InlineBox(left_side_box));
        }

        // End the current inline formatting context, as we've found a block box
        // that interrupts the sequence of inline boxes
        if !self.current_inline_formatting_context.is_empty() {
            self.end_inline_formatting_context();
        }

        // Push the actual box
        let is_absolutely_positioned =
            style.position().is_absolute() || style.position().is_fixed();

        let block_box = match (style.float().side(), is_absolutely_positioned) {
            (Some(side), _) => {
                let content = IndependentFormattingContext::create(
                    element.clone(),
                    self.style_computer,
                    style.clone(),
                );
                float::FloatingBox::new(element.upcast(), style, side, content).into()
            },
            (None, true) => {
                let content = IndependentFormattingContext::create(
                    element.clone(),
                    self.style_computer,
                    style.clone(),
                );

                AbsolutelyPositionedBox {
                    style,
                    node: element.upcast(),
                    content,
                }
                .into()
            },
            (None, false) => {
                if let Some(replaced_element) =
                    ReplacedElement::try_from(element.clone(), style.clone())
                {
                    replaced_element.into()
                } else {
                    let content = BlockContainerBuilder::build(
                        element.clone().upcast(),
                        self.style_computer,
                        &style,
                    );
                    InFlowBlockBox::new(style, Some(element.upcast()), content).into()
                }
            },
        };

        self.block_level_boxes.push(block_box);
    }
}
