use swc_common::{comments::SingleThreadedComments, Mark};
use swc_core::common as swc_common;
use swc_core::common::pass::Optional;
use swc_core::ecma::transforms::base::helpers::{self, HELPERS};
use swc_core::ecma::transforms::base::{
  fixer::{self, paren_remover},
  helpers::inject_helpers,
  hygiene::hygiene,
};
use swc_core::ecma::visit::as_folder;
use swc_core::ecma::{
  ast,
  transforms::{base::resolver, module::common_js},
  visit::FoldWith,
};

use crate::default_export_mode_shimer;

pub fn to_cjs(
  ast: ast::Module,
  unresolved_mark: Mark,
  comments: &SingleThreadedComments,
  shim_default_export: bool,
) -> ast::Module {
  HELPERS.set(&helpers::Helpers::new(false), || {
    ast
      .fold_with(&mut paren_remover(Some(comments)))
      .fold_with(&mut resolver(unresolved_mark, Mark::new(), false))
      .fold_with(&mut common_js::common_js::<SingleThreadedComments>(
        unresolved_mark,
        common_js::Config {
          ..Default::default()
        },
        Default::default(),
        Default::default(),
      ))
      .fold_with(&mut hygiene())
      .fold_with(&mut fixer::fixer(Some(comments)))
      .fold_with(&mut inject_helpers(unresolved_mark))
      .fold_with(&mut Optional {
        visitor: as_folder(default_export_mode_shimer()),
        enabled: shim_default_export,
      })
  })
}
