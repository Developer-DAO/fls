use crate::{context::Context, symbols};
use anyhow::Result;
use crossbeam::channel::Sender;
use lsp_server::Request;
use lsp_types::{Diagnostic, GotoDefinitionParams, ReferenceParams};
use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
    sync::{Arc, Condvar, Mutex},
    thread,
    num::NonZeroU64,
    fmt,
};
use url::Url;

/// Enabling/disabling the language server reporting readiness to support go-to-def and
/// go-to-references to the IDE.
pub const DEFS_AND_REFS_SUPPORT: bool = true;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Copy)]
enum RunnerState {
    Run,
    Wait,
    Quit,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Symbol(NonZeroU64);


impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

pub struct Symbolicator {}

pub struct Symbols {}

/// Data used during symbolication running and symbolication info updating
pub struct SymbolicatorRunner {
    mtx_cvar: Arc<(Mutex<RunnerState>, Condvar)>,
}

impl SymbolicatorRunner {
    /// Create a new idle runner (one that does not actually symbolicate)
    pub fn idle() -> Self {
        let mtx_cvar = Arc::new((Mutex::new(RunnerState::Wait), Condvar::new()));
        SymbolicatorRunner { mtx_cvar }
    }

    /// Create a new runner
    pub fn new(
        uri: &Url,
        symbols: Arc<Mutex<HashMap<String, Symbols>>>,
        sender: Sender<Result<BTreeMap<Symbol, Vec<Diagnostic>>>>,
    ) -> Self {
        let mtx_cvar = Arc::new((Mutex::new(RunnerState::Wait), Condvar::new()));
        let thread_mtx_cvar = mtx_cvar.clone();
        let pkg_path = uri.to_file_path().unwrap();

        thread::spawn(move || {
            let (mtx, cvar) = &*thread_mtx_cvar;
            // infinite loop to wait for symbolication requests
            loop {
                let get_symbols = {
                    // hold the lock only as long as it takes to get the data, rather than through
                    // the whole symbolication process (hence a separate scope here)
                    let mut symbolicate = mtx.lock().unwrap();
                    match *symbolicate {
                        RunnerState::Quit => break,
                        RunnerState::Run => {
                            *symbolicate = RunnerState::Wait;
                            true
                        }
                        RunnerState::Wait => {
                            // wait for next request
                            symbolicate = cvar.wait(symbolicate).unwrap();
                            match *symbolicate {
                                RunnerState::Quit => break,
                                RunnerState::Run => {
                                    *symbolicate = RunnerState::Wait;
                                    true
                                }
                                RunnerState::Wait => false,
                            }
                        }
                    }
                };
                if get_symbols {
                    eprintln!("symbolication started");
                    match Symbolicator::get_symbols(&pkg_path) {
                        Ok((symbols_opt, lsp_diagnostics)) => {
                            eprintln!("symbolication finished");
                            if let Some(new_symbols) = symbols_opt {
                                // replace symbols only if they have been actually recomputed,
                                // otherwise keep the old (possibly out-dated) symbolication info
                                let mut old_symbols = symbols.lock().unwrap();
                                *old_symbols
                                    .entry(pkg_path.as_path().display().to_string())
                                    .or_insert(symbols::Symbolicator::empty_symbols()) =
                                    new_symbols;
                            }
                            // set/reset (previous) diagnostics
                            if let Err(err) = sender.send(Ok(lsp_diagnostics)) {
                                eprintln!("could not pass diagnostics: {:?}", err);
                            }
                        }
                        Err(err) => {
                            eprintln!("symbolication failed: {:?}", err);
                            if let Err(err) = sender.send(Err(err)) {
                                eprintln!("could not compiler error: {:?}", err);
                            }
                        }
                    }
                }
            }
        });

        SymbolicatorRunner { mtx_cvar }
    }

    pub fn run(&self) {
        eprintln!("scheduling run");
        let (mtx, cvar) = &*self.mtx_cvar;
        let mut symbolicate = mtx.lock().unwrap();
        *symbolicate = RunnerState::Run;
        cvar.notify_one();
        eprintln!("scheduled run");
    }

    pub fn quit(&self) {
        let (mtx, cvar) = &*self.mtx_cvar;
        let mut symbolicate = mtx.lock().unwrap();
        *symbolicate = RunnerState::Quit;
        cvar.notify_one();
    }
}

impl Symbolicator {
    /// Main driver to get symbols for the whole package. Returned symbols is an option as only the
    /// correctly computed symbols should be a replacement for the old set - if symbols are not
    /// actually (re)computed and the diagnostics are returned, the old symbolic information should
    /// be retained even if it's getting out-of-date.
    pub fn get_symbols(
        pkg_path: &Path,
    ) -> Result<(Option<Symbols>, BTreeMap<Symbol, Vec<Diagnostic>>)> {
        eprintln!("pkg path: {:?}", pkg_path);
        let empty_symbols = Symbolicator::empty_symbols();
        let empty_diagnostics = BTreeMap::new();
        Ok((Some(empty_symbols), empty_diagnostics))
    }

    /// Get empty symbols
    pub fn empty_symbols() -> Symbols {
        Symbols {}
    }
}

/// Handles go-to-def request of the language server
pub fn on_go_to_def_request(context: &Context, request: &Request, symbols: &Symbols) {
    let parameters = serde_json::from_value::<GotoDefinitionParams>(request.params.clone())
        .expect("could not deserialize go-to-def request");

    let fpath = parameters
        .text_document_position_params
        .text_document
        .uri
        .path();
    let loc = parameters.text_document_position_params.position;
    let line = loc.line;
    let col = loc.character;
}

/// Handles go-to-references request of the language server
pub fn on_references_request(context: &Context, request: &Request, symbols: &Symbols) {
    let parameters = serde_json::from_value::<ReferenceParams>(request.params.clone())
        .expect("could not deserialize references request");

    let fpath = parameters.text_document_position.text_document.uri.path();
    let loc = parameters.text_document_position.position;
    let line = loc.line;
    let col = loc.character;
    let include_decl = parameters.context.include_declaration;
}
