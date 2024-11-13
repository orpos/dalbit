use full_moon::{
    ast::{Block, Stmt},
    node::Node,
    tokenizer::{Token, TokenReference},
    visitors::VisitorMut,
};

#[derive(Debug)]
pub struct RemoveEmptyDo {}

impl VisitorMut for RemoveEmptyDo {
    fn visit_block(&mut self, block: Block) -> Block {
        let mut stmts: Vec<(Stmt, Option<TokenReference>)> = Vec::new();
        let mut trivia_backup: Vec<&Token> = Vec::new();
        for (stmt, semicolon) in block.stmts_with_semicolon() {
            if let Stmt::Do(do_stmt) = stmt {
                let block = do_stmt.block();

                if block.stmts().count() < 1 && block.last_stmt().is_none() {
                    println!("DELETE END DO");
                    for token_ref in do_stmt.tokens() {
                        for t in token_ref.leading_trivia() {
                            trivia_backup.push(t);
                        }
                        for t in token_ref.trailing_trivia() {
                            trivia_backup.push(t);
                        }
                    }

                    continue;
                }
            }

            let new_stmt = stmt.clone();
            if !trivia_backup.is_empty() {
                let (leading, _) = &mut new_stmt.surrounding_trivia();
                for t in trivia_backup.clone() {
                    leading.push(t);
                }
                println!("trivia: {:?}\n\ninserted: {:?}", trivia_backup, new_stmt.surrounding_trivia());
                trivia_backup.clear();
            }

            stmts.push((new_stmt, semicolon.clone()));
        }

        block.with_stmts(stmts)
    }
}
