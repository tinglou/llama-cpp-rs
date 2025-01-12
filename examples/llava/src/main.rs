//! llava-cli demo

use std::ffi::c_int;
use std::io::Write;
use std::num::NonZeroU32;
use std::str::FromStr;
use std::{ffi::CString, pin::pin};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use llama_cpp_2::model::Special;
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::{
    context::{params::LlamaContextParams, LlamaContext},
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    llava3::{ClipCtx, LlavaImageEmbed},
    model::{
        params::{kv_overrides::ParamOverrideValue, LlamaModelParams},
        AddBos, LlamaModel,
    },
    token::LlamaToken,
};

#[derive(Parser, Debug, Clone)]
#[command(
    version,
    about = "llava cli demo",
    long_about = "llava cli demo for Rust"
)]
struct Args {
    /// llama model
    #[arg(short, long)]
    model: String,

    /// CLIP(Contrastive Language–Image Pre-training) model
    #[arg(long)]
    mmproj: String,

    /// temperature. Note: a lower temperature value like 0.1 is recommended for better quality.
    #[arg(short, long, default_value_t = 0.1)]
    temperature: f32,

    /// path to image
    #[arg(short, long)]
    image: String,

    /// path to image
    #[arg(short, long, default_value = "describe the image in detail.")]
    prompt: String,

    /// override some parameters of the model, e.g. key=value
    #[arg(short = 'o', value_parser = parse_key_val)]
    key_value_overrides: Vec<(String, ParamOverrideValue)>,

    /// Disable offloading layers to the gpu
    #[cfg(any(feature = "cuda", feature = "vulkan"))]
    #[clap(long)]
    disable_gpu: bool,
}

/// Parse a single key-value pair
fn parse_key_val(s: &str) -> Result<(String, ParamOverrideValue)> {
    let pos = s
        .find('=')
        .ok_or_else(|| anyhow!("invalid KEY=value: no `=` found in `{}`", s))?;
    let key = s[..pos].parse()?;
    let value: String = s[pos + 1..].parse()?;
    let value = i64::from_str(&value)
        .map(ParamOverrideValue::Int)
        .or_else(|_| f64::from_str(&value).map(ParamOverrideValue::Float))
        .or_else(|_| bool::from_str(&value).map(ParamOverrideValue::Bool))
        .map_err(|_| anyhow!("must be one of i64, f64, or bool"))?;

    Ok((key, value))
}

struct LlavaContext<'a> {
    ctx_clip: ClipCtx,
    ctx_llama: LlamaContext<'a>,
    _model: &'a LlamaModel,
}

fn llava_init<'a>(backend: &LlamaBackend, args: &'a Args) -> Result<LlamaModel> {
    // offload all layers to the gpu
    let model_params = {
        #[cfg(any(feature = "cuda", feature = "vulkan"))]
        if !disable_gpu {
            LlamaModelParams::default().with_n_gpu_layers(1000)
        } else {
            LlamaModelParams::default()
        }
        #[cfg(not(any(feature = "cuda", feature = "vulkan")))]
        LlamaModelParams::default()
    };

    let mut model_params = pin!(model_params);
    for (k, v) in &args.key_value_overrides {
        let k = CString::new(k.as_bytes()).with_context(|| format!("invalid key: {k}"))?;
        model_params.as_mut().append_kv_override(k.as_c_str(), *v);
    }

    let llama_model_path = &args.model;
    let llama_model = LlamaModel::load_from_file(backend, llama_model_path, &model_params)
        .with_context(|| "unable to load model")?;
    Ok(llama_model)
}

fn llava_init_context<'a>(
    backend: &LlamaBackend,
    args: &'a Args,
    model: &'a LlamaModel,
) -> Result<LlavaContext<'a>> {
    let clip_path = &args.mmproj;
    // locad clip model
    let ctx_clip = ClipCtx::load_from_file(&clip_path, 1)?;

    // initialize the context
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(2048))
        .with_n_threads(16)
        .with_n_threads_batch(16);

    let ctx_llama = model
        .new_context(&backend, ctx_params)
        .with_context(|| "unable to create the llama_context")?;

    Ok(LlavaContext {
        ctx_clip,
        ctx_llama: ctx_llama,
        _model: model,
    })
}

fn load_image(ctx_llava: &mut LlavaContext, args: &Args) -> Result<LlavaImageEmbed> {
    let embed = LlavaImageEmbed::make_with_file(&mut ctx_llava.ctx_clip, 4, &args.image)?;

    Ok(embed)
}

fn process_prompt(
    ctx_llava: &mut LlavaContext,
    image_embed: &LlavaImageEmbed,
    _args: &Args,
    prompt: &str,
) -> Result<()> {
    let system_prompt = "A chat between a curious human and an artificial intelligence assistant. The assistant gives helpful, detailed, and polite answers to the human's questions.\nUSER:";
    let user_prompt = format!("{}\nASSISTANT:", prompt);

    let mut n_past: c_int = 0;
    let n_batch = 2048; // logical batch size for prompt processing (must be >=32 to use BLAS)

    // eval system prompt
    eprintln!("evaluating system prompt...");
    eval_string(
        &mut ctx_llava.ctx_llama,
        system_prompt,
        n_batch,
        &mut n_past,
        AddBos::Always,
    )?;

    // eval image
    eprintln!("evaluating image...");
    image_embed.eval(&mut ctx_llava.ctx_llama, n_batch, &mut n_past);

    // eval user prompt
    eprintln!("evaluating user prompt...");
    let logit = eval_string(
        &mut ctx_llava.ctx_llama,
        &user_prompt,
        n_batch,
        &mut n_past,
        AddBos::Never,
    )?;

    // generate the response
    eprintln!();
    let max_tgt_len: c_int = 256;
    generate(&mut ctx_llava.ctx_llama, n_past, logit, max_tgt_len)?;

    println!();

    Ok(())
}

fn generate(
    llama_ctx: &mut LlamaContext<'_>,
    mut _n_cur: i32,
    logit: i32,
    n_len: i32,
) -> Result<(), anyhow::Error> {
    let mut decoder = encoding_rs::UTF_8.new_decoder();

    let mut sampler =
        LlamaSampler::chain_simple([LlamaSampler::dist(1234), LlamaSampler::greedy()]);

    let mut first = true;
    for _i in 0..n_len {
        // sample the next token

        let token_pos = if first {
            first = false;
            logit - 1
        } else {
            0
        };

        let new_token_id = sampler.sample(&llama_ctx, token_pos);

        sampler.accept(new_token_id);

        // let candidates = llama_ctx.candidates_ith(token_pos);
        // let candidates_p = LlamaTokenDataArray::from_iter(candidates, false);

        // // sample the most likely token
        // let new_token_id = llama_ctx.sample_token_greedy(candidates_p);

        // is it an end of stream?
        if new_token_id == llama_ctx.model.token_eos() {
            eprintln!();
            break;
        }

        let output_bytes = llama_ctx
            .model
            .token_to_bytes(new_token_id, Special::Tokenize)?;
        // use `Decoder.decode_to_string()` to avoid the intermediate buffer
        let mut output_string = String::with_capacity(32);
        let _decode_result = decoder.decode_to_string(&output_bytes, &mut output_string, false);
        print!("{output_string}");
        std::io::stdout().flush()?;

        // let mut batch = LlamaBatch::get_one(&[new_token_id], n_cur, 0)?;
        let mut batch = LlamaBatch::get_one(&[new_token_id])?;
        // batch.clear();
        // batch.add(new_token_id, n_cur, &[0], true)?;

        llama_ctx
            .decode(&mut batch)
            .with_context(|| "failed to eval")?;

        _n_cur += 1;
        // n_decode += 1;
    }
    Ok(())
}

fn eval_string(
    ctx_llama: &mut LlamaContext,
    str: &str,
    n_batch: i32,
    n_past: &mut i32,
    add_bos: AddBos,
) -> Result<i32> {
    let embd_inp = ctx_llama.model.str_to_token(str, add_bos)?;

    eval_token(ctx_llama, &embd_inp, n_batch, n_past)
}

fn eval_token(
    ctx_llama: &mut LlamaContext,
    tokens: &[LlamaToken],
    n_batch: c_int,
    n_past: &mut i32,
) -> Result<i32> {
    let n = tokens.len() as c_int;
    let mut i: c_int = 0;
    let mut n_eval = 0;
    while i < n {
        n_eval = (tokens.len() as c_int) - i;
        if n_eval > n_batch {
            n_eval = n_batch;
        }

        let tokens_batch = &tokens[i as usize..(i + n_eval) as usize];
        // let mut batch = LlamaBatch::get_one(&tokens_batch, *n_past, 0);
        let mut batch = LlamaBatch::get_one(&tokens_batch)?;
        ctx_llama.decode(&mut batch).with_context(|| {
            format!(
                "failed to eval. token {}/{} (batch size {}, n_past {})",
                i, n, n_batch, n_past,
            )
        })?;

        *n_past += n_eval;
        i += n_batch;
    }

    Ok(n_eval)
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("args: {:#?}", args);

    // init LLM
    let backend = LlamaBackend::init()?;
    let llama_model = llava_init(&backend, &args)?;
    let mut ctx_llava = llava_init_context(&backend, &args, &llama_model)?;

    let image_embed = load_image(&mut ctx_llava, &args)?;

    process_prompt(&mut ctx_llava, &image_embed, &args, &args.prompt)?;

    println!();
    println!("{}", ctx_llava.ctx_llama.timings());
    println!();

    Ok(())
}
