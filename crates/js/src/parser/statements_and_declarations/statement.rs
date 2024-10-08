use crate::parser::{
    expressions::Expression,
    tokenization::{Punctuator, SkipLineTerminators, Token, Tokenizer},
    SyntaxError,
};

use super::{
    block_statement::BlockStatement, if_statement::IfStatement, throw_statement::ThrowStatement,
    Declaration, WhileStatement,
};

/// <https://262.ecma-international.org/14.0/#prod-StatementListItem>
#[derive(Clone, Debug)]
pub enum StatementListItem {
    Statement(Statement),
    Declaration(Declaration),
}

impl StatementListItem {
    /// <https://262.ecma-international.org/14.0/#prod-StatementListItem>
    pub(crate) fn parse<const YIELD: bool, const AWAIT: bool, const RETURN: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        let Some(next_token) = tokenizer.peek(0, SkipLineTerminators::Yes)? else {
            return Err(tokenizer.syntax_error("expected more tokens"));
        };

        let statement_list_item = match next_token {
            Token::Identifier(ident) if matches!(ident.as_str(), "function" | "let" | "const") => {
                Declaration::parse::<YIELD, AWAIT>(tokenizer)?.into()
            },
            Token::Punctuator(Punctuator::CurlyBraceOpen | Punctuator::Semicolon) => {
                Statement::parse::<YIELD, AWAIT, RETURN>(tokenizer)?.into()
            },
            Token::Identifier(ident) if matches!(ident.as_str(), "if" | "while" | "throw") => {
                Statement::parse::<YIELD, AWAIT, RETURN>(tokenizer)?.into()
            },
            _ => Statement::parse::<YIELD, AWAIT, RETURN>(tokenizer)?.into(),
        };

        Ok(statement_list_item)
    }
}

/// <https://262.ecma-international.org/14.0/#prod-Statement>
#[derive(Clone, Debug)]
pub enum Statement {
    BlockStatement(BlockStatement),
    EmptyStatement,
    ExpressionStatement(Expression),
    IfStatement(IfStatement),
    WhileStatement(WhileStatement),
    ThrowStatement(ThrowStatement),
}

impl Statement {
    /// <https://262.ecma-international.org/14.0/#prod-Statement>
    pub fn parse<const YIELD: bool, const AWAIT: bool, const RETURN: bool>(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Result<Self, SyntaxError> {
        let Some(next_token) = tokenizer.peek(0, SkipLineTerminators::Yes)? else {
            return Err(tokenizer.syntax_error("expected more tokens"));
        };

        let statement = match next_token {
            Token::Punctuator(Punctuator::CurlyBraceOpen) => {
                let block_statement = BlockStatement::parse::<YIELD, AWAIT, RETURN>(tokenizer)?;
                block_statement.into()
            },
            Token::Identifier(ident) if ident == "if" => {
                let if_statement = IfStatement::parse::<YIELD, AWAIT, RETURN>(tokenizer)?;
                if_statement.into()
            },
            Token::Identifier(ident) if ident == "while" => {
                let while_statement = WhileStatement::parse::<YIELD, AWAIT, RETURN>(tokenizer)?;
                while_statement.into()
            },
            Token::Identifier(ident) if ident == "throw" => {
                let throw_statement = ThrowStatement::parse::<YIELD, AWAIT>(tokenizer)?;
                throw_statement.into()
            },
            Token::Punctuator(Punctuator::Semicolon) => {
                // https://262.ecma-international.org/14.0/#prod-EmptyStatement
                tokenizer.advance(1);
                Self::EmptyStatement
            },
            _ => {
                let expression_statement = Expression::parse::<true, YIELD, AWAIT>(tokenizer)?;
                expression_statement.into()
            },
        };

        Ok(statement)
    }
}

impl From<Statement> for StatementListItem {
    fn from(value: Statement) -> Self {
        Self::Statement(value)
    }
}

impl From<Declaration> for StatementListItem {
    fn from(value: Declaration) -> Self {
        Self::Declaration(value)
    }
}

impl From<ThrowStatement> for Statement {
    fn from(value: ThrowStatement) -> Self {
        Self::ThrowStatement(value)
    }
}

impl From<WhileStatement> for Statement {
    fn from(value: WhileStatement) -> Self {
        Self::WhileStatement(value)
    }
}

impl From<IfStatement> for Statement {
    fn from(value: IfStatement) -> Self {
        Self::IfStatement(value)
    }
}

impl From<BlockStatement> for Statement {
    fn from(value: BlockStatement) -> Self {
        Self::BlockStatement(value)
    }
}

impl From<Expression> for Statement {
    fn from(value: Expression) -> Self {
        Self::ExpressionStatement(value)
    }
}
