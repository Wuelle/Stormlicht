use std::{fmt, fmt::Write};

use math::{Rectangle, Vec2D};

use crate::{
    css::{
        fragment_tree::{BoxFragment, Fragment},
        layout::{replaced::ReplacedElement, ContainingBlock, Pixels, Sides},
        style::{
            computed::{Clear, Margin, Padding},
            specified::DisplayInside,
        },
        values::{AutoOr, PercentageOr},
        ComputedStyle, StyleComputer,
    },
    dom::{dom_objects, DomPtr},
    TreeDebug, TreeFormatter,
};

use super::{
    positioning::AbsolutelyPositionedBox, BlockContainerBuilder, FloatContext, FloatingBox,
    InlineFormattingContext,
};

/// <https://drafts.csswg.org/css2/#block-formatting>
///
/// Holds state about collapsible margins and floating elements.
#[derive(Clone)]
pub struct BlockFormattingContextState {
    last_margin: Pixels,
    float_context: FloatContext,
}

impl BlockFormattingContextState {
    #[must_use]
    pub fn new(containing_block: ContainingBlock) -> Self {
        Self {
            last_margin: Pixels::ZERO,
            float_context: FloatContext::new(containing_block),
        }
    }

    fn prevent_margin_collapse(&mut self) {
        self.last_margin = Pixels::ZERO;
    }

    fn get_collapsed_margin(&mut self, specified_margin: Pixels) -> Pixels {
        if specified_margin <= self.last_margin {
            // The new margin fully collapses into the previous one
            Pixels::ZERO
        } else {
            let used_margin = specified_margin - self.last_margin;
            self.last_margin = specified_margin;
            used_margin
        }
    }
}

#[derive(Clone)]
pub struct BlockFormattingContext {
    contents: BlockContainer,
}

impl BlockFormattingContext {
    #[must_use]
    pub fn build(
        element: DomPtr<dom_objects::Element>,
        element_style: ComputedStyle,
        display_inside: DisplayInside,
        style_computer: StyleComputer<'_>,
    ) -> Self {
        let contents = BlockContainerBuilder::build(
            element.upcast(),
            style_computer,
            &element_style,
            display_inside,
        );

        Self { contents }
    }

    #[must_use]
    pub fn layout(&self, containing_block: ContainingBlock) -> ContentLayoutInfo {
        let mut formatting_context_state = BlockFormattingContextState::new(containing_block);

        self.contents
            .layout(containing_block, &mut formatting_context_state)
    }
}

/// A Box that participates in a [BlockFormattingContext]
/// <https://drafts.csswg.org/css2/#block-level-boxes>
#[derive(Clone)]
pub(crate) enum BlockLevelBox {
    Floating(FloatingBox),
    InFlow(InFlowBlockBox),
    AbsolutelyPositioned(AbsolutelyPositionedBox),
    Replaced(ReplacedElement),
}

#[derive(Clone)]
pub struct InFlowBlockBox {
    style: ComputedStyle,

    /// The DOM element that produced this box.
    /// Some boxes might not correspond to a DOM node,
    /// for example anonymous block boxes
    node: Option<DomPtr<dom_objects::Node>>,

    /// Boxes contained by this box
    contents: BlockContainer,
}

/// Elements contained in a [BlockLevelBox]
///
/// <https://drafts.csswg.org/css2/#block-container-box>
#[derive(Clone)]
pub enum BlockContainer {
    BlockLevelBoxes(Vec<BlockLevelBox>),
    InlineFormattingContext(InlineFormattingContext),
}

impl Default for BlockContainer {
    fn default() -> Self {
        Self::InlineFormattingContext(vec![].into())
    }
}

impl InFlowBlockBox {
    #[must_use]
    pub const fn new(
        style: ComputedStyle,
        node: Option<DomPtr<dom_objects::Node>>,
        contents: BlockContainer,
    ) -> Self {
        Self {
            style,
            node,
            contents,
        }
    }

    #[inline]
    #[must_use]
    pub const fn style(&self) -> &ComputedStyle {
        &self.style
    }

    #[must_use]
    pub fn create_anonymous_box(contents: BlockContainer, parent_style: &ComputedStyle) -> Self {
        Self {
            style: parent_style.get_inherited(),
            node: None,
            contents,
        }
    }

    /// Compute layout for this block box, turning it into a fragment
    ///
    /// The `position` parameter describes the top-left corner of the parents
    /// content rect.
    fn fragment(
        &self,
        position: Vec2D<Pixels>,
        containing_block: ContainingBlock,
        formatting_context: &mut BlockFormattingContextState,
    ) -> BoxFragment {
        let mut dimensions = BlockDimensions::compute(self.style(), containing_block);

        // Possibly collapse top margin
        dimensions.margin.top = formatting_context.get_collapsed_margin(dimensions.margin.top);

        // The top-left corner of the content-rect
        let top_left = position + dimensions.content_offset();

        // Prevent margin-collapse of our top margin with the top margin of the
        // first in-flow child if there is a top border or top padding on this element
        if dimensions.border.top != Pixels::ZERO || dimensions.padding.top != Pixels::ZERO {
            formatting_context.prevent_margin_collapse();
        }

        let position_relative_to_formatting_context_root =
            containing_block.position_relative_to_formatting_context_root + top_left;

        // Floats may never be placed above the top edge of their containing block
        formatting_context
            .float_context
            .lower_float_ceiling(position_relative_to_formatting_context_root.y);

        let content_info = self.contents.layout(
            dimensions.as_containing_block(position_relative_to_formatting_context_root),
            formatting_context,
        );

        // If the content did not contain any in-flow elements *but* it has a nonzero
        // height anyways then it does prevent the top and bottom margins from collapsing
        if !content_info.has_in_flow_content
            && dimensions.height.is_not_auto_and(|&l| l != Pixels::ZERO)
        {
            formatting_context.prevent_margin_collapse();
        }

        // Prevent margin-collapse of our bottom margin with the bottom margin of the
        // last in-flow child if there is a bottom border or bottom padding on this element
        if dimensions.border.bottom != Pixels::ZERO || dimensions.padding.bottom != Pixels::ZERO {
            formatting_context.prevent_margin_collapse();
        }

        dimensions.margin.bottom =
            formatting_context.get_collapsed_margin(dimensions.margin.bottom);

        // After having looked at all the children we can now actually determine the box height
        // if it wasn't defined previously
        let height = dimensions.height.unwrap_or(content_info.height);

        // The bottom right corner of the content area
        let bottom_right = top_left + Vec2D::new(dimensions.width, height);

        let content_area = Rectangle::from_corners(top_left, bottom_right);

        // FIXME: This is ugly, refactor the way we tell our parent
        //        about the height of the box fragment
        let padding_area = dimensions.padding.surround(content_area);
        let margin_area = dimensions
            .margin
            .surround(dimensions.border.surround(padding_area));

        BoxFragment::new(
            self.node.clone(),
            self.style().clone(),
            margin_area,
            dimensions.border,
            padding_area,
            content_area,
            content_info.fragments,
        )
    }
}

impl From<FloatingBox> for BlockLevelBox {
    fn from(value: FloatingBox) -> Self {
        Self::Floating(value)
    }
}

impl From<InFlowBlockBox> for BlockLevelBox {
    fn from(value: InFlowBlockBox) -> Self {
        Self::InFlow(value)
    }
}

impl From<AbsolutelyPositionedBox> for BlockLevelBox {
    fn from(value: AbsolutelyPositionedBox) -> Self {
        Self::AbsolutelyPositioned(value)
    }
}

impl From<ReplacedElement> for BlockLevelBox {
    fn from(value: ReplacedElement) -> Self {
        Self::Replaced(value)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ContentLayoutInfo {
    pub height: Pixels,
    pub fragments: Vec<Fragment>,
    pub has_in_flow_content: bool,
}

impl BlockContainer {
    #[must_use]
    pub(crate) fn layout(
        &self,
        containing_block: ContainingBlock,
        formatting_context: &mut BlockFormattingContextState,
    ) -> ContentLayoutInfo {
        match &self {
            Self::BlockLevelBoxes(block_level_boxes) => {
                let mut state = BlockFlowState::new(containing_block, formatting_context);
                for block_box in block_level_boxes {
                    state.visit_block_box(block_box);
                }
                state.finish()
            },
            Self::InlineFormattingContext(inline_formatting_context) => {
                // Margins cannot collapse across inline formatting contexts
                // FIXME: Zero-height line boxes do not prevent margin collapse
                //        https://drafts.csswg.org/css2/#inline-formatting
                formatting_context.prevent_margin_collapse();

                let (fragments, height) = inline_formatting_context.layout(containing_block);

                ContentLayoutInfo {
                    height,
                    fragments,
                    has_in_flow_content: true,
                }
            },
        }
    }
}

pub struct BlockFlowState<'box_tree, 'formatting_context> {
    block_formatting_context: &'formatting_context mut BlockFormattingContextState,
    cursor: Vec2D<Pixels>,
    fragments_so_far: Vec<Fragment>,
    containing_block: ContainingBlock,
    absolute_boxes_requiring_layout: Vec<AbsoluteBoxRequiringLayout<'box_tree>>,
    has_in_flow_content: bool,
}

#[derive(Clone, Copy)]
struct AbsoluteBoxRequiringLayout<'a> {
    absolute_box: &'a AbsolutelyPositionedBox,
    static_position: Vec2D<Pixels>,
    index: usize,
}

impl<'box_tree, 'formatting_context> BlockFlowState<'box_tree, 'formatting_context> {
    pub fn new(
        containing_block: ContainingBlock,
        formatting_context: &'formatting_context mut BlockFormattingContextState,
    ) -> Self {
        Self {
            cursor: Vec2D::new(Pixels::ZERO, Pixels::ZERO),
            block_formatting_context: formatting_context,
            fragments_so_far: vec![],
            containing_block,
            absolute_boxes_requiring_layout: vec![],
            has_in_flow_content: false,
        }
    }

    fn respect_clearance(&mut self, clear: &Clear) {
        let clear_to = match clear {
            Clear::Left => self.block_formatting_context.float_context.clear_left(),
            Clear::Right => self.block_formatting_context.float_context.clear_right(),
            Clear::Both => self.block_formatting_context.float_context.clear_both(),
            _ => return,
        };

        // The clear value is always relative to the formatting context root
        // - make it relative to our containing block (which is where the cursor lives)
        let clear_to = clear_to
            - self
                .containing_block
                .position_relative_to_formatting_context_root
                .y;

        if self.cursor.y < clear_to {
            // Introduce "clearance".
            // This prevents margin collapse
            self.block_formatting_context.prevent_margin_collapse();

            self.cursor.y = clear_to;
        }
    }

    pub fn visit_block_box(&mut self, block_box: &'box_tree BlockLevelBox) {
        match block_box {
            BlockLevelBox::Floating(float_box) => {
                self.respect_clearance(float_box.style.clear());

                // Floats are placed at or below the flow position
                let new_ceiling = self.cursor.y
                    + self
                        .containing_block
                        .position_relative_to_formatting_context_root
                        .y;
                self.block_formatting_context
                    .float_context
                    .lower_float_ceiling(new_ceiling);

                let box_fragment = float_box.layout(
                    self.containing_block,
                    &mut self.block_formatting_context.float_context,
                );

                self.fragments_so_far.push(box_fragment.into())
            },
            BlockLevelBox::InFlow(in_flow_box) => {
                self.respect_clearance(in_flow_box.style.clear());

                // Every block box creates exactly one box fragment
                let box_fragment = in_flow_box.fragment(
                    self.cursor,
                    self.containing_block,
                    self.block_formatting_context,
                );

                let box_height = box_fragment.margin_area().height();
                self.cursor.y += box_height;

                self.fragments_so_far.push(Fragment::Box(box_fragment));
            },
            BlockLevelBox::AbsolutelyPositioned(absolute_box) => {
                // Absolutely positioned boxes cannot be laid out during the initial pass,
                // as their positioning requires the size of the containing block to be known.
                //
                // However, they should still be painted in tree-order.
                // To accomodate this, we keep track of the absolute boxes we found during the first
                // pass and later insert the fragments at the correct position once the
                // size of the containing block is known.
                self.absolute_boxes_requiring_layout
                    .push(AbsoluteBoxRequiringLayout {
                        absolute_box,
                        static_position: self.cursor,
                        index: self.fragments_so_far.len(),
                    });
            },
            BlockLevelBox::Replaced(replaced_element) => {
                self.layout_block_level_replaced_element(replaced_element);
            },
        }
    }

    pub fn finish(self) -> ContentLayoutInfo {
        // Now that we have processed all in-flow elements, we can layout absolutely positioned
        // elements.
        let height = self.cursor.y;
        let mut fragments = self.fragments_so_far;
        let definite_containing_block = self.containing_block.make_definite(height);

        for task in self.absolute_boxes_requiring_layout {
            let fragment = task
                .absolute_box
                .layout(definite_containing_block, task.static_position);
            fragments.insert(task.index, fragment.into());
        }

        ContentLayoutInfo {
            height,
            fragments,
            has_in_flow_content: self.has_in_flow_content,
        }
    }

    fn layout_block_level_replaced_element(&mut self, replaced_element: &ReplacedElement) {
        let element_style = replaced_element.style();
        self.respect_clearance(element_style.clear());

        let content_size = replaced_element.used_size_if_it_was_inline(self.containing_block);

        let resolve_margin =
            |margin: &Margin| margin.map(|p| p.resolve_against(self.containing_block.width()));

        // Choose horizontal margins such that the total width of the element is equal to the available space.
        // This is similar to https://drafts.csswg.org/css2/#blockwidth, except padding and borders are always zero
        let horizontal_margin_space = self.containing_block.width() - content_size.width;
        let mut margin_left = resolve_margin(element_style.margin_left());
        let mut margin_right = resolve_margin(element_style.margin_right());

        if content_size.width + margin_left.unwrap_or_default() + margin_right.unwrap_or_default()
            > self.containing_block.width()
        {
            margin_left = AutoOr::NotAuto(margin_left.unwrap_or_default());
            margin_right = AutoOr::NotAuto(margin_right.unwrap_or_default());
        }

        let (margin_left, margin_right) = match (margin_left, margin_right) {
            (AutoOr::Auto, AutoOr::Auto) => {
                let margin = horizontal_margin_space / 2.;
                (margin, margin)
            },
            (AutoOr::NotAuto(margin_left), AutoOr::Auto) => {
                (margin_left, horizontal_margin_space - margin_left)
            },
            (AutoOr::Auto, AutoOr::NotAuto(margin_right)) => {
                (horizontal_margin_space - margin_right, margin_right)
            },
            (AutoOr::NotAuto(margin_left), AutoOr::NotAuto(_)) => {
                // Overconstrained case
                (margin_left, horizontal_margin_space - margin_left)
            },
        };

        let mut margins = Sides {
            top: resolve_margin(element_style.margin_top()).unwrap_or_default(),
            right: margin_right,
            bottom: resolve_margin(element_style.margin_bottom()).unwrap_or_default(),
            left: margin_left,
        };

        // Perform margin-collapse
        margins.top = self
            .block_formatting_context
            .get_collapsed_margin(margins.top);

        if content_size.height != Pixels::ZERO {
            self.block_formatting_context.prevent_margin_collapse();
        }

        margins.bottom = self
            .block_formatting_context
            .get_collapsed_margin(margins.bottom);

        // Create a fragment for at the calculated position
        let content_position = Vec2D::new(margins.left, self.cursor.y + margins.top);
        let fragment = replaced_element
            .content()
            .create_fragment(content_position, content_size);

        // Advance the flow state
        self.cursor.y += margins.vertical_sum() + content_size.height;

        self.fragments_so_far.push(fragment);
    }
}

#[derive(Clone, Copy, Debug)]
struct BlockDimensions {
    margin: Sides<Pixels>,
    padding: Sides<Pixels>,
    border: Sides<Pixels>,
    width: Pixels,
    height: AutoOr<Pixels>,
}

impl BlockDimensions {
    /// Compute dimensions for normal, non replaced block elements
    ///
    /// The relevant parts of the specification are:
    /// * https://drafts.csswg.org/css2/#blockwidth
    /// * https://drafts.csswg.org/css2/#normal-block
    ///
    /// This method does **not** layout the blocks contents nor does it perform margin-collapsing.
    #[must_use]
    fn compute(style: &ComputedStyle, containing_block: ContainingBlock) -> Self {
        // NOTE: This is not a mistake, *both* horizontal and vertical margin percentages are calculated
        //       with respect to the *width* of the containing block.
        //       Refer to https://drafts.csswg.org/css2/#margin-properties
        let available_length = containing_block.width();
        let resolve_margin = |margin: &Margin| margin.map(|p| p.resolve_against(available_length));

        let resolve_padding = |padding: &Padding| padding.resolve_against(available_length);

        let padding = Sides {
            top: resolve_padding(style.padding_top()),
            right: resolve_padding(style.padding_right()),
            bottom: resolve_padding(style.padding_bottom()),
            left: resolve_padding(style.padding_left()),
        };

        let border = style.used_border_widths();

        // See https://drafts.csswg.org/css2/#blockwidth for a description of how the width is computed
        let width = style.width().map(|p| p.resolve_against(available_length));

        let mut margin_left = resolve_margin(style.margin_left());
        let mut margin_right = resolve_margin(style.margin_right());

        // Margins are treated as zero if the total width exceeds the available width
        let total_width_is_more_than_available = |width: &Pixels| {
            let total_width = margin_left.unwrap_or_default()
                + border.horizontal_sum()
                + padding.horizontal_sum()
                + *width
                + margin_right.unwrap_or_default();
            total_width > containing_block.width()
        };
        if width.is_not_auto_and(total_width_is_more_than_available) {
            margin_left = margin_left.or(AutoOr::NotAuto(Pixels::ZERO));
            margin_right = margin_right.or(AutoOr::NotAuto(Pixels::ZERO));
        }

        // If there is exactly one value specified as auto, its used value follows from the equality.
        let (width, margin_left, margin_right) = match (width, margin_left, margin_right) {
            (AutoOr::Auto, margin_left, margin_right) => {
                // If width is set to auto, any other auto values become 0 and width follows from the resulting equality.
                let margin_left: Pixels = margin_left.unwrap_or(Pixels::ZERO);
                let margin_right = margin_right.unwrap_or(Pixels::ZERO);
                let width = containing_block.width()
                    - margin_left
                    - border.horizontal_sum()
                    - padding.horizontal_sum()
                    - margin_right;
                (width, margin_left, margin_right)
            },
            (AutoOr::NotAuto(width), AutoOr::Auto, AutoOr::Auto) => {
                let margin_width = (containing_block.width()
                    - border.horizontal_sum()
                    - padding.horizontal_sum()
                    - width)
                    / 2.;
                (width, margin_width, margin_width)
            },
            (AutoOr::NotAuto(width), AutoOr::NotAuto(margin_left), AutoOr::Auto) => {
                let margin_right = containing_block.width()
                    - margin_left
                    - border.horizontal_sum()
                    - padding.horizontal_sum()
                    - width;
                (width, margin_left, margin_right)
            },
            (AutoOr::NotAuto(width), AutoOr::Auto, AutoOr::NotAuto(margin_right)) => {
                let margin_left = containing_block.width()
                    - border.horizontal_sum()
                    - padding.horizontal_sum()
                    - width
                    - margin_right;
                (width, margin_left, margin_right)
            },
            (AutoOr::NotAuto(width), AutoOr::NotAuto(margin_left), AutoOr::NotAuto(_)) => {
                // The values are overconstrained
                // FIXME: If the "direction" property is "rtl", we should ignore the margin left instead
                let margin_right = containing_block.width()
                    - margin_left
                    - border.horizontal_sum()
                    - padding.horizontal_sum()
                    - width;
                (width, margin_left, margin_right)
            },
        };

        // Compute the height according to https://drafts.csswg.org/css2/#normal-block
        // If the height is a percentage it is
        let height = style.height().flat_map(|percentage_or_length| {
            match percentage_or_length {
                PercentageOr::Percentage(percentage) => {
                    if let Some(available_height) = containing_block.height() {
                        AutoOr::NotAuto(available_height * percentage.as_fraction())
                    } else {
                        // If the value is a percentage but the length of the containing block is not
                        // yet determined, the value should be treated as auto.
                        // (https://drafts.csswg.org/css2/#the-height-property)
                        AutoOr::Auto
                    }
                },
                PercentageOr::NotPercentage(length) => AutoOr::NotAuto(length),
            }
        });

        let margin = Sides {
            top: resolve_margin(style.margin_top()).unwrap_or_default(),
            right: margin_right,
            bottom: resolve_margin(style.margin_bottom()).unwrap_or_default(),
            left: margin_left,
        };

        Self {
            margin,
            padding,
            border,
            width,
            height,
        }
    }

    /// Return the offset of the top-left corner of the content area from the top-left corner
    /// of the margin area
    #[must_use]
    fn content_offset(&self) -> Vec2D<Pixels> {
        Vec2D::new(
            self.margin.left + self.border.left + self.padding.left,
            self.margin.top + self.border.top + self.padding.top,
        )
    }

    #[must_use]
    fn as_containing_block(
        &self,
        position_relative_to_formatting_context_root: Vec2D<Pixels>,
    ) -> ContainingBlock {
        ContainingBlock {
            width: self.width,
            height: self.height.into_option(),
            position_relative_to_formatting_context_root,
        }
    }
}

impl TreeDebug for BlockLevelBox {
    fn tree_fmt(&self, formatter: &mut TreeFormatter<'_, '_>) -> fmt::Result {
        match self {
            Self::Floating(float_box) => float_box.tree_fmt(formatter),
            Self::AbsolutelyPositioned(abs_box) => abs_box.tree_fmt(formatter),
            Self::InFlow(block_box) => block_box.tree_fmt(formatter),
            Self::Replaced(_) => {
                formatter.indent()?;
                write!(formatter, "Replaced Element")
            },
        }
    }
}

impl TreeDebug for InFlowBlockBox {
    fn tree_fmt(&self, formatter: &mut TreeFormatter<'_, '_>) -> std::fmt::Result {
        formatter.indent()?;
        write!(formatter, "Block Box")?;
        if let Some(node) = &self.node {
            writeln!(formatter, " ({:?})", node.underlying_type())?;
        } else {
            writeln!(formatter, " (anonymous)")?;
        }

        formatter.increase_indent();
        self.contents.tree_fmt(formatter)?;
        formatter.decrease_indent();
        Ok(())
    }
}

impl TreeDebug for BlockContainer {
    fn tree_fmt(&self, formatter: &mut TreeFormatter<'_, '_>) -> fmt::Result {
        match &self {
            Self::BlockLevelBoxes(block_level_boxes) => {
                for block_box in block_level_boxes {
                    block_box.tree_fmt(formatter)?;
                }
                Ok(())
            },
            Self::InlineFormattingContext(inline_formatting_context) => {
                inline_formatting_context.tree_fmt(formatter)
            },
        }
    }
}

impl TreeDebug for BlockFormattingContext {
    fn tree_fmt(&self, formatter: &mut TreeFormatter<'_, '_>) -> fmt::Result {
        self.contents.tree_fmt(formatter)
    }
}

impl From<BlockContainer> for BlockFormattingContext {
    fn from(contents: BlockContainer) -> Self {
        Self { contents }
    }
}
