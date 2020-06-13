use crate::commands::WholeStreamCommand;
use crate::evaluate::evaluate_baseline_expr;
use crate::prelude::*;
use log::trace;
use nu_errors::ShellError;
use nu_protocol::{hir::ClassifiedCommand, Signature, SyntaxShape, UntaggedValue, Value};

pub struct SkipWhile;

#[async_trait]
impl WholeStreamCommand for SkipWhile {
    fn name(&self) -> &str {
        "skip-while"
    }

    fn signature(&self) -> Signature {
        Signature::build("skip-while")
            .required(
                "condition",
                SyntaxShape::Math,
                "the condition that must be met to continue skipping",
            )
            .filter()
    }

    fn usage(&self) -> &str {
        "Skips rows while the condition matches."
    }

    async fn run(
        &self,
        args: CommandArgs,
        registry: &CommandRegistry,
    ) -> Result<OutputStream, ShellError> {
        let registry = Arc::new(registry.clone());
        let scope = Arc::new(args.call_info.scope.clone());
        let call_info = args.evaluate_once(&registry).await?;

        let block = call_info.args.expect_nth(0)?.clone();

        let condition = Arc::new(match block {
            Value {
                value: UntaggedValue::Block(block),
                tag,
            } => {
                if block.block.len() != 1 {
                    return Err(ShellError::labeled_error(
                        "Expected a condition",
                        "expected a condition",
                        tag,
                    ));
                }
                match block.block[0].list.get(0) {
                    Some(item) => match item {
                        ClassifiedCommand::Expr(expr) => expr.clone(),
                        _ => {
                            return Err(ShellError::labeled_error(
                                "Expected a condition",
                                "expected a condition",
                                tag,
                            ));
                        }
                    },
                    None => {
                        return Err(ShellError::labeled_error(
                            "Expected a condition",
                            "expected a condition",
                            tag,
                        ));
                    }
                }
            }
            Value { tag, .. } => {
                return Err(ShellError::labeled_error(
                    "Expected a condition",
                    "expected a condition",
                    tag,
                ));
            }
        });

        Ok(call_info
            .input
            .skip_while(move |item| {
                let item = item.clone();
                let condition = condition.clone();
                let registry = registry.clone();
                let scope = scope.clone();
                trace!("ITEM = {:?}", item);

                async move {
                    let result = evaluate_baseline_expr(
                        &*condition,
                        &registry,
                        &item,
                        &scope.vars,
                        &scope.env,
                    )
                    .await;
                    trace!("RESULT = {:?}", result);

                    match result {
                        Ok(ref v) if v.is_true() => true,
                        _ => false,
                    }
                }
            })
            .to_output_stream())
    }
}

#[cfg(test)]
mod tests {
    use super::SkipWhile;

    #[test]
    fn examples_work_as_expected() {
        use crate::examples::test as test_examples;

        test_examples(SkipWhile {})
    }
}
