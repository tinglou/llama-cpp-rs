use crate::context::LlamaContext;
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
