//! Developer Syntax & Code Mode Drill Generator (Milestone 5.2).
//!
//! Provides programming syntax exercises targeting brackets, operators,
//! punctuation, and language-specific idioms (Rust, TypeScript, Python, SQL).

use crate::app::{Level, RoundLength};

pub const RUST_CODE_SNIPPETS: &[&str] = &[
    "fn handle_packet<'a, T: Deserialize<'a>>(buf: &'a [u8]) -> Result<Option<T>, Box<dyn Error>> {",
    "let header = parse_header(&buf[..16])?; if header.version != PROTOCOL_V2 { return Ok(None); }",
    "match msg.kind { MsgKind::Heartbeat => Ok(Some(T::default())), _ => Err(Box::new(Error::Unknown)) }",
    "pub struct Engine<T> where T: Send + Sync + 'static { inner: Arc<Mutex<T>>, sender: mpsc::Sender<Event> }",
    "let mut map: HashMap<String, Vec<u64>> = HashMap::with_capacity(128);",
];

pub const TYPESCRIPT_CODE_SNIPPETS: &[&str] = &[
    "export const updateMetrics = async <T extends BaseRecord>(endpoint: string, payload: Partial<T>): Promise<Result<T, ApiError>> => {",
    "const { data, status } = await httpClient.post<T>(`/api/v1/${endpoint}`, { ...payload, timestamp: Date.now() });",
    "type FilterPredicate<T> = (item: T, index: number, array: readonly T[]) => item is T;",
    "interface AppState<K extends string, V> { readonly cache: Map<K, V>; dispatch: (action: Action) => void; }",
    "const result = items.filter(x => x.isValid).map(x => ({ id: x.id, score: x.val * 1.5 }));",
];

pub const PYTHON_CODE_SNIPPETS: &[&str] = &[
    "async def stream_records(pool: asyncpg.Pool, query: str, limit: int = 100) -> list[dict[str, Any]]:",
    "async with pool.acquire() as conn: records = await conn.fetch(f'{query} LIMIT $1;', limit)",
    "return [{'id': r['id'], 'score': round(float(r['score']), 3)} for r in records if r]",
    "class MetricPipeline(Generic[T]): def __init__(self, buffer_size: int = 256) -> None: self._buf = []",
    "@dataclass(frozen=True) class EventPayload: event_id: UUID; payload: bytes; retries: int = 0",
];

pub const SYMBOL_SPRINTS: &[&str] = &[
    "() [] {} <> {()} [{}] (<>) ({[]}) [()] {<>} <{}>",
    "-> => :: == != <= >= += -= *= /= && || ?? &= |=",
    "getUserById total_error_count MAX_BUFFER_SIZE kebab-case-attr",
    "arr[i] = &obj->field; map.get(&key).unwrap_or_default();",
    "if (x != null && x.length > 0) { return x[0]::parse(); }",
    "SELECT users.id, count(*) FROM users WHERE users.active = TRUE GROUP BY users.id;",
];

/// Generates a developer syntax practice string.
pub fn generate_code_drill(level: Level, length: RoundLength) -> Vec<char> {
    let target_words = length.word_count();

    match level {
        Level::Beginner => {
            // Synthetic Symbol Sprints: brackets and operators
            build_from_pool(SYMBOL_SPRINTS, target_words)
        }
        Level::Intermediate => {
            // Mixed modern language snippets (TS and Python)
            let mut pool = Vec::new();
            pool.extend_from_slice(TYPESCRIPT_CODE_SNIPPETS);
            pool.extend_from_slice(PYTHON_CODE_SNIPPETS);
            build_from_pool(&pool, target_words)
        }
        Level::Advanced => {
            // Advanced Rust idioms, generics, and lifetimes
            build_from_pool(RUST_CODE_SNIPPETS, target_words)
        }
    }
}

fn build_from_pool(pool: &[&str], count: usize) -> Vec<char> {
    let mut words = Vec::new();
    let mut pool_idx = 0;

    while words.len() < count {
        let snippet = pool[pool_idx % pool.len()];
        for token in snippet.split_whitespace() {
            words.push(token);
            if words.len() >= count {
                break;
            }
        }
        pool_idx += 1;
    }

    words.truncate(count);
    words.join(" ").chars().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_code_drill_symbols() {
        let drill = generate_code_drill(Level::Beginner, RoundLength::Words15);
        let text: String = drill.into_iter().collect();
        assert!(text.contains('{') || text.contains('[') || text.contains("->") || text.contains("=>"));
    }

    #[test]
    fn test_generate_code_drill_advanced_rust() {
        let drill = generate_code_drill(Level::Advanced, RoundLength::Words15);
        let text: String = drill.into_iter().collect();
        assert!(text.contains("fn") || text.contains("let") || text.contains("match") || text.contains("pub"));
    }
}
