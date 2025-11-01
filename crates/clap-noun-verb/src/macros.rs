//! Convenience macros for composable CLI patterns

/// Helper macro to create a noun command
#[macro_export]
macro_rules! noun {
    ($name:expr, $about:expr, [$($verb:expr),* $(,)?]) => {
        {
            let verbs: Vec<Box<dyn $crate::VerbCommand>> = vec![
                $(Box::new($verb)),*
            ];

            struct NounImpl {
                verbs: Vec<Box<dyn $crate::VerbCommand>>,
            }

            impl $crate::NounCommand for NounImpl {
                fn name(&self) -> &'static str {
                    $name
                }

                fn about(&self) -> &'static str {
                    $about
                }

                fn verbs(&self) -> Vec<Box<dyn $crate::VerbCommand>> {
                    // Create new boxes for each verb since we can't clone Box<dyn VerbCommand>
                    vec![
                        $(Box::new($verb)),*
                    ]
                }

                fn sub_nouns(&self) -> Vec<Box<dyn $crate::NounCommand>> {
                    Vec::new()
                }
            }

            NounImpl {
                verbs,
            }
        }
    };

    // Support for nested nouns (compound commands)
    ($name:expr, $about:expr, { $($noun:expr),* $(,)? }) => {
        {
            let sub_nouns: Vec<Box<dyn $crate::NounCommand>> = vec![
                $(Box::new($noun)),*
            ];

            struct CompoundNounImpl;

            impl $crate::NounCommand for CompoundNounImpl {
                fn name(&self) -> &'static str {
                    $name
                }

                fn about(&self) -> &'static str {
                    $about
                }

                fn verbs(&self) -> Vec<Box<dyn $crate::VerbCommand>> {
                    Vec::new()
                }

                fn sub_nouns(&self) -> Vec<Box<dyn $crate::NounCommand>> {
                    // Create new boxes for each sub-noun since we can't clone Box<dyn NounCommand>
                    vec![
                        $(Box::new($noun)),*
                    ]
                }
            }

            impl $crate::CompoundNounCommand for CompoundNounImpl {}

            CompoundNounImpl
        }
    };
}

/// Helper macro to create a verb command
///
/// # Examples
///
/// ```rust,no_run
/// use clap_noun_verb::{verb, VerbArgs};
///
/// // Simple verb without arguments
/// let _status_verb = verb!("status", "Show status", |_args: &VerbArgs| {
///     println!("Status: running");
///     Ok::<(), clap_noun_verb::NounVerbError>(())
/// });
///
/// // Verb with arguments (using clap::Arg)
/// let _logs_verb = verb!("logs", "Show logs", |args: &VerbArgs| {
///     let service = args.get_one_str("service").unwrap();
///     let lines = args.get_one_opt::<usize>("lines").unwrap_or(50);
///     println!("Logs for {} ({} lines)", service, lines);
///     Ok::<(), clap_noun_verb::NounVerbError>(())
/// }, args: [
///     clap::Arg::new("service").required(true),
///     clap::Arg::new("lines").short('n').long("lines").default_value("50"),
/// ]);
/// ```
#[macro_export]
macro_rules! verb {
    // Verb without additional arguments
    ($name:expr, $about:expr, $handler:expr) => {
        $crate::verb!($name, $about, $handler, args: [])
    };

    // Verb with additional arguments
    ($name:expr, $about:expr, $handler:expr, args: [$($arg:expr),* $(,)?]) => {
        {
            struct VerbImpl<F> {
                handler: F,
                args: Vec<clap::Arg>,
            }

            impl<F> $crate::VerbCommand for VerbImpl<F>
            where
                F: Fn(&$crate::VerbArgs) -> $crate::Result<()> + Send + Sync,
            {
                fn name(&self) -> &'static str {
                    $name
                }

                fn about(&self) -> &'static str {
                    $about
                }

                fn run(&self, args: &$crate::VerbArgs) -> $crate::Result<()> {
                    (self.handler)(args)
                }

                fn build_command(&self) -> clap::Command {
                    let mut cmd = clap::Command::new(self.name()).about(self.about());
                    for arg in &self.args {
                        cmd = cmd.arg(arg.clone());
                    }
                    cmd
                }

                fn additional_args(&self) -> Vec<clap::Arg> {
                    self.args.clone()
                }
            }

            VerbImpl {
                handler: $handler,
                args: vec![$($arg),*],
            }
        }
    };
}

/// Helper macro to create a command group (noun with multiple verbs)
#[macro_export]
macro_rules! command_group {
    ($name:expr, $about:expr, [$($verb:expr),* $(,)?]) => {
        $crate::noun!($name, $about, [$($verb),*])
    };
}

/// Helper macro to create a command tree for hierarchical composition
#[macro_export]
macro_rules! command_tree {
    ($builder:expr => $($command:expr),* $(,)?) => {
        {
            $(
                $builder = $builder.noun($command);
            )*
            $builder
        }
    };
}

/// Helper macro for building CLI applications with a declarative syntax
///
/// # Example
///
/// ```rust
/// use clap_noun_verb::{app, noun, verb, VerbArgs, Result};
///
/// let cli = app! {
///     name: "myapp",
///     about: "My awesome CLI application",
///     commands: [
///         noun!("services", "Manage services", [
///             verb!("status", "Show status", |_args: &VerbArgs| {
///                 println!("Services are running");
///                 Ok(())
///             }),
///         ]),
///         noun!("collector", "Manage collector", [
///             verb!("up", "Start collector", |_args: &VerbArgs| {
///                 println!("Starting collector");
///                 Ok(())
///             }),
///         ]),
///     ]
/// };
/// ```
#[macro_export]
macro_rules! app {
    // Match without trailing comma
    (name: $name:expr, about: $about:expr, commands: [$($command:expr),*]) => {
        {
            let mut builder = $crate::CliBuilder::new()
                .name($name)
                .about($about);

            $(
                builder = builder.noun($command);
            )*

            builder
        }
    };
    // Match with trailing comma
    (name: $name:expr, about: $about:expr, commands: [$($command:expr),*,]) => {
        $crate::app!(name: $name, about: $about, commands: [$($command),*])
    };
}
