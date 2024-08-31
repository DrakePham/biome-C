use biome_analyze::{
    context::RuleContext, declare_lint_rule, Ast, Rule, RuleDiagnostic, RuleSource, RuleSourceKind,
};
use biome_console::markup;
use biome_graphql_syntax::{
    GraphqlObjectTypeDefinition, GraphqlFieldDefinition, GraphqlFieldsDefinition,
    GraphqlArgumentsDefinition, GraphqlInputValueDefinition,
};
use biome_rowan::AstNode;

declare_lint_rule! {
    /// Require mutation argument to be always called "input" and input type to be called Mutation name + "Input".
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
    pub UseMutationInputName {
        version: "next",
        name: "useMutationInputName",
        language: "graphql",
        sources: &[RuleSource::EslintGraphql("input-name")],
        source_kind: RuleSourceKind::SameLogic,
        recommended: true,
    }
}

impl Rule for UseMutationInputName {
    type Query = Ast<GraphqlObjectTypeDefinition>;
    type State = InvalidInputName;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<Self::State> {
        let object_type = ctx.query();
        let mut invalid_inputs = vec![];

        if let Some(object_name) = object_type.name().ok() {
            if object_name.text() == "Mutation" {
                if let Some(fields_def) = object_type.fields() {
                    for field in fields_def.fields() {
                        invalid_inputs.extend(check_field_arguments(&field));
                    }
                }
            }
        }

        invalid_inputs
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        Some(
            RuleDiagnostic::new(
                rule_category!(),
                state.text_range,
                markup! {
                    "The mutation argument `"{state.argument_name}"` should be named `input` and its type should match the mutation name with 'Input' suffix."
                },
            )
            .note(markup! {
                "Rename the argument and its type according to the convention."
            }),
        )
    }
}

#[derive(Debug)]
pub struct InvalidInputName {
    argument_name: String,
    text_range: biome_rowan::TextRange,
}

fn check_field_arguments(field: &GraphqlFieldDefinition) -> Vec<InvalidInputName> {
    let mut invalid_inputs = vec![];

    if let Some(arguments_def) = field.arguments() {
        for argument in arguments_def.arguments() {
            if let Some(argument_name) = argument.name().ok() {
                if argument_name.text() != "input" {
                    invalid_inputs.push(InvalidInputName {
                        argument_name: argument_name.text().to_string(),
                        text_range: argument.range(),
                    });
                } else if let Some(argument_type) = argument.ty().ok() {
                    let expected_type = format!("{}Input", field.name().unwrap().text());
                    if argument_type.text() != expected_type {
                        invalid_inputs.push(InvalidInputName {
                            argument_name: argument_name.text().to_string(),
                            text_range: argument.range(),
                        });
                    }
                }
            }
        }
    }

    invalid_inputs
}
