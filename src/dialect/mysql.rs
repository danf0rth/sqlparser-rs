// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::ast::Statement;
use crate::dialect::Dialect;
use crate::keywords::Keyword;
use crate::parser::{Parser, ParserError};
use crate::tokenizer::Token;

#[derive(Debug)]
pub struct MySqlDialect {}

impl MySqlDialect {
    fn parse_delete(&self, parser: &mut Parser) -> Result<Statement, ParserError> {
        parser.expect_keyword(Keyword::FROM)?;
        let table_name = parser.parse_table_factor()?;
        let using = if parser.parse_keyword(Keyword::USING) {
            Some(parser.parse_table_factor()?)
        } else {
            None
        };
        let selection = if parser.parse_keyword(Keyword::WHERE) {
            Some(parser.parse_expr()?)
        } else {
            None
        };

        let order_by = if parser.parse_keywords(&[Keyword::ORDER, Keyword::BY]) {
            Some(parser.parse_comma_separated(Parser::parse_order_by_expr)?)
        } else {
            None
        };

        let limit = if parser.parse_keyword(Keyword::LIMIT) {
            // TODO: can be ...LIMIT 5, 10 = LIMIT 5 OFFSET 10
            parser.parse_limit()?
        } else {
            None
        };

        Ok(Statement::Delete {
            table_name,
            using,
            selection,
            order_by,
            limit,
        })
    }
}

impl Dialect for MySqlDialect {
    fn is_delimited_identifier_start(&self, ch: char) -> bool {
        ch == '`'
    }

    fn is_identifier_start(&self, ch: char) -> bool {
        // See https://dev.mysql.com/doc/refman/8.0/en/identifiers.html.
        // We don't yet support identifiers beginning with numbers, as that
        // makes it hard to distinguish numeric literals.
        ('a'..='z').contains(&ch)
            || ('A'..='Z').contains(&ch)
            || ch == '_'
            || ch == '$'
            || ch == '@'
            || ('\u{0080}'..='\u{ffff}').contains(&ch)
    }

    fn is_identifier_part(&self, ch: char) -> bool {
        self.is_identifier_start(ch) || ('0'..='9').contains(&ch)
    }

    fn parse_statement(&self, _parser: &mut Parser) -> Option<Result<Statement, ParserError>> {
        if let Token::Word(w) = _parser.next_token() {
            if w.keyword == Keyword::DELETE {
                return Some(self.parse_delete(_parser))
            }
        }
        _parser.prev_token();
        None
    }
}
