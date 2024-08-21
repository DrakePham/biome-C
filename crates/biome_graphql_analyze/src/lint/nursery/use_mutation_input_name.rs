use std::collections::HashSet;
use biome_analyze::{
    context::RuleContext, declare_lint_rule, Ast, Rule, RuleDiagnostic, RuleSource, RuleSourceKind,
};
use biome_console::markup;
use biome_graphql_syntax::{
    GraphqlFieldDefinition, GraphqlObjectTypeDefinition, GraphqlArguments,
};
use biome_rowan::{AstNode, TextRange};

declare_lint_rule! {
    /// Enforces naming conventions for mutation arguments and input types in GraphQL.
    ///
    /// Ensures that mutation arguments are named "input" and input types follow the "<MutationName>Input" naming convention.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```graphql,expect_diagnostic
    /// type Mutation {
    ///   SetMessage(message: InputMessage): String
    /// }
    /// ```
    ///
    /// ### Valid
    ///
    /// ```graphql
    /// type Mutation {
    ///   SetMessage(input: SetMessageInput): String
    /// }
    /// ```
    ///
    pub InputNameConvention {
        version: "next",
        name: "inputNameConvention",
        language: "graphql",
        sources: &[RuleSource::EslintGraphql("input-name")],
        source_kind: RuleSourceKind::SameLogic,
        recommended: true,
    }
}

impl Rule for InputNameConvention {
    type Query = Ast<GraphqlObjectTypeDefinition>;
    type State = NamingIssue;
    type Signals = Vec<Self::State>;
    type Options = InputNameConventionOptions;

    fn run(ctx: &RuleContext<Self>) -> Vec<Self::State> {
        let object_type = ctx.query();
        let mut issues = vec![];

        for field in object_type.fields() {
            if let Some(arguments) = field.arguments() {
                issues.extend(check_argument_naming(field, &arguments, &ctx.options()));
            }
        }

        issues
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let NamingIssue {
            text_range,
            message,
        } = state;
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                *text_range,
                markup! { { message } },
            )
            .note(markup! {
                "Consider renaming to follow the input name convention."
            }),
        )
    }
}

#[derive(Debug)]
pub struct NamingIssue {
    text_range: TextRange,
    message: String,
}

#[derive(Debug)]
pub struct InputNameConventionOptions {
    pub check_input_type: bool,
}

impl Default for InputNameConventionOptions {
    fn default() -> Self {
        Self {
            check_input_type: true,
        }
    }
}

fn check_argument_naming(
    field: &GraphqlFieldDefinition,
    arguments: &GraphqlArguments,
    options: &InputNameConventionOptions,
) -> Vec<NamingIssue> {
    let mut issues = vec![];

    for argument in arguments.arguments() {
        let argument_name = argument.name().text();

        if argument_name != "input" {
            issues.push(NamingIssue {
                text_range: argument.range(),
                message: format!("Argument '{}' should be named 'input'.", argument_name),
            });
        }

        if options.check_input_type {
            let expected_type_name = format!("{}Input", field.name().text());
            let argument_type_name = argument.type_().name().text();

            if argument_type_name != expected_type_name {
                issues.push(NamingIssue {
                    text_range: argument.range(),
                    message: format!(
                        "Input type '{}' should be named '{}'.",
                        argument_type_name, expected_type_name
                    ),
                });
            }
        }
    }

    issues
}
