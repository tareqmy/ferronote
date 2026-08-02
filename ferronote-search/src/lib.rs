
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

fn tag_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)#([a-z0-9_-]+)").expect("valid tag regex"))
}

/// Single search result or note selection candidate.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Note filename (e.g. `meeting.md`).
    pub filename: String,
    /// Clean title displayed in UI without extension.
    pub title: String,
    /// Match score derived from Skim fuzzy matcher.
    pub score: i64,
    /// Indices of characters in title matching the search query.
    pub title_match_indices: Vec<usize>,
    /// Optional content preview snippet around content match.
    pub content_preview: Option<String>,
    /// Flag set if item represents the unified "Create note" prompt.
    pub is_create_prompt: bool,
    /// Last modification Unix timestamp.
    pub modified_at: i64,
    /// Whether the note is pinned.
    pub is_pinned: bool,
}

/// In-memory search index supporting fuzzy search, tag indexing, and backlink resolution.
pub struct Index {
    // filename -> (title, content, modified_at, tags, is_pinned)
    notes: HashMap<String, (String, String, i64, HashSet<String>, bool)>,
}

impl std::fmt::Debug for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Index").field("notes", &self.notes).finish()
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}

impl Index {
    /// Creates a new empty search `Index`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            notes: HashMap::new(),
        }
    }

    /// Adds or updates a note in the search index, extracting `#tag` annotations automatically.
    pub fn add_note(&mut self, filename: String, content: String, modified_at: i64) {
        self.add_note_with_pin(filename, content, modified_at, false);
    }

    /// Adds or updates a note with an explicit `is_pinned` status.
    pub fn add_note_with_pin(
        &mut self,
        filename: String,
        content: String,
        modified_at: i64,
        is_pinned: bool,
    ) {
        let title = filename
            .strip_suffix(".md")
            .unwrap_or(&filename)
            .to_string();
        let mut tags = HashSet::new();
        for cap in tag_regex().captures_iter(&content) {
            if let Some(match_) = cap.get(1) {
                tags.insert(match_.as_str().to_lowercase());
            }
        }
        self.notes
            .insert(filename, (title, content, modified_at, tags, is_pinned));
    }

    /// Sets or updates the pinned status of a note.
    pub fn set_pinned(&mut self, filename: &str, is_pinned: bool) {
        if let Some(note) = self.notes.get_mut(filename) {
            note.4 = is_pinned;
        }
    }

    /// Removes a note from the search index by filename.
    pub fn remove_note(&mut self, filename: &str) {
        self.notes.remove(filename);
    }

    /// Renames a note in the index, updating its key, content, and modified timestamp.
    pub fn rename_note(
        &mut self,
        old_filename: &str,
        new_filename: String,
        content: String,
        modified_at: i64,
    ) {
        let is_pinned = self.notes.get(old_filename).map_or(false, |n| n.4);
        self.remove_note(old_filename);
        self.add_note_with_pin(new_filename, content, modified_at, is_pinned);
    }

    /// Returns a list of notes containing wiki-style links (`[[note_title]]`) pointing to `note_title`.
    #[must_use]
    pub fn get_backlinks(&self, note_title: &str) -> Vec<SearchResult> {
        let clean_title = note_title.strip_suffix(".md").unwrap_or(note_title).trim();
        let target_title_lower = clean_title.to_lowercase();

        let mut backlinks = Vec::new();

        for (filename, (title, content, modified_at, _, is_pinned)) in &self.notes {
            if title.eq_ignore_ascii_case(clean_title) {
                continue;
            }

            let content_lower = content.to_lowercase();
            let mut current_idx = 0;
            let mut matches = false;

            while let Some(start) = content_lower[current_idx..].find("[[") {
                let start_idx = current_idx + start;
                if let Some(end) = content_lower[start_idx..].find("]]") {
                    let link_text = &content_lower[start_idx + 2..start_idx + end];
                    let clean_link = link_text.strip_suffix(".md").unwrap_or(link_text).trim();
                    if clean_link == target_title_lower {
                        matches = true;
                        break;
                    }
                    current_idx = start_idx + end + 2;
                } else {
                    break;
                }
            }

            if matches {
                backlinks.push(SearchResult {
                    filename: filename.clone(),
                    title: title.clone(),
                    score: 100,
                    title_match_indices: Vec::new(),
                    content_preview: Some(format!("Links to [[{clean_title}]]")),
                    is_create_prompt: false,
                    modified_at: *modified_at,
                    is_pinned: *is_pinned,
                });
            }
        }

        backlinks.sort_by_key(|b| std::cmp::Reverse(b.modified_at));
        backlinks
    }

    /// Searches the index with `query`. Supports `#tag` filtering and fuzzy title/content matching.
    #[must_use]
    pub fn search(&self, query: &str, sort_order: &str) -> Vec<SearchResult> {
        let is_tag_search = query.starts_with('#');
        let tag_query = if is_tag_search {
            query.trim_start_matches('#').to_lowercase()
        } else {
            String::new()
        };

        if query.is_empty() {
            // For empty query, return all notes sorted with pinned notes first, then sort_order
            let mut all_notes: Vec<_> = self
                .notes
                .iter()
                .map(|(filename, (title, _, modified_at, _, is_pinned))| SearchResult {
                    filename: filename.clone(),
                    title: title.clone(),
                    score: 0,
                    title_match_indices: Vec::new(),
                    content_preview: None,
                    is_create_prompt: false,
                    modified_at: *modified_at,
                    is_pinned: *is_pinned,
                })
                .collect();
            all_notes.sort_by(|a, b| {
                let pin_cmp = b.is_pinned.cmp(&a.is_pinned);
                pin_cmp.then_with(|| {
                    let cmp = match sort_order {
                        "title_asc" => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
                        "title_desc" => b.title.to_lowercase().cmp(&a.title.to_lowercase()),
                        "modified_asc" | "created_asc" => a.modified_at.cmp(&b.modified_at),
                        _ => b.modified_at.cmp(&a.modified_at),
                    };
                    cmp.then_with(|| a.filename.cmp(&b.filename))
                })
            });
            return all_notes;
        }

        let mut matches = Vec::new();

        for (filename, (title, content, modified_at, tags, is_pinned)) in &self.notes {
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
                    *is_pinned,
                ));
                continue;
            }

            let query_words: Vec<&str> = query.split_whitespace().collect();
            let mut matches_all = true;
            let mut total_score = 0;
            let mut all_title_indices = Vec::new();
            let mut all_content_indices = Vec::new();
            
            let title_lower = title.to_lowercase();
            let content_lower = content.to_lowercase();

            for word in &query_words {
                let word_lower = word.to_lowercase();
                let mut word_matched = false;

                if let Some(start) = title_lower.find(&word_lower) {
                    word_matched = true;
                    let char_start = title_lower[..start].chars().count();
                    let char_len = word_lower.chars().count();
                    for i in char_start..char_start + char_len {
                        if !all_title_indices.contains(&i) {
                            all_title_indices.push(i);
                        }
                    }
                    total_score += 30; // 3x multiplier for title match
                }

                if let Some(start) = content_lower.find(&word_lower) {
                    word_matched = true;
                    let byte_len = word_lower.len();
                    for i in start..start + byte_len {
                        if !all_content_indices.contains(&i) {
                            all_content_indices.push(i);
                        }
                    }
                    total_score += 10;
                }

                if !word_matched {
                    matches_all = false;
                    break;
                }
            }

            if !matches_all || query_words.is_empty() {
                continue;
            }

            let has_content_match = !all_content_indices.is_empty();
            all_title_indices.sort_unstable();
            all_content_indices.sort_unstable();

            matches.push((
                filename,
                title,
                content,
                total_score,
                all_title_indices,
                has_content_match,
                all_content_indices,
                *modified_at,
                *is_pinned,
            ));
        }

        matches.sort_by(|a, b| {
            b.8.cmp(&a.8)
                .then_with(|| b.3.cmp(&a.3))
                .then_with(|| {
                    let cmp = match sort_order {
                        "title_asc" => a.1.to_lowercase().cmp(&b.1.to_lowercase()),
                        "title_desc" => b.1.to_lowercase().cmp(&a.1.to_lowercase()),
                        "modified_asc" | "created_asc" => a.7.cmp(&b.7),
                        _ => b.7.cmp(&a.7),
                    };
                    cmp.then_with(|| a.0.cmp(b.0))
                })
        });

        matches
            .into_iter()
            .take(100)
            .map(
                |(
                    filename,
                    title,
                    content,
                    score,
                    title_match_indices,
                    has_content_match,
                    content_indices,
                    modified_at,
                    is_pinned,
                )| {
                    let mut content_preview = None;
                    if has_content_match && let Some(&first_idx) = content_indices.first() {
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
                        content_preview = Some(format!(
                            "{}{}{}",
                            prefix,
                            snippet.replace('\n', " "),
                            suffix
                        ));
                    }

                    SearchResult {
                        filename: filename.clone(),
                        title: title.clone(),
                        score,
                        title_match_indices,
                        content_preview,
                        is_create_prompt: false,
                        modified_at,
                        is_pinned,
                    }
                },
            )
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

        let results = index.search("", "modified_desc");
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

        let results = index.search("rust", "modified_desc");
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

        let results = index.search("blazing", "modified_desc");
        assert_eq!(results.len(), 1);
        assert!(results[0].content_preview.is_some());

        let preview = results[0].content_preview.as_ref().unwrap();
        assert!(preview.contains("blazing"));
    }

    #[test]
    fn test_tag_search_filter() {
        let mut index = Index::new();
        index.add_note(
            "todo.md".to_string(),
            "Finish project #todo #urgent".to_string(),
            100,
        );
        index.add_note("ideas.md".to_string(), "New ideas #ideas".to_string(), 200);

        let results = index.search("#todo", "modified_desc");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "todo.md");

        let results_urgent = index.search("#urgent", "modified_desc");
        assert_eq!(results_urgent.len(), 1);
        assert_eq!(results_urgent[0].filename, "todo.md");

        let results_ideas = index.search("#ideas", "modified_desc");
        assert_eq!(results_ideas.len(), 1);
        assert_eq!(results_ideas[0].filename, "ideas.md");
    }

    #[test]
    fn test_get_backlinks() {
        let mut index = Index::new();
        index.add_note(
            "source.md".to_string(),
            "Check out [[Target Note]] for details.".to_string(),
            100,
        );
        index.add_note(
            "target note.md".to_string(),
            "Content of target note".to_string(),
            200,
        );

        let backlinks = index.get_backlinks("Target Note");
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].filename, "source.md");
    }

    #[test]
    fn test_remove_and_rename_note_index() {
        let mut index = Index::new();
        index.add_note("old.md".to_string(), "Ferronote app #v1".to_string(), 100);

        let results = index.search("Ferronote", "modified_desc");
        assert_eq!(results.len(), 1);

        index.rename_note(
            "old.md",
            "new.md".to_string(),
            "Ferronote app #v2".to_string(),
            150,
        );

        let results_old = index.search("old", "modified_desc");
        assert!(results_old.iter().all(|r| r.filename != "old.md"));

        let results_new = index.search("Ferronote", "modified_desc");
        assert_eq!(results_new.len(), 1);
        assert_eq!(results_new[0].filename, "new.md");

        index.remove_note("new.md");
        let results_removed = index.search("Ferronote", "modified_desc");
        assert_eq!(results_removed.len(), 0);
    }

    #[test]
    fn test_pinned_notes_sort_first() {
        let mut index = Index::new();
        index.add_note_with_pin("old_unpinned.md".to_string(), "content".to_string(), 200, false);
        index.add_note_with_pin("older_pinned.md".to_string(), "content".to_string(), 100, true);

        let results = index.search("", "modified_desc");
        assert_eq!(results.len(), 2);
        // Pinned note should sort first despite having an older modified timestamp
        assert_eq!(results[0].filename, "older_pinned.md");
        assert!(results[0].is_pinned);
        assert_eq!(results[1].filename, "old_unpinned.md");
        assert!(!results[1].is_pinned);
    }
}
