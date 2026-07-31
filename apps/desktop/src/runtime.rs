//! Closed backend, renderer, and process-startup boundary for the native shell.

use std::ffi::OsString;
use std::fmt;

use slint::{BackendSelector, ComponentHandle, PlatformError};

use crate::generated_ui::PartmanApp;

/// One renderer request permitted by ADR-0009.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Renderer {
    /// OpenGL-backed `FemtoVG` renderer.
    FemtoVg,
    /// CPU software renderer.
    Software,
}

impl Renderer {
    /// Stable Slint renderer name passed to the closed backend selector.
    #[must_use]
    pub const fn slint_name(self) -> &'static str {
        match self {
            Self::FemtoVg => "femtovg",
            Self::Software => "software",
        }
    }
}

/// Startup configuration was invalid or the native platform could not start.
#[derive(Debug)]
pub enum StartupError {
    /// Ambient Slint state was present.
    Environment(crate::slint_environment::ForbiddenEnvironmentName),
    /// Command-line renderer selection was not in the closed grammar.
    Arguments(&'static str),
    /// A renderer was requested that is absent from this exact feature graph.
    RendererUnavailable(Renderer),
    /// The checked synthetic shell fixture or catalogue was inconsistent.
    ViewModel(crate::view_model::ViewModelError),
    /// Slint could not create or run the selected native platform.
    Platform(PlatformError),
}

impl fmt::Display for StartupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Environment(error) => error.fmt(formatter),
            Self::Arguments(message) => formatter.write_str(message),
            Self::RendererUnavailable(renderer) => write!(
                formatter,
                "renderer {:?} is absent from this exact desktop feature graph",
                renderer.slint_name()
            ),
            Self::ViewModel(error) => write!(formatter, "native shell model failed: {error}"),
            Self::Platform(error) => write!(formatter, "native UI startup failed: {error}"),
        }
    }
}

impl std::error::Error for StartupError {}

impl From<crate::slint_environment::ForbiddenEnvironmentName> for StartupError {
    fn from(error: crate::slint_environment::ForbiddenEnvironmentName) -> Self {
        Self::Environment(error)
    }
}

impl From<PlatformError> for StartupError {
    fn from(error: PlatformError) -> Self {
        Self::Platform(error)
    }
}

impl From<crate::view_model::ViewModelError> for StartupError {
    fn from(error: crate::view_model::ViewModelError) -> Self {
        Self::ViewModel(error)
    }
}

/// Parse the process arguments without consulting environment state.
///
/// Single-renderer artifacts need no argument and reject attempts to request
/// another renderer. The deliberately non-shipping combined control requires
/// an explicit `--renderer femtovg|software` pair for each fresh process.
///
/// # Errors
///
/// Rejects non-Unicode values, unknown options or renderer names, extra
/// arguments, and a request absent from the compiled feature graph.
pub fn renderer_from_arguments(arguments: &[OsString]) -> Result<Renderer, StartupError> {
    let requested = match arguments {
        [] if !cfg!(feature = "comparison-combined") => compiled_single_renderer()?,
        [option, value] if option == "--renderer" => {
            let value = value
                .to_str()
                .ok_or(StartupError::Arguments("renderer name is not Unicode"))?;
            match value {
                "femtovg" => Renderer::FemtoVg,
                "software" => Renderer::Software,
                _ => {
                    return Err(StartupError::Arguments(
                        "renderer must be exactly femtovg or software",
                    ));
                }
            }
        }
        [] => {
            return Err(StartupError::Arguments(
                "comparison-combined requires --renderer femtovg|software",
            ));
        }
        _ => {
            return Err(StartupError::Arguments(
                "usage: partman-desktop [--renderer femtovg|software]",
            ));
        }
    };
    if renderer_is_compiled(requested) {
        Ok(requested)
    } else {
        Err(StartupError::RendererUnavailable(requested))
    }
}

/// Install exact Winit/renderer selection, create the AOT component, and run.
///
/// # Errors
///
/// Fails before constructing Slint when ambient state is present, when the
/// request is absent from this artifact, or when platform startup/run fails.
pub fn run(renderer: Renderer) -> Result<(), StartupError> {
    crate::slint_environment::guard_current_environment()?;
    if !renderer_is_compiled(renderer) {
        return Err(StartupError::RendererUnavailable(renderer));
    }
    BackendSelector::new()
        .backend_name("winit".to_owned())
        .renderer_name(renderer.slint_name().to_owned())
        .select()?;

    let application = PartmanApp::new()?;
    crate::view_model::bind(&application)?;
    application.run()?;
    Ok(())
}

fn renderer_is_compiled(renderer: Renderer) -> bool {
    match renderer {
        Renderer::FemtoVg => cfg!(feature = "renderer-femtovg"),
        Renderer::Software => cfg!(feature = "renderer-software"),
    }
}

fn compiled_single_renderer() -> Result<Renderer, StartupError> {
    match (
        cfg!(feature = "renderer-femtovg"),
        cfg!(feature = "renderer-software"),
    ) {
        (true, false) => Ok(Renderer::FemtoVg),
        (false, true) => Ok(Renderer::Software),
        _ => Err(StartupError::Arguments(
            "this artifact does not contain exactly one shipping renderer",
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::{Renderer, renderer_from_arguments};

    // Requirements: SAFE-004, SEC-010
    //   Renderer selection is a closed argument grammar and never accepts a
    //   backend name or an uncompiled renderer.
    // Work-Package: WP-030
    // Evidence: renderer_arguments_are_closed_and_feature_bound
    #[test]
    fn renderer_arguments_are_closed_and_feature_bound() {
        for rejected in [
            vec!["--renderer", "skia"],
            vec!["--backend", "qt"],
            vec!["--renderer", "femtovg", "extra"],
        ] {
            assert!(
                renderer_from_arguments(
                    &rejected.into_iter().map(OsString::from).collect::<Vec<_>>(),
                )
                .is_err()
            );
        }

        #[cfg(feature = "renderer-femtovg")]
        assert_eq!(
            renderer_from_arguments(&[OsString::from("--renderer"), OsString::from("femtovg")])
                .expect("compiled FemtoVG request passes"),
            Renderer::FemtoVg
        );
        #[cfg(feature = "renderer-software")]
        assert_eq!(
            renderer_from_arguments(&[OsString::from("--renderer"), OsString::from("software")])
                .expect("compiled software request passes"),
            Renderer::Software
        );
    }
}
