use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::Context;
use rustyline_derive::Helper;
use std::borrow::Cow::{self, Borrowed, Owned};

pub struct MicrodotLanguageCompleter {}

impl Completer for MicrodotLanguageCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        Ok((0, vec![]))
        // Ok((
        //     1,
        //     vec![Pair {
        //         display: "foo".to_string(),
        //         replacement: "foo".to_string(),
        //     }],
        // ))

        //        self.completer.complete(line, pos, ctx)
    }
}

impl Default for MicrodotLanguageCompleter {
    fn default() -> Self {
        Self {}
    }
}

impl MicrodotLanguageCompleter {
    fn new() -> Self {
        Default::default()
    }
}

#[derive(Helper)]
pub struct MicrodotHelper {
    completer: MicrodotLanguageCompleter,
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
    hinter: HistoryHinter,
    colored_prompt: String,
}

impl Default for MicrodotHelper {
    fn default() -> Self {
        Self {
            completer: MicrodotLanguageCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
            colored_prompt: ">> ".to_owned(),
            validator: MatchingBracketValidator::new(),
        }
    }
}

impl MicrodotHelper {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Completer for MicrodotHelper {
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

impl Hinter for MicrodotHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for MicrodotHelper {
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

impl Validator for MicrodotHelper {
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
