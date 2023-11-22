use crate::error;
use crate::error::ParseResult;
use crate::internal::blocks;
use crate::internal::identifiers;
use crate::internal::utils;
use crate::state::State;
use pxp_ast::try_block::CatchBlock;
use pxp_ast::try_block::CatchType;
use pxp_ast::try_block::FinallyBlock;
use pxp_ast::try_block::TryStatement;
use pxp_ast::StatementKind;
use pxp_token::TokenKind;

use super::variables;

pub fn try_block(state: &mut State) -> ParseResult<StatementKind> {
    let start = state.stream.current().span;

    state.stream.next();
    utils::skip_left_brace(state)?;

    let body = blocks::multiple_statements_until(state, &TokenKind::RightBrace)?;

    let last_right_brace = utils::skip_right_brace(state)?;

    let mut catches = Vec::new();
    loop {
        if state.stream.current().kind != TokenKind::Catch {
            break;
        }

        let catch_start = state.stream.current().span;

        state.stream.next();
        utils::skip_left_parenthesis(state)?;

        let types = catch_type(state)?;
        let var = if state.stream.current().kind == TokenKind::RightParen {
            None
        } else {
            Some(variables::simple_variable(state)?)
        };

        utils::skip_right_parenthesis(state)?;
        utils::skip_left_brace(state)?;

        let catch_body = blocks::multiple_statements_until(state, &TokenKind::RightBrace)?;

        utils::skip_right_brace(state)?;

        let catch_end = state.stream.current().span;

        catches.push(CatchBlock {
            start: catch_start,
            end: catch_end,
            types,
            var,
            body: catch_body,
        })
    }

    let mut finally = None;
    if state.stream.current().kind == TokenKind::Finally {
        let finally_start = state.stream.current().span;
        state.stream.next();
        utils::skip_left_brace(state)?;

        let finally_body = blocks::multiple_statements_until(state, &TokenKind::RightBrace)?;

        utils::skip_right_brace(state)?;
        let finally_end = state.stream.current().span;

        finally = Some(FinallyBlock {
            start: finally_start,
            end: finally_end,
            body: finally_body,
        });
    }

    if catches.is_empty() && finally.is_none() {
        todo!("tolerant mode")
        // return Err(error::try_without_catch_or_finally(start, last_right_brace));
    }

    let end = state.stream.current().span;

    Ok(StatementKind::Try(TryStatement {
        start,
        end,
        body,
        catches,
        finally,
    }))
}

#[inline(always)]
fn catch_type(state: &mut State) -> ParseResult<CatchType> {
    let id = identifiers::full_name(state)?;

    if state.stream.current().kind == TokenKind::Pipe {
        state.stream.next();

        let mut types = vec![id];

        while !state.stream.is_eof() {
            let id = identifiers::full_name(state)?;
            types.push(id);

            if state.stream.current().kind != TokenKind::Pipe {
                break;
            }

            state.stream.next();
        }

        return Ok(CatchType::Union { identifiers: types });
    }

    Ok(CatchType::Identifier { identifier: id })
}
