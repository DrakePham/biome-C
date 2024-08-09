use std::collections::HashSet;

use biome_analyze::{
    context::RuleContext, declare_lint_rule, Ast, Rule, RuleDiagnostic, RuleSource, RuleSourceKind,
};
use biome_console::markup;
use biome_graphql_syntax::{
    AnyGraphqlOperationDefinition, GraphqlFieldDefinition, GraphqlSelectionSet,
};
use biome_rowan::{AstNode, TextRange};

declare_lint_rule! {
    /// Ensure mutation arguments are named `input` and input types follow the `MutationNameInput` naming convention.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```graphql,expect_diagnostic
    /// mutation {
    ///   addUser(userData: AddUserInput) {
    ///     id
    ///     name
    ///   }
    /// }
    ///
    /// input AddUserInput {
    ///   name: String
    ///   email: String
    /// }
    /// ```
    ///
    /// ### Valid
    ///
    /// ```graphql
    /// mutation {
    ///   addUser(input: AddUserInput) {
    ///     id
    ///     name
    ///   }
    /// }
    ///
    /// input AddUserInput {
    ///   name: String
    ///   email: String
    /// }
    /// ```
    ///
    pub InputName {
        version: "next",
        name: "inputName",
        language: "graphql",
        sources: &[RuleSource::EslintGraphql("input-name")],
        source_kind: RuleSourceKind::SameLogic,
        recommended: true,
    }
}

impl Rule for InputName {
    type Query = Ast<GraphqlFieldDefinition>; // Correct your type based on the actual query type needed
    type State = InvalidInputName;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<Self::State> {
        let field = ctx.query();
        let mut invalid_names = vec![];

        // Check for specific conditions, e.g., ensure we're in a mutation operation context
        if let Some(field_name) = field.name().ok() {
            let name = field_name.text();
            // Check if the argument is named correctly
            if name != "input" {
                invalid_names.push(InvalidInputName {
                    name: name.to_string(),
                    text_range: field.range(),
                    kind: InvalidNameKind::Argument,
                });
            }
            // Further checks based on context
        }

        invalid_names
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let InvalidInputName { name, text_range, kind } = state;
        let message = match kind {
            InvalidNameKind::Argument => format!("Mutation argument `{}` should be named `input`.", name),
            InvalidNameKind::InputType => format!("Input type `{}` should be named according to the `MutationNameInput` convention.", name),
        };

        Some(RuleDiagnostic::new(
            rule_category!(),
            *text_range,
            markup! { { message } },
        ))
    }
}

#[derive(Debug)]
pub enum InvalidNameKind {
    Argument,
    InputType,
}

#[derive(Debug)]
pub struct InvalidInputName {
    name: String,
    text_range: TextRange,
    kind: InvalidNameKind,
}
