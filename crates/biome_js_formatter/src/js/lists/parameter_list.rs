use crate::js::bindings::parameters::ParameterLayout;
use crate::prelude::*;

use crate::context::trailing_comma::FormatTrailingComma;
use biome_js_syntax::parameter_ext::{AnyJsParameterList, AnyParameter};
use biome_js_syntax::{AnyJsConstructorParameter, AnyJsParameter, JsParameterList};

#[derive(Debug, Clone, Default)]
pub(crate) struct FormatJsParameterList;

impl FormatRule<JsParameterList> for FormatJsParameterList {
    type Context = JsFormatContext;

    fn fmt(&self, node: &JsParameterList, f: &mut JsFormatter) -> FormatResult<()> {
        FormatJsAnyParameterList::with_layout(
            &AnyJsParameterList::from(node.clone()),
            ParameterLayout::Default,
        )
        .fmt(f)
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct FormatJsAnyParameterList<'a> {
    list: &'a AnyJsParameterList,
    layout: Option<ParameterLayout>,
}

impl<'a> FormatJsAnyParameterList<'a> {
    pub fn with_layout(list: &'a AnyJsParameterList, layout: ParameterLayout) -> Self {
        Self {
            list,
            layout: Some(layout),
        }
    }
}

impl Format<JsFormatContext> for FormatJsAnyParameterList<'_> {
    fn fmt(&self, f: &mut Formatter<JsFormatContext>) -> FormatResult<()> {
        match self.layout {
            None
            | Some(
                ParameterLayout::Default | ParameterLayout::Compact | ParameterLayout::NoParameters,
            ) => {
                // The trailing separator is disallowed if the last element in the list is a rest parameter
                let has_trailing_rest = match self.list.last() {
                    Some(elem) => matches!(
                        elem?,
                        AnyParameter::AnyJsParameter(AnyJsParameter::JsRestParameter(_))
                            | AnyParameter::AnyJsConstructorParameter(
                                AnyJsConstructorParameter::JsRestParameter(_)
                            )
                    ),
                    None => false,
                };

                let is_compact = matches!(self.layout, Some(ParameterLayout::Compact));

                let trailing_separator = if is_compact || has_trailing_rest {
                    TrailingSeparator::Disallowed
                } else {
                    FormatTrailingComma::All.trailing_separator(f.options())
                };

                if is_compact {
                    let mut joiner = f.join_nodes_with_space();
                    join_parameter_list(&mut joiner, &self.list, trailing_separator)?;
                    joiner.finish()
                } else {
                    let mut joiner = f.join_nodes_with_soft_line();
                    join_parameter_list(&mut joiner, &self.list, trailing_separator)?;
                    joiner.finish()
                }
            }
            Some(ParameterLayout::Hug) => {
                let mut join = f.join_with(space());

                match self.list {
                    AnyJsParameterList::JsParameterList(list) => join.entries(
                        list.format_separated(",")
                            .with_trailing_separator(TrailingSeparator::Omit),
                    ),
                    AnyJsParameterList::JsConstructorParameterList(list) => join.entries(
                        list.format_separated(",")
                            .with_trailing_separator(TrailingSeparator::Omit),
                    ),
                };

                join.finish()
            }
        }
    }
}

fn join_parameter_list<S>(
    joiner: &mut JoinNodesBuilder<'_, '_, S, JsFormatContext>,
    list: &AnyJsParameterList,
    trailing_separator: TrailingSeparator,
) -> FormatResult<()>
where
    S: Format<JsFormatContext>,
{
    match list {
        AnyJsParameterList::JsParameterList(list) => {
            let entries = list
                .format_separated(",")
                .with_trailing_separator(trailing_separator)
                .zip(list.iter());

            for (format_entry, node) in entries {
                joiner.entry(node?.syntax(), &format_entry);
            }
        }
        AnyJsParameterList::JsConstructorParameterList(list) => {
            let entries = list
                .format_separated(",")
                .with_trailing_separator(trailing_separator)
                .zip(list.iter());

            for (format_entry, node) in entries {
                joiner.entry(node?.syntax(), &format_entry);
            }
        }
    }

    Ok(())
}
