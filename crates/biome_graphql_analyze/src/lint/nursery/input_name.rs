use biome_analyze::{context::RuleContext, declare_rule, Ast, Rule, RuleDiagnostic};
use biome_console::markup;
use biome_graphql_syntax::GraphqlRoot;
use biome_rowan::AstNode;

declare_rule! {
    /// 
    /// Requires mutation argument to be always called "input" and input type to be called
    /// <MutationName> + "Input".
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```graphql,expect_diagnostic
    /// mutation {
    ///   addUser(userData: AddUserInput){
    ///       id
    ///       name
    ///   }
    /// }
    /// 
    /// input AddUserInput{
    ///   name: String
    ///   email: String
    /// }
    /// ```
    ///
    /// ### Valid
    ///
    /// ```graphql
    /// mutation {
    ///   addUser(input: AddUserInput){
    ///     id
    ///     name
    ///   }
    /// }
    /// 
    /// input AddUserInput{
    ///   name: String
    ///   email: String
    /// }
    /// 
    /// ```
    ///
    pub InputName {
        version: "next",
        name: "inputName",
        language: "graphql",
        recommended: false,
        sources: &[RuleSource::EslintGraphql("input-name")],
        source_kind: RuleSourceKind::SameLogic,
        recommended: true,
    }
}

impl Rule for InputName {
    type Query = Ast<GraphqlFieldDefinition>;
    type State = InvalidInputName;
    type Signals = Vec<Self::State>;
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<Self::State> {
        let field = ctx.query();
        let mut invalid_names = vec![];

        // Double checks if we are within a mutation context
        if !field.is_in_mutation_context(){
            return invalid_names;
        }

        // Checking for argument names
        if let Some(field_name) = field.name().ok(){
            let name = field_name.text();
            if name != "input"{
                invalid_names.push(InvalidInputName{
                    name: name.to_string(),
                    text_range: field.range(),
                    kind: InvalidNameKind::Argument,
                });
            }
        }

        // Add other comprehensize checks or refining the existing logic

        invalid_names
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Vec<RuleDiagnostic> {
        let InvalidInputName { name, text_range, kind } = state;

        let message = match kind {
            InvalidNameKind::Argument => format!("Mutation argument '{}' should be named 'input'.", name),
            InvalidNameKind::InputType => format!("Input type `{}` should follow the `MutationNameInput` naming convention.", name),
        };

        Some(RuleDiagnostic::new(
            rule_category!(),
            *text_range,
            markup! {{ message }},
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