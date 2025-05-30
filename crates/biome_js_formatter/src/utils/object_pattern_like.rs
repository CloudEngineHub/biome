use crate::JsFormatContext;
use crate::js::bindings::parameters::{FormatAnyJsParameters, should_hug_function_parameters};
use crate::prelude::*;
use biome_formatter::formatter::Formatter;
use biome_formatter::write;
use biome_formatter::{Format, FormatResult};
use biome_js_syntax::{
    AnyJsAssignmentPattern, AnyJsBindingPattern, AnyJsFormalParameter,
    AnyJsObjectAssignmentPatternMember, AnyJsObjectBindingPatternMember, JsObjectAssignmentPattern,
    JsObjectBindingPattern, JsSyntaxKind, JsSyntaxToken,
};
use biome_rowan::{AstNode, SyntaxNodeOptionExt, SyntaxResult, declare_node_union};

declare_node_union! {
    pub (crate) JsObjectPatternLike = JsObjectAssignmentPattern | JsObjectBindingPattern
}

impl JsObjectPatternLike {
    fn l_curly_token(&self) -> SyntaxResult<JsSyntaxToken> {
        match self {
            Self::JsObjectAssignmentPattern(node) => node.l_curly_token(),
            Self::JsObjectBindingPattern(node) => node.l_curly_token(),
        }
    }

    fn r_curly_token(&self) -> SyntaxResult<JsSyntaxToken> {
        match self {
            Self::JsObjectAssignmentPattern(node) => node.r_curly_token(),
            Self::JsObjectBindingPattern(node) => node.r_curly_token(),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Self::JsObjectAssignmentPattern(node) => node.properties().is_empty(),
            Self::JsObjectBindingPattern(node) => node.properties().is_empty(),
        }
    }

    fn write_properties(&self, f: &mut JsFormatter) -> FormatResult<()> {
        match self {
            Self::JsObjectAssignmentPattern(node) => {
                write!(f, [node.properties().format()])
            }
            Self::JsObjectBindingPattern(node) => {
                write!(f, [node.properties().format()])
            }
        }
    }

    fn is_inline(&self, comments: &JsComments) -> FormatResult<bool> {
        let parent_kind = self.syntax().parent().kind();

        Ok(
            (matches!(parent_kind, Some(JsSyntaxKind::JS_FORMAL_PARAMETER))
                || self.is_hug_parameter(comments))
                && !self.l_curly_token()?.leading_trivia().has_skipped(),
        )
    }

    fn should_break_properties(&self) -> bool {
        let parent_kind = self.syntax().parent().kind();

        // Catch only has a single expression in the declaration, so it will
        // be the direct parent of the object pattern, and the pattern should
        // not break unless it has to there.
        //
        // Parameters in function-like expressions are also kept inline, and
        // all parameters end up wrapped by a `JsFormalParameter` node, as
        // checked here. Note that this is also checked ahead of time by the
        // `is_inline` function.
        if matches!(
            parent_kind,
            Some(JsSyntaxKind::JS_CATCH_DECLARATION | JsSyntaxKind::JS_FORMAL_PARAMETER)
        ) {
            return false;
        }

        match self {
            Self::JsObjectAssignmentPattern(node) => node.properties().iter().any(|property| {
                if let Ok(AnyJsObjectAssignmentPatternMember::JsObjectAssignmentPatternProperty(
                    node,
                )) = property
                {
                    let pattern = node.pattern();
                    matches!(
                        pattern,
                        Ok(AnyJsAssignmentPattern::JsObjectAssignmentPattern(_)
                            | AnyJsAssignmentPattern::JsArrayAssignmentPattern(_))
                    )
                } else {
                    false
                }
            }),
            Self::JsObjectBindingPattern(node) => {
                node.properties().iter().any(|property| {
                    if let Ok(AnyJsObjectBindingPatternMember::JsObjectBindingPatternProperty(
                        node,
                    )) = property
                    {
                        let pattern = node.pattern();

                        // In Prettier, object patterns with nested object patterns are always
                        // expanded. However, it is not the case if the nested object pattern is in
                        // an assignment pattern. Prettier's AST can have an AssignmentPattern as a
                        // child of an ObjectPattern, while Biome's have an initializer **on** an
                        // object pattern. Check an initializer is set to ensure it does not have an
                        // assignment.
                        // https://github.com/prettier/prettier/blob/2d6877fcd1b78f2624e22d0ddb17a895ab12ac07/src/language-js/print/object.js#L81-L95
                        let is_assignment_pattern = node.init().is_some();

                        matches!(
                            pattern,
                            Ok(AnyJsBindingPattern::JsObjectBindingPattern(_)
                                | AnyJsBindingPattern::JsArrayBindingPattern(_))
                        ) && !is_assignment_pattern
                    } else {
                        false
                    }
                })
            }
        }
    }

    fn is_in_assignment_like(&self) -> bool {
        matches!(
            self.syntax().parent().kind(),
            Some(JsSyntaxKind::JS_ASSIGNMENT_EXPRESSION | JsSyntaxKind::JS_VARIABLE_DECLARATOR),
        )
    }

    fn is_hug_parameter(&self, comments: &JsComments) -> bool {
        match self {
            Self::JsObjectAssignmentPattern(_) => false,
            Self::JsObjectBindingPattern(binding) => binding
                .parent::<AnyJsFormalParameter>()
                .and_then(|parameter| parameter.syntax().grand_parent())
                .and_then(FormatAnyJsParameters::cast)
                .is_some_and(|parameters| {
                    should_hug_function_parameters(&parameters, comments, false).unwrap_or(false)
                }),
        }
    }

    fn layout(&self, comments: &JsComments) -> FormatResult<ObjectPatternLayout> {
        if self.is_empty() {
            return Ok(ObjectPatternLayout::Empty);
        }

        if self.is_inline(comments)? {
            return Ok(ObjectPatternLayout::Inline);
        }

        let break_properties = self.should_break_properties();

        let result = if break_properties {
            ObjectPatternLayout::Group { expand: true }
        } else if self.is_in_assignment_like() {
            ObjectPatternLayout::Inline
        } else {
            ObjectPatternLayout::Group { expand: false }
        };

        Ok(result)
    }
}

impl Format<JsFormatContext> for JsObjectPatternLike {
    fn fmt(&self, f: &mut Formatter<JsFormatContext>) -> FormatResult<()> {
        let should_insert_space_around_brackets = f.options().bracket_spacing().value();
        let format_properties = format_with(|f| {
            write!(
                f,
                [soft_block_indent_with_maybe_space(
                    &format_with(|f| self.write_properties(f)),
                    should_insert_space_around_brackets
                )]
            )
        });

        write!(f, [self.l_curly_token().format()])?;

        match self.layout(f.comments())? {
            ObjectPatternLayout::Empty => {
                write!(
                    f,
                    [format_dangling_comments(self.syntax()).with_soft_block_indent()]
                )?;
            }
            ObjectPatternLayout::Inline => {
                write!(f, [format_properties])?;
            }
            ObjectPatternLayout::Group { expand } => {
                write!(f, [group(&format_properties).should_expand(expand)])?;
            }
        }

        write!(f, [self.r_curly_token().format()])
    }
}

#[derive(Copy, Clone, Debug)]
enum ObjectPatternLayout {
    /// Wrap the properties in a group with [`should_expand`](Group::should_expand) equal to `expand`.
    ///
    /// This is the default layout when no other special case applies.
    Group { expand: bool },

    /// Layout for a pattern without any properties.
    Empty,

    /// Don't wrap the properties in a group and instead "inline" them in the parent.
    ///
    /// Desired if the pattern is a parameter of a function that [should hug](should_hug_function_parameters) OR
    /// if the pattern is the left side of an assignment.
    Inline,
}
