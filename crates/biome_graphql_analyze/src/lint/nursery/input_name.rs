use std::collections::HashSet;

use biome_analyze::{
    context::RuleContext, declare_lint_rule, Ast, Rule, RuleDiagnostic, RuleSource, RuleSourceKind,
};
use biome_graphql_syntax::{
    GraphqlArgument, GraphqlInputObjectType, GraphqlMutation, GraphqlSchema, GraphqlTypeDefinition,
};
use biome_rowan::TextRange;

declare_lint_rule! {
    /// Ensure that mutation arguments are named "input" and input types are named "<MutationName>Input".
    ///
    /// ## Examples
    ///
    /// ### Incorrect
    ///
    /// ```graphql
    /// type Mutation {
    ///   SetMessage(message: InputMessage): String
    /// }
    /// ```
    ///
    /// ### Correct (with checkInputType)
    ///
    /// ```graphql
    /// type Mutation {
    ///   SetMessage(input: SetMessageInput): String
    /// }
    /// ```
    ///
    /// ### Correct (without checkInputType)
    ///
    /// ```graphql
    /// type Mutation {
    ///   SetMessage(input: AnyInputTypeName): String
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
    type Query = Ast<GraphqlSchema>;
    type State = NamingIssue;
    type Signals = Vec<Self::State>;
    type Options = InputNameConventionOptions;

    fn run(ctx: &RuleContext<Self>) -> Vec<Self::State> {
        let schema = ctx.query();
        let options = ctx.options();

        let mut issues = vec![];
        
        for mutation in schema.mutations() {
            for field in mutation.fields() {
                if let Some(argument) = field.arguments().first() {
                    let argument_name = argument.name().text();
                    let expected_name = "input";
                    
                    if argument_name != expected_name && options.check_input_name {
                        issues.push(NamingIssue {
                            name: argument_name,
                            text_range: argument.range(),
                            issue_type: IssueType::ArgumentName,
                        });
                    }
                    
                    if options.check_input_type {
                        if let Some(input_type) = field.input_type() {
                            let expected_type_name = format!("{}Input", mutation.name());
                            let input_type_name = input_type.name().text();
                            
                            if input_type_name != expected_type_name {
                                issues.push(NamingIssue {
                                    name: input_type_name,
                                    text_range: input_type.range(),
                                    issue_type: IssueType::InputTypeName,
                                });
                            }
                        }
                    }
                }
            }
        }

        issues
    }

    fn diagnostic(_ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let message = match state.issue_type {
            IssueType::ArgumentName => format!(
                "Mutation argument should be named `input`. Found `{}`.",
                state.name
            ),
            IssueType::InputTypeName => format!(
                "Input type should be named `<MutationName>Input`. Found `{}`.",
                state.name
            ),
        };
        
        Some(RuleDiagnostic::new(
            rule_category!(),
            state.text_range,
            markup! { {message} },
        ))
    }
}

#[derive(Debug)]
pub enum IssueType {
    ArgumentName,
    InputTypeName,
}

#[derive(Debug)]
pub struct NamingIssue {
    name: String,
    text_range: TextRange,
    issue_type: IssueType,
}

#[derive(Debug, Default)]
pub struct InputNameConventionOptions {
    check_input_name: bool,
    check_input_type: bool,
    case_sensitive_input_type: bool,
    check_queries: bool,
    check_mutations: bool,
}
