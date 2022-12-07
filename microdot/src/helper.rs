use crate::parser::parse_line;
use crate::Command;
use microdot_core::command::GraphCommand;
use microdot_core::{Id, Label, Line};
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::Context;
use rustyline_derive::Helper;
use std::borrow::Cow::{self, Borrowed, Owned};

pub trait GetNodeLabel {
    fn get_node_label(&self, id: &Id) -> Option<Label>;
}

pub struct MicrodotLanguageCompleter<'a, GNL>
where
    GNL: GetNodeLabel,
{
    label_info: &'a GNL,
}

const ALLOW_COMPLETION: bool = true;

impl<'a, GNL> Completer for MicrodotLanguageCompleter<'a, GNL>
where
    GNL: GetNodeLabel,
{
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        _pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        fn no_options() -> Result<(usize, Vec<Pair>), ReadlineError> {
            Ok((0, vec![]))
        }

        if !ALLOW_COMPLETION {
            // feature-flagged off
            return no_options();
        }

        let parse_result = parse_line(Line::new(line));

        let id = match parse_result {
            Command::GraphCommand(GraphCommand::RenameNode { id, .. }) => id,
            Command::RenameNodeUnlabelled { id } => id,
            _ => return no_options(),
        };

        let label = match self.label_info.get_node_label(&id) {
            Some(label) => label,
            None => return no_options(),
        };

        let new_line = format!("r {} {}", id, label);

        let replacement = Pair {
            display: new_line.clone(),
            replacement: new_line,
        };
        Ok((0, vec![replacement]))
    }
}

impl<'a, GNL> MicrodotLanguageCompleter<'a, GNL>
where
    GNL: GetNodeLabel,
{
    fn new(label_info: &'a GNL) -> Self {
        Self { label_info }
    }
}

#[derive(Helper)]
pub struct MicrodotHelper<'a, GNL>
where
    GNL: GetNodeLabel,
{
    completer: MicrodotLanguageCompleter<'a, GNL>,
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
    hinter: HistoryHinter,
    colored_prompt: String,
}

impl<'a, GNL> MicrodotHelper<'a, GNL>
where
    GNL: GetNodeLabel,
{
    pub fn new(label_info: &'a GNL) -> Self {
        Self {
            completer: MicrodotLanguageCompleter::new(label_info),
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
            colored_prompt: ">> ".to_owned(),
            validator: MatchingBracketValidator::new(),
        }
    }
}

impl<'a, GNL> Completer for MicrodotHelper<'a, GNL>
where
    GNL: GetNodeLabel,
{
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        self.completer.complete(line, pos, ctx)
    }
}

impl<'a, GNL> Hinter for MicrodotHelper<'a, GNL>
where
    GNL: GetNodeLabel,
{
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl<'a, GNL> Highlighter for MicrodotHelper<'a, GNL>
where
    GNL: GetNodeLabel,
{
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

impl<'a, GNL> Validator for MicrodotHelper<'a, GNL>
where
    GNL: GetNodeLabel,
{
    fn validate(
        &self,
        ctx: &mut validate::ValidationContext,
    ) -> rustyline::Result<validate::ValidationResult> {
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}
