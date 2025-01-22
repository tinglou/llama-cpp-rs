use crate::context::params::LlamaContextParams;
use crate::context::LlamaContext;
use crate::model::params::LlamaModelParams;
use crate::model::LlamaModel;
use crate::sampling::LlamaSampler;

/// exgtend LlamaContext
impl<'model> LlamaContext<'model> {
    /// return raw const pointer
    pub fn as_ptr(&self) -> *const llama_cpp_sys_2::llama_context {
        self.context.as_ptr()
    }

    /// return raw mut pointer
    pub fn as_mut(&mut self) -> *mut llama_cpp_sys_2::llama_context {
        unsafe { self.context.as_mut() }
    }
}

/// extend LlamaModel
impl LlamaModel {
    /// return raw const pointer
    pub fn as_ptr(&self) -> *const llama_cpp_sys_2::llama_model {
        self.model.as_ptr()
    }

    /// return raw mut pointer
    pub fn as_mut(&mut self) -> *mut llama_cpp_sys_2::llama_model {
        unsafe { self.model.as_mut() }
    }
}

/// extend LlamaSampler
impl LlamaSampler {
    /// return raw const pointer
    pub fn as_ptr(&self) -> *const llama_cpp_sys_2::llama_sampler {
        self.sampler
    }

    /// return raw mut pointer
    pub fn as_mut(&mut self) -> *mut llama_cpp_sys_2::llama_sampler {
        self.sampler
    }
}

impl LlamaModelParams {
    /// return raw const pointer
    pub fn as_ptr(&self) -> *const llama_cpp_sys_2::llama_model_params {
        &self.params
    }

    /// return raw mut pointer
    pub fn as_mut(&mut self) -> *mut llama_cpp_sys_2::llama_model_params {
        &mut self.params
    }
}

impl LlamaContextParams {
    /// return raw const pointer
    pub fn as_ptr(&self) -> *const llama_cpp_sys_2::llama_context_params {
        &self.context_params
    }

    /// return raw mut pointer
    pub fn as_mut(&mut self) -> *mut llama_cpp_sys_2::llama_context_params {
        &mut self.context_params
    }

    /// sets the main GPU
    #[must_use]
    pub fn with_no_perf(mut self, no_perf: bool) -> Self {
        self.context_params.no_perf = no_perf;
        self
    }
}
