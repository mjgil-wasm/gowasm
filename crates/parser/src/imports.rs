use super::*;

impl Parser {
    pub(super) fn parse_import_decls(&mut self) -> Result<Vec<ImportDecl>, ParseError> {
        self.expect_keyword("`import`", TokenKind::Import)?;
        if self.check(|kind| matches!(kind, TokenKind::LParen)) {
            self.bump();
            let mut imports = Vec::new();
            while !self.check(|kind| matches!(kind, TokenKind::RParen)) {
                imports.push(self.parse_import_decl()?);
                while self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                    self.bump();
                }
            }
            self.bump();
            return Ok(imports);
        }

        Ok(vec![self.parse_import_decl()?])
    }

    fn parse_import_decl(&mut self) -> Result<ImportDecl, ParseError> {
        let start = self.current_token()?.span.start;
        let alias = match &self.current_token()?.kind {
            TokenKind::Ident(name) => {
                let alias = name.clone();
                self.bump();
                Some(alias)
            }
            _ => None,
        };
        let path = self.expect_string()?;
        self.source_file_spans.imports.push(Span {
            start,
            end: self.last_consumed_end(start),
        });
        Ok(ImportDecl { alias, path })
    }
}
