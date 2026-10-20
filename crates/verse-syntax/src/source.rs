use serde::Serialize;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct FileId(pub u32);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Span {
    pub file_id: FileId,
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(file_id: FileId, start: usize, end: usize) -> Self {
        Self {
            file_id,
            start: start.try_into().expect("source exceeds 4 GiB"),
            end: end.try_into().expect("source exceeds 4 GiB"),
        }
    }
}

/// Source text and its byte-indexed line table. Columns are Unicode scalar counts.
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub id: FileId,
    pub name: String,
    pub text: String,
    line_starts: Vec<usize>,
}

impl SourceFile {
    pub fn new(id: FileId, name: impl Into<String>, text: impl Into<String>) -> Self {
        let text = text.into();
        let mut line_starts = vec![0];
        line_starts.extend(text.match_indices('\n').map(|(p, _)| p + 1));
        Self {
            id,
            name: name.into(),
            text,
            line_starts,
        }
    }

    pub fn position(&self, offset: u32) -> (usize, usize) {
        let mut offset = (offset as usize).min(self.text.len());
        while !self.text.is_char_boundary(offset) {
            offset -= 1;
        }
        let line = self.line_starts.partition_point(|&p| p <= offset) - 1;
        (
            line + 1,
            self.text[self.line_starts[line]..offset].chars().count() + 1,
        )
    }

    pub fn line(&self, number: usize) -> &str {
        let Some(&start) = number.checked_sub(1).and_then(|i| self.line_starts.get(i)) else {
            return "";
        };
        let end = self
            .line_starts
            .get(number)
            .copied()
            .unwrap_or(self.text.len());
        self.text[start..end].trim_end_matches(['\r', '\n'])
    }
}
