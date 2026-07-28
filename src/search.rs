use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::collections::{HashMap, HashSet};
use regex::Regex;
use std::sync::OnceLock;

fn tag_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)#([a-z0-9_-]+)").unwrap())
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub filename: String,
    pub title: String,
    pub score: i64,
    pub title_match_indices: Vec<usize>,
    pub content_preview: Option<String>,
    pub is_create_prompt: bool,
    pub modified_at: i64,
}

pub struct Index {
    matcher: SkimMatcherV2,
    // filename -> (title, content, modified_at, tags)
    notes: HashMap<String, (String, String, i64, HashSet<String>)>,
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

    pub fn add_note(&mut self, filename: String, content: String, modified_at: i64) {
        let title = filename.strip_suffix(".md").unwrap_or(&filename).to_string();
        let mut tags = HashSet::new();
        for cap in tag_regex().captures_iter(&content) {
            if let Some(match_) = cap.get(1) {
                tags.insert(match_.as_str().to_lowercase());
            }
        }
        self.notes.insert(filename, (title, content, modified_at, tags));
    }

    pub fn remove_note(&mut self, filename: &str) {
        self.notes.remove(filename);
    }

    pub fn rename_note(&mut self, old_filename: &str, new_filename: String, content: String, modified_at: i64) {
        self.remove_note(old_filename);
        self.add_note(new_filename, content, modified_at);
    }

    #[must_use]
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let is_tag_search = query.starts_with('#');
        let tag_query = if is_tag_search {
            query.trim_start_matches('#').to_lowercase()
        } else {
            String::new()
        };

        if query.is_empty() {
            // For empty query, return all notes sorted by modified date descending
            let mut all_notes: Vec<_> = self.notes.iter().map(|(filename, (title, _, modified_at, _))| {
                SearchResult {
                    filename: filename.clone(),
                    title: title.clone(),
                    score: 0,
                    title_match_indices: Vec::new(),
                    content_preview: None,
                    is_create_prompt: false,
                    modified_at: *modified_at,
                }
            }).collect();
            all_notes.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
            return all_notes;
        }

        let mut matches = Vec::new();

        for (filename, (title, content, modified_at, tags)) in &self.notes {
            if is_tag_search {
                if !tags.contains(&tag_query) && !tag_query.is_empty() {
                    continue;
                }
                
                // If it's a tag search and it matched, we give it a high score
                matches.push((
                    filename,
                    title,
                    content,
                    100, // Fixed score for tag matches
                    vec![],
                    false,
                    vec![],
                    *modified_at,
                ));
                continue;
            }

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
                *modified_at,
            ));
        }

        matches.sort_by(|a, b| b.3.cmp(&a.3));

        matches
            .into_iter()
            .take(100)
            .map(|(filename, title, content, score, title_match_indices, has_content_match, content_indices, modified_at)| {
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
                    is_create_prompt: false,
                    modified_at,
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
        index.add_note("1.md".to_string(), "content".to_string(), 100);
        index.add_note("2.md".to_string(), "content".to_string(), 200);
        
        let results = index.search("");
        assert_eq!(results.len(), 2);
        // Should sort by modified_at descending, so 2.md (200) is first
        assert_eq!(results[0].filename, "2.md");
        assert_eq!(results[1].filename, "1.md");
    }

    #[test]
    fn test_search_title_match_ranks_higher() {
        let mut index = Index::new();
        index.add_note("rust-guide.md".to_string(), "some guide".to_string(), 0);
        index.add_note("other.md".to_string(), "learn rust here".to_string(), 0);
        
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
        index.add_note("test.md".to_string(), content.to_string(), 0);
        
        let results = index.search("blazing");
        assert_eq!(results.len(), 1);
        assert!(results[0].content_preview.is_some());
        
        let preview = results[0].content_preview.as_ref().unwrap();
        assert!(preview.contains("blazing"));
    }
}
