use crate::types::generator::{GenerateContext, GenerateOutput, Generator};

use anyhow::Result;
use rolldown_common::{InstantiatedChunk, InstantiationKind};
use rolldown_error::BuildResult;

pub struct CssGenerator;

impl Generator for CssGenerator {
  #[allow(clippy::too_many_lines)]
  async fn instantiate_chunk<'a>(
    ctx: &mut GenerateContext<'a>,
  ) -> Result<BuildResult<GenerateOutput>> {
    let mut ordered_css_modules = ctx
      .chunk
      .modules
      .iter()
      .filter_map(|&id| ctx.link_output.module_table.modules[id].as_normal())
      .filter(|m| m.css_view.is_some())
      .collect::<Vec<_>>();

    if ordered_css_modules.is_empty() {
      return Ok(Ok(GenerateOutput {
        chunks: vec![],
        warnings: std::mem::take(&mut ctx.warnings),
      }));
    }

    ordered_css_modules.sort_by_key(|m| m.exec_order);

    let mut content = String::new();

    for module in &ordered_css_modules {
      let css_view = module.css_view.as_ref().unwrap();
      let mut magic_string = string_wizard::MagicString::new(&css_view.source);
      for mutation in &css_view.mutations {
        mutation.apply(&mut magic_string);
      }
      content.push_str(&magic_string.to_string());
      content.push('\n');
    }

    // Here file path is generated by chunk file name template, it maybe including path segments.
    // So here need to read it's parent directory as file_dir.
    let file_path = ctx.options.cwd.as_path().join(&ctx.options.dir).join(
      ctx
        .chunk
        .css_preliminary_filename
        .as_deref()
        .expect("chunk file name should be generated before rendering")
        .as_str(),
    );
    let file_dir = file_path.parent().expect("chunk file name should have a parent");

    Ok(Ok(GenerateOutput {
      chunks: vec![InstantiatedChunk {
        origin_chunk: ctx.chunk_idx,
        content,
        map: None,
        meta: InstantiationKind::None,
        augment_chunk_hash: None,
        file_dir: file_dir.to_path_buf(),
        preliminary_filename: ctx
          .chunk
          .css_preliminary_filename
          .clone()
          .expect("should have preliminary filename"),
      }],
      warnings: std::mem::take(&mut ctx.warnings),
    }))
  }
}
