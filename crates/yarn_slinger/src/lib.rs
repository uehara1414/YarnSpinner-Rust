pub use log;

pub mod prelude {
    pub use crate::compiler::{
        Compilation, CompilationType, Compiler as YarnCompiler, CompilerError, File as YarnFile,
        LineInfo, Result as YarnCompilerResult, StringInfo,
    };
    pub use crate::core::{
        yarn_fn_registry, IntoYarnValueFromNonYarnValue, Library as YarnFnLibrary, LineId,
        Program as YarnProgram, YarnFn, YarnValue,
    };
    pub use crate::runtime::{
        Command as YarnCommand, CompiledProgramAnalyser as YarnAnalyser,
        Context as YarnAnalysisContext, Dialogue, DialogueError, DialogueEvent, DialogueOption,
        Language, Line as YarnLine, MarkupAttribute, MarkupValue, OptionId,
        Result as YarnRuntimeResult, StringTable, TextProvider, UnsupportedLanguageError,
        VariableStorage,
    };
}

pub mod core {
    pub use yarn_slinger_core::prelude::{
        yarn_fn_registry, yarn_fn_type, Header, Instruction, IntoYarnValueFromNonYarnValue,
        InvalidOpCodeError, Library, LineId, Node, Position, Program, Type, UntypedYarnFn, YarnFn,
        YarnFnParam, YarnFnParamItem, YarnValue, YarnValueCastError, YarnValueWrapper,
        YarnValueWrapperIter,
    };
}
pub mod compiler {
    pub use yarn_slinger_compiler::prelude::*;
    pub use yarn_slinger_compiler::Result;
}

pub mod runtime {
    pub use yarn_slinger_runtime::markup::{
        MarkupAttribute, MarkupParseError, MarkupValue, CHARACTER_ATTRIBUTE,
        CHARACTER_ATTRIBUTE_NAME_PROPERTY, TRIM_WHITESPACE_PROPERTY,
    };
    pub use yarn_slinger_runtime::prelude::*;
    pub use yarn_slinger_runtime::Result;
}
