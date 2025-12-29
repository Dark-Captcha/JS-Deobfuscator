//! Configuration types for the deobfuscation engine.

// ============================================================================
// LogLevel
// ============================================================================

/// Log level for engine output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogLevel {
    /// No logging.
    Off,
    /// Errors only.
    Error,
    /// Warnings and errors.
    Warn,
    /// Informational messages.
    #[default]
    Info,
    /// Debug messages.
    Debug,
    /// Trace-level messages.
    Trace,
}

// ============================================================================
// EcmaConfig
// ============================================================================

/// Fine-grained ECMA layer configuration.
#[derive(Debug, Clone)]
pub struct EcmaConfig {
    /// Enable operator passes (+, -, *, /, etc.).
    pub operators: bool,
    /// Enable String.prototype method passes.
    pub string: bool,
    /// Enable Array.prototype method passes.
    pub array: bool,
    /// Enable Number method passes.
    pub number: bool,
    /// Enable Math method passes.
    pub math: bool,
    /// Enable Object method passes.
    pub object: bool,
    /// Enable JSON method passes.
    pub json: bool,
    /// Enable RegExp method passes.
    pub regexp: bool,
    /// Enable URI function passes.
    pub uri: bool,
    /// Enable type coercion passes (JSFuck patterns).
    pub coercion: bool,
    /// Enable syntax simplification passes.
    pub syntax: bool,
    /// Enable control flow simplification passes.
    pub control: bool,
    /// Enable dead code removal passes.
    pub dead: bool,
}

impl Default for EcmaConfig {
    fn default() -> Self {
        Self::all()
    }
}

impl EcmaConfig {
    /// Enable all ECMA passes.
    pub fn all() -> Self {
        Self {
            operators: true,
            string: true,
            array: true,
            number: true,
            math: true,
            object: true,
            json: true,
            regexp: true,
            uri: true,
            coercion: true,
            syntax: true,
            control: true,
            dead: true,
        }
    }

    /// Disable all ECMA passes.
    pub fn none() -> Self {
        Self {
            operators: false,
            string: false,
            array: false,
            number: false,
            math: false,
            object: false,
            json: false,
            regexp: false,
            uri: false,
            coercion: false,
            syntax: false,
            control: false,
            dead: false,
        }
    }
}

// ============================================================================
// RuntimeConfig
// ============================================================================

/// Fine-grained Runtime layer configuration.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Enable encoding passes (atob, btoa).
    pub encoding: bool,
    /// Enable deprecated API passes (escape, unescape).
    pub deprecated: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self::all()
    }
}

impl RuntimeConfig {
    /// Enable all Runtime passes.
    pub fn all() -> Self {
        Self {
            encoding: true,
            deprecated: true,
        }
    }

    /// Disable all Runtime passes.
    pub fn none() -> Self {
        Self {
            encoding: false,
            deprecated: false,
        }
    }
}

// ============================================================================
// ExtensionsConfig
// ============================================================================

/// Fine-grained Extensions layer configuration.
#[derive(Debug, Clone, Default)]
pub struct ExtensionsConfig {
    /// Enable string array decoder.
    pub string_array: bool,
    /// Enable control flow deflattener.
    pub control_flow: bool,
    /// Enable proxy function inliner.
    pub proxy: bool,
}

impl ExtensionsConfig {
    /// Enable all extension patterns.
    pub fn all() -> Self {
        Self {
            string_array: true,
            control_flow: true,
            proxy: true,
        }
    }

    /// Disable all extension patterns.
    pub fn none() -> Self {
        Self {
            string_array: false,
            control_flow: false,
            proxy: false,
        }
    }
}

// ============================================================================
// LayerConfig
// ============================================================================

/// Layer enable/disable configuration.
#[derive(Debug, Clone)]
pub struct LayerConfig {
    /// Enable ECMA layer (recommended: always true).
    pub ecma: bool,
    /// Enable Runtime layer (browser/Node APIs).
    pub runtime: bool,
    /// Enable Extensions layer (obfuscator patterns).
    pub extensions: bool,
    /// Fine-grained ECMA configuration.
    pub ecma_config: EcmaConfig,
    /// Fine-grained Runtime configuration.
    pub runtime_config: RuntimeConfig,
    /// Fine-grained Extensions configuration.
    pub extensions_config: ExtensionsConfig,
}

impl Default for LayerConfig {
    fn default() -> Self {
        Self {
            ecma: true,
            runtime: true,
            extensions: false, // Opt-in for obfuscator-specific patterns
            ecma_config: EcmaConfig::default(),
            runtime_config: RuntimeConfig::default(),
            extensions_config: ExtensionsConfig::default(),
        }
    }
}

impl LayerConfig {
    /// Create config with all layers enabled.
    pub fn all() -> Self {
        Self {
            ecma: true,
            runtime: true,
            extensions: true,
            ecma_config: EcmaConfig::all(),
            runtime_config: RuntimeConfig::all(),
            extensions_config: ExtensionsConfig::all(),
        }
    }

    /// Create config with only ECMA layer (pure ECMAScript).
    pub fn ecma_only() -> Self {
        Self {
            ecma: true,
            runtime: false,
            extensions: false,
            ecma_config: EcmaConfig::all(),
            runtime_config: RuntimeConfig::none(),
            extensions_config: ExtensionsConfig::none(),
        }
    }
}

// ============================================================================
// EngineConfig
// ============================================================================

/// Engine configuration.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Maximum iterations before stopping.
    pub max_iterations: usize,
    /// Layer configuration.
    pub layers: LayerConfig,
    /// Log level.
    pub log_level: LogLevel,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            layers: LayerConfig::default(),
            log_level: LogLevel::Info,
        }
    }
}

impl EngineConfig {
    /// Create minimal config (ECMA operators only).
    pub fn minimal() -> Self {
        Self {
            layers: LayerConfig {
                ecma: true,
                runtime: false,
                extensions: false,
                ecma_config: EcmaConfig {
                    operators: true,
                    ..EcmaConfig::none()
                },
                runtime_config: RuntimeConfig::none(),
                extensions_config: ExtensionsConfig::none(),
            },
            ..Default::default()
        }
    }

    /// Create standard config (ECMA + Runtime).
    pub fn standard() -> Self {
        Self::default()
    }

    /// Create full config (all layers enabled).
    pub fn full() -> Self {
        Self {
            layers: LayerConfig::all(),
            ..Default::default()
        }
    }

    /// Create aggressive config (full + high iterations).
    pub fn aggressive() -> Self {
        Self {
            max_iterations: 200,
            layers: LayerConfig::all(),
            ..Default::default()
        }
    }

    /// Set max iterations.
    #[must_use]
    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    /// Set layer configuration.
    #[must_use]
    pub fn with_layers(mut self, layers: LayerConfig) -> Self {
        self.layers = layers;
        self
    }

    /// Set log level.
    #[must_use]
    pub fn with_log_level(mut self, level: LogLevel) -> Self {
        self.log_level = level;
        self
    }
}
