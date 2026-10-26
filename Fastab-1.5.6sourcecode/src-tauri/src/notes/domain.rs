use crate::notes::Note;

/// 工作台只保留「手动」排序：顺序由每条笔记的 custom_order 决定，
/// 「最新/最早」两种按时间排序的模式已在 1.2.0 中移除。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoteSortMode {
    Custom,
}

pub(crate) fn sort_notes(notes: &mut [Note], _mode: NoteSortMode) {
    notes.sort_by(|a, b| {
        if a.is_deleted != b.is_deleted {
            return if a.is_deleted {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            };
        }
        if a.is_archived != b.is_archived {
            return if a.is_archived {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            };
        }
        if a.is_pinned != b.is_pinned {
            return if b.is_pinned {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            };
        }
        let ao = a.custom_order.unwrap_or(0);
        let bo = b.custom_order.unwrap_or(0);
        ao.cmp(&bo)
    });
}
