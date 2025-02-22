use ast::{
  ClassDecl, ClassExpr, ExportDefaultDecl, ExprStmt, FnDecl, ModuleDecl, ModuleItem, ParenExpr,
};
use swc_common::{util::take::Take, DUMMY_SP};
use swc_core::{
  common as swc_common,
  ecma::{ast, visit as swc_ecma_visit},
};
use swc_ecma_visit::{VisitMut, VisitMutWith};

struct ExportRemover;

pub fn remove_export_and_import(ast: &mut ast::Module) {
  ast.visit_mut_with(&mut ExportRemover);
}

impl ExportRemover {
  fn rewrite_default_export_decl_to_stmt(&self, export_decl: ExportDefaultDecl) -> ModuleItem {
    match export_decl.decl {
      ast::DefaultDecl::Class(cls_decl) => {
        if let Some(ident) = cls_decl.ident {
          ModuleItem::Stmt(ast::Stmt::Decl(ast::Decl::Class(ClassDecl {
            ident,
            declare: false,
            class: cls_decl.class,
          })))
        } else {
          ModuleItem::Stmt(ast::Stmt::Expr(ExprStmt {
            span: DUMMY_SP,
            expr: Box::new(ast::Expr::Paren(ParenExpr {
              span: DUMMY_SP,
              expr: Box::new(ast::Expr::Class(ClassExpr {
                ident: None,
                class: cls_decl.class,
              })),
            })),
          }))
        }
      }
      ast::DefaultDecl::Fn(fn_decl) => {
        if let Some(ident) = fn_decl.ident {
          ModuleItem::Stmt(ast::Stmt::Decl(ast::Decl::Fn(FnDecl {
            ident,
            declare: false,
            function: fn_decl.function,
          })))
        } else {
          ModuleItem::Stmt(ast::Stmt::Expr(ExprStmt {
            span: DUMMY_SP,
            expr: Box::new(ast::Expr::Paren(ParenExpr {
              span: DUMMY_SP,
              expr: Box::new(ast::Expr::Fn(ast::FnExpr {
                ident: None,
                function: fn_decl.function,
              })),
            })),
          }))
        }
      }
      ast::DefaultDecl::TsInterfaceDecl(_) => unreachable!(),
    }
  }
}

impl VisitMut for ExportRemover {
  fn visit_mut_module_items(&mut self, items: &mut Vec<ast::ModuleItem>) {
    *items = items
      .take()
      .into_iter()
      .flat_map(|module_item| match module_item {
        ModuleItem::ModuleDecl(ast::ModuleDecl::ExportDecl(decl)) => match decl.decl {
          ast::Decl::Class(class) => {
            vec![ModuleItem::Stmt(ast::Stmt::Decl(ast::Decl::Class(class)))]
          }
          ast::Decl::Fn(fn_decl) => {
            vec![ModuleItem::Stmt(ast::Stmt::Decl(ast::Decl::Fn(fn_decl)))]
          }
          ast::Decl::Var(var) => {
            vec![ModuleItem::Stmt(ast::Stmt::Decl(ast::Decl::Var(var)))]
          }
          _ => unreachable!(),
        },
        ModuleItem::ModuleDecl(ast::ModuleDecl::ExportDefaultDecl(decl)) => {
          vec![self.rewrite_default_export_decl_to_stmt(decl)]
        }
        ModuleItem::ModuleDecl(ast::ModuleDecl::ExportDefaultExpr(default_expr)) => {
          // Remove `export default foo`
          if let box ast::Expr::Ident(_) = &default_expr.expr {
            // It's ok to remove the `export default [ident]` pattern
            // 1. It's doesn't have side-effect
            // 2. We will generate export stmts later,
            vec![]
          } else {
            // Could has side effect, could not simply remove
            vec![ModuleItem::Stmt(ast::Stmt::Expr(ExprStmt {
              span: DUMMY_SP,
              expr: default_expr.expr,
            }))]
          }
        }
        ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(_)) => vec![],
        ModuleItem::ModuleDecl(ModuleDecl::ExportAll(_)) => vec![],
        ModuleItem::ModuleDecl(ModuleDecl::Import(_)) => vec![],
        _ => vec![module_item],
      })
      .collect()
  }
}
