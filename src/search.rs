use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub filename: String,
    pub title: String,
    pub score: i64,
    pub title_match_indices: Vec<usize>,
    pub content_preview: Option<String>,
}

pub struct Index {
    matcher: SkimMatcherV2,
    // filename -> (title, content)
    notes: HashMap<String, (String, String)>,
}

impl std::fmt::Debug for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Index")
            .field("notes", &self.notes)
            .finish()
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}

impl Index {
    #[must_use]
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default().smart_case(),
            notes: HashMap::new(),
        }
    }

    pub fn add_note(&mut self, filename: String, content: String) {
        let title = filename.strip_suffix(".md").unwrap_or(&filename).to_string();
        self.notes.insert(filename, (title, content));
    }

    pub fn remove_note(&mut self, filename: &str) {
        self.notes.remove(filename);
    }

    pub fn rename_note(&mut self, old_filename: &str, new_filename: String, content: String) {
        self.remove_note(old_filename);
        self.add_note(new_filename, content);
    }

    #[must_use]
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        if query.is_empty() {
            // For empty query, return all notes with 0 score. 
            // They will be sorted externally (e.g. by modified date)
            return self.notes.iter().map(|(filename, (title, _))| {
                SearchResult {
                    filename: filename.clone(),
                    title: title.clone(),
                    score: 0,
                    title_match_indices: Vec::new(),
                    content_preview: None,
                }
            }).collect();
        }

        let mut matches = Vec::new();

        for (filename, (title, content)) in &self.notes {
            let title_match = self.matcher.fuzzy_indices(title, query);
            let content_match = self.matcher.fuzzy_indices(content, query);

            if title_match.is_none() && content_match.is_none() {
                continue;
            }

            let (title_score, title_indices) = title_match.unwrap_or((0, vec![]));
            let (content_score, content_indices) = content_match.unwrap_or((0, vec![]));

            let total_score = (title_score * 3) + content_score;

            matches.push((
                filename,
                title,
                content,
                total_score,
                title_indices,
                content_score > 0,
                content_indices,
            ));
        }

        matches.sort_by(|a, b| b.3.cmp(&a.3));

        matches
            .into_iter()
            .take(100)
            .map(|(filename, title, content, score, title_match_indices, has_content_match, content_indices)| {
                let mut content_preview = None;
                if has_content_match {
                    if let Some(&first_idx) = content_indices.first() {
                        let start = first_idx.saturating_sub(15);
                        let end = (first_idx + 40).min(content.len());
                        
                        let mut start_idx = start;
                        while start_idx > 0 && !content.is_char_boundary(start_idx) {
                            start_idx -= 1;
                        }
                        let mut end_idx = end;
                        while end_idx < content.len() && !content.is_char_boundary(end_idx) {
                            end_idx += 1;
                        }
                        
                        let snippet = &content[start_idx..end_idx];
                        let prefix = if start_idx > 0 { "..." } else { "" };
                        let suffix = if end_idx < content.len() { "..." } else { "" };
                        content_preview = Some(format!("{}{}{}", prefix, snippet.replace('\n', " "), suffix));
                    }
                }

                SearchResult {
                    filename: filename.clone(),
                    title: title.clone(),
                    score,
                    title_match_indices,
                    content_preview,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_query_returns_all() {
        let mut index = Index::new();
        index.add_note("1.md".to_string(), "content".to_string());
        index.add_note("2.md".to_string(), "content".to_string());
        
        let results = index.search("");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].score, 0);
    }

    #[test]
    fn test_search_title_match_ranks_higher() {
        let mut index = Index::new();
        index.add_note("rust-guide.md".to_string(), "some guide".to_string());
        index.add_note("other.md".to_string(), "learn rust here".to_string());
        
        let results = index.search("rust");
        assert_eq!(results.len(), 2);
        
        // title match should rank higher because of 3x multiplier
        assert_eq!(results[0].filename, "rust-guide.md");
        assert_eq!(results[1].filename, "other.md");
    }

    #[test]
    fn test_search_content_snippet() {
        let mut index = Index::new();
        let content = "This is a long text containing the word blazing and some other stuff.";
        index.add_note("test.md".to_string(), content.to_string());
        
        let results = index.search("blazing");
        assert_eq!(results.len(), 1);
        assert!(results[0].content_preview.is_some());
        
        let preview = results[0].content_preview.as_ref().unwrap();
        assert!(preview.contains("blazing"));
    }
}
