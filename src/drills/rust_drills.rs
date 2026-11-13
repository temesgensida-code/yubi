//! Rust Programming Language Practice Engine & Snippet Repository.
//!
//! Provides comprehensive, authentic Rust syntax drill pools across 8 specialized
//! language topics, graduated across Beginner, Intermediate, and Advanced tiers.
//!
//! Snippets are concatenated generous to syntax: statements are never severed
//! mid-token or mid-expression just to hit an exact word count.

use crate::app::{Level, RoundLength, RustTopic};
use rand::seq::SliceRandom;
use rand::thread_rng;

// =========================================================================
// 1. Ownership, Borrowing & Lifetimes
// =========================================================================

pub const OWNERSHIP_BEGINNER: &[&str] = &[
    "let s: String = String::from(\"hello\"); let ref_s: &String = &s;",
    "let mut count: i32 = 42; let ref_mut: &mut i32 = &mut count; *ref_mut += 1;",
    "let val: u64 = 100; let ptr: *const u64 = &val as *const u64;",
    "let name = String::from(\"yubi\"); let borrowed: &str = &name[..];",
    "let greeting = \"hello world\"; let len: usize = greeting.len();",
    "let x: i32 = 10; let y = x; println!(\"x = {}, y = {}\", x, y);",
    "let mut text = String::from(\"buffer\"); text.push_str(\"_ready\");",
    "let raw_num: usize = 128; let num_ref: &usize = &raw_num;",
];

pub const OWNERSHIP_INTERMEDIATE: &[&str] = &[
    "fn calculate_len(s: &str) -> usize { s.trim().len() }",
    "fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { if x.len() > y.len() { x } else { y } }",
    "struct Buffer<'a> { data: &'a [u8], pos: usize }",
    "let text = String::from(\"tokio\"); let consumer = move || println!(\"moved: {}\", text);",
    "fn split_first<'a, T>(slice: &'a [T]) -> Option<(&'a T, &'a [T])> { slice.split_first() }",
    "let mut data = vec![1, 2, 3]; for item in &mut data { *item *= 2; }",
    "fn find_substr<'a>(haystack: &'a str, needle: &str) -> Option<&'a str> { haystack.find(needle).map(|i| &haystack[i..]) }",
    "pub struct RefHolder<'a> { value: &'a mut i32 }",
];

pub const OWNERSHIP_ADVANCED: &[&str] = &[
    "pub struct TokenStream<'a, 'b: 'a> { source: &'a str, lookahead: &'b [Token] }",
    "fn transform_context<'a, T>(ctx: &'a mut Context<T>) -> Result<&'a T, Error> where T: 'a + Send { Ok(&ctx.inner) }",
    "pub trait Parser<'de>: Sized { type Output: 'de; fn parse(input: &'de str) -> Result<Self::Output, ParseError>; }",
    "let worker = std::thread::spawn(move || { let mut buf = [0u8; 64]; process_stream(&mut buf) });",
    "pub struct RefWrapper<'a, T: ?Sized + 'a> { ptr: &'a T, marker: std::marker::PhantomData<&'a ()> }",
    "unsafe fn as_mut_slice<'a, T>(ptr: *mut T, len: usize) -> &'a mut [T] { std::slice::from_raw_parts_mut(ptr, len) }",
];

// =========================================================================
// 2. Smart Pointers & Interior Mutability
// =========================================================================

pub const POINTERS_BEGINNER: &[&str] = &[
    "let b: Box<i32> = Box::new(42); assert_eq!(*b, 42);",
    "let rc = std::rc::Rc::new(String::from(\"shared\")); let clone = std::rc::Rc::clone(&rc);",
    "let arc = std::sync::Arc::new(100u32); let shared = std::sync::Arc::clone(&arc);",
    "let cell = std::cell::Cell::new(10); cell.set(20); assert_eq!(cell.get(), 20);",
    "let cow: std::borrow::Cow<'static, str> = std::borrow::Cow::Borrowed(\"immutable\");",
    "let boxed_slice: Box<[u8]> = vec![0x10, 0x20, 0x30].into_boxed_slice();",
];

pub const POINTERS_INTERMEDIATE: &[&str] = &[
    "let refcell = std::cell::RefCell::new(vec![1, 2, 3]); refcell.borrow_mut().push(4);",
    "let mutex = std::sync::Mutex::new(0); { let mut lock = mutex.lock().unwrap(); *lock += 1; }",
    "let rwlock = std::sync::RwLock::new(HashMap::new()); let r = rwlock.read().unwrap();",
    "let weak: std::sync::Weak<String> = std::sync::Arc::downgrade(&arc); if let Some(strong) = weak.upgrade() { ... }",
    "fn process_text<'a>(input: &'a str) -> std::borrow::Cow<'a, str> { if input.contains(' ') { Cow::Owned(input.replace(' ', \"_\")) } else { Cow::Borrowed(input) } }",
    "let state: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::with_capacity(16)));",
];

pub const POINTERS_ADVANCED: &[&str] = &[
    "type SafeMap<K, V> = std::sync::Arc<std::sync::RwLock<std::collections::HashMap<K, V>>>;",
    "pub struct SharedState<T> { data: std::sync::Arc<std::sync::Mutex<Option<T>>>, notifier: std::sync::Condvar }",
    "let pool: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<Connection>>> = Arc::new(Mutex::new(VecDeque::with_capacity(32)));",
    "impl<T> Deref for CustomBox<T> { type Target = T; fn deref(&self) -> &Self::Target { unsafe { &*self.ptr.as_ptr() } } }",
    "let node = std::rc::Rc::new(std::cell::RefCell::new(Node { parent: std::rc::Rc::downgrade(&parent), children: Vec::new() }));",
    "pub struct AtomicRef<T> { inner: std::sync::atomic::AtomicPtr<T>, _phantom: std::marker::PhantomData<T> }",
];

// =========================================================================
// 3. Data Structures & Collections
// =========================================================================

pub const COLLECTIONS_BEGINNER: &[&str] = &[
    "let mut vec: Vec<i32> = Vec::new(); vec.push(10); vec.push(20);",
    "let mut map: HashMap<&str, i32> = HashMap::new(); map.insert(\"apple\", 3);",
    "let mut set: HashSet<u32> = HashSet::new(); set.insert(42);",
    "struct Point(f64, f64); let origin = Point(0.0, 0.0);",
    "enum Status { Active, Inactive, Pending } let s = Status::Active;",
    "let capacity_vec: Vec<String> = Vec::with_capacity(64);",
    "let numbers = vec![1, 2, 3, 4, 5]; assert_eq!(numbers.len(), 5);",
];

pub const COLLECTIONS_INTERMEDIATE: &[&str] = &[
    "let mut map = HashMap::new(); *map.entry(\"counter\").or_insert(0) += 1;",
    "let mut deque: VecDeque<Task> = VecDeque::with_capacity(64); deque.push_back(task); deque.pop_front();",
    "let btree: BTreeMap<String, Vec<u8>> = BTreeMap::new();",
    "let intersection: HashSet<_> = set_a.intersection(&set_b).cloned().collect();",
    "enum Message { Quit, Move { x: i32, y: i32 }, Write(String), ChangeColor(i32, i32, i32) }",
    "let mut registry: HashMap<u64, Vec<String>> = HashMap::with_capacity(128);",
    "let mut sorted_set: BTreeSet<i64> = BTreeSet::new(); sorted_set.insert(100);",
];

pub const COLLECTIONS_ADVANCED: &[&str] = &[
    "pub struct LRUCache<K: Hash + Eq + Clone, V> { map: HashMap<K, usize>, entries: VecDeque<(K, V)>, cap: usize }",
    "let mut trie: HashMap<char, Box<TrieNode>> = HashMap::with_capacity(26);",
    "let chunked: Vec<Vec<u8>> = stream.chunks(4096).map(|c| c.to_vec()).collect::<Vec<_>>();",
    "pub struct RingBuffer<T, const N: usize> { storage: [Option<T>; N], head: usize, tail: usize }",
    "let index: BTreeMap<&'static str, Vec<(u32, f64)>> = BTreeMap::from([(\"cpu\", vec![(1, 0.95)])]);",
    "pub struct Graph<N, E> { nodes: Vec<N>, edges: HashMap<usize, Vec<(usize, E)>> }",
];

// =========================================================================
// 4. Pattern Matching & Control Flow
// =========================================================================

pub const PATTERNS_BEGINNER: &[&str] = &[
    "if let Some(val) = maybe_val { println!(\"found: {}\", val); }",
    "match number { 1 => \"one\", 2 => \"two\", 3 => \"three\", _ => \"many\" };",
    "let is_positive = match x { n if n > 0 => true, _ => false };",
    "while let Some(item) = queue.pop() { handle(item); }",
    "let (a, b) = (10, 20); assert_eq!(a + b, 30);",
    "if let Ok(result) = parse_val(input) { return result; }",
];

pub const PATTERNS_INTERMEDIATE: &[&str] = &[
    "match opt_result { Some(Ok(v)) => process(v), Some(Err(e)) => log_err(e), None => () }",
    "let Some(config) = load_config() else { return Err(AppError::MissingConfig); };",
    "match event { Event::KeyPress(k) if k.is_ctrl() => handle_ctrl(k), Event::Resize(w, h) => resize(w, h), _ => () }",
    "let Point { x, y: 0 } = target else { return false; };",
    "match &slice[..] { [] => 0, [single] => *single, [first, .., last] => first + last }",
    "let Ok(token) = auth_header.strip_prefix(\"Bearer \").ok_or(AuthError::InvalidHeader) else { return unauthorized(); };",
];

pub const PATTERNS_ADVANCED: &[&str] = &[
    "match (&prev_state, &next_state) { (State::Init, State::Running(pid)) if *pid > 0 => resume(*pid), (State::Running(_), State::Stopped(0)) => Ok(()), (_, State::Failed(code)) => Err(Error::Crash(*code)), _ => unreachable!() }",
    "let Ok(Packet::V2 { header: Header { id, flags, .. }, payload }) = parse(&raw) else { return None; };",
    "for &[ref k, ref v] in pairs.chunks_exact(2) { registry.insert(k.clone(), v.clone()); }",
    "match ast_node { Expr::Binary { op: BinOp::Add | BinOp::Sub, lhs, rhs } => eval_arithmetic(lhs, rhs), Expr::Call { callee, args, .. } => eval_call(callee, args), _ => () }",
    "if let (Some(a), Some(b), ref tail) = (iter.next(), iter.next(), iter.as_slice()) { compute_pair(a, b, tail); }",
];

// =========================================================================
// 5. Error Handling (Result & Option)
// =========================================================================

pub const ERRORS_BEGINNER: &[&str] = &[
    "let val: Result<i32, ParseIntError> = \"123\".parse::<i32>();",
    "let count = opt.unwrap_or(0);",
    "let mapped = res.map(|x| x * 2);",
    "let unwrapped = result.expect(\"failed to load resource\");",
    "let ok_val: Option<i32> = res.ok();",
    "let default_str = maybe_name.unwrap_or_else(|| String::from(\"anonymous\"));",
];

pub const ERRORS_INTERMEDIATE: &[&str] = &[
    "fn read_username(path: &Path) -> Result<String, io::Error> { let mut file = File::open(path)?; let mut s = String::new(); file.read_to_string(&mut s)?; Ok(s) }",
    "let user = find_user(id).ok_or_else(|| AppError::NotFound(id))?;",
    "let parsed = raw.parse::<u64>().map_err(|e| ParseError::InvalidFormat(e))?;",
    "let res: anyhow::Result<Config> = serde_json::from_str(&data).context(\"failed parsing JSON\");",
    "match std::fs::remove_file(path) { Ok(()) => (), Err(e) if e.kind() == io::ErrorKind::NotFound => (), Err(e) => return Err(e), }",
    "let port = std::env::var(\"PORT\").ok().and_then(|p| p.parse::<u16>().ok()).unwrap_or(8080);",
];

pub const ERRORS_ADVANCED: &[&str] = &[
    "#[derive(thiserror::Error, Debug)] pub enum NetworkError { #[error(\"io failed: {0}\")] Io(#[from] io::Error), #[error(\"timeout after {secs}s\")] Timeout { secs: u64 }, #[error(\"unknown: {0}\")] Custom(String), }",
    "impl fmt::Display for CustomError { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, \"[code {}] {}\", self.code, self.message) } }",
    "fn execute_pipeline() -> Result<Summary, Box<dyn std::error::Error + Send + Sync>> { let stage1 = init_stage()?; let stage2 = run_stage(stage1)?; Ok(stage2.summarize()) }",
    "pub type ApiResult<T> = std::result::Result<T, (StatusCode, Json<ErrorResponse>)>;",
    "impl std::error::Error for ParseError { fn source(&self) -> Option<&(dyn std::error::Error + 'static)> { self.cause.as_deref() } }",
];

// =========================================================================
// 6. Traits, Generics & Bounds
// =========================================================================

pub const TRAITS_BEGINNER: &[&str] = &[
    "#[derive(Debug, Clone, PartialEq, Eq)] pub struct UserId(pub u64);",
    "fn print_value<T: std::fmt::Display>(val: T) { println!(\"{}\", val); }",
    "trait Greeter { fn greet(&self) -> String; }",
    "impl Greeter for String { fn greet(&self) -> String { format!(\"Hello, {}!\", self) } }",
    "fn make_pair<T>(a: T, b: T) -> (T, T) { (a, b) }",
    "#[derive(Default, Copy, Clone)] struct Config { pub timeout_ms: u64 }",
];

pub const TRAITS_INTERMEDIATE: &[&str] = &[
    "pub trait Repository<T, Id>: Send + Sync where Id: Hash + Eq { fn find_by_id(&self, id: &Id) -> Option<T>; fn save(&mut self, entity: T) -> Result<(), RepoError>; }",
    "fn parse_stream<T: FromStr>(input: &str) -> impl Iterator<Item = Result<T, T::Err>> + '_ { input.lines().map(str::parse) }",
    "pub fn render_widgets(widgets: &[Box<dyn Widget>]) { for w in widgets { w.render(); } }",
    "impl<T: Clone> Clone for Node<T> { fn clone(&self) -> Self { Node { val: self.val.clone(), next: self.next.clone() } } }",
    "#[derive(Serialize, Deserialize, Default, Hash)] #[serde(rename_all = \"camelCase\")] struct Payload { token: String }",
    "pub trait Deserializer<'de>: Sized { fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>; }",
];

pub const TRAITS_ADVANCED: &[&str] = &[
    "pub trait Service<Request> { type Response; type Error; type Future: Future<Output = Result<Self::Response, Self::Error>>; fn call(&mut self, req: Request) -> Self::Future; }",
    "pub struct Wrapper<T> where T: for<'a> Deserialize<'a> + Send + 'static { inner: Arc<T> }",
    "fn serialize_generic<S, T>(val: &T, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer, T: Serialize + ?Sized { val.serialize(serializer) }",
    "pub trait Component: 'static { type State: Default + Send + Sync; fn update(&mut self, msg: Message) -> Command<Self::State>; }",
    "impl<T, const N: usize> AsRef<[T]> for FixedArray<T, N> { fn as_ref(&self) -> &[T] { &self.data } }",
];

// =========================================================================
// 7. Iterators & Closures
// =========================================================================

pub const ITERATORS_BEGINNER: &[&str] = &[
    "let squares: Vec<i32> = (1..=10).map(|x| x * x).collect();",
    "let sum: i32 = vec![1, 2, 3, 4, 5].into_iter().sum();",
    "let evens: Vec<_> = numbers.into_iter().filter(|n| n % 2 == 0).collect();",
    "let add_five = |x: i32| -> i32 { x + 5 }; assert_eq!(add_five(10), 15);",
    "for (idx, ch) in \"hello\".chars().enumerate() { println!(\"{}: {}\", idx, ch); }",
    "let any_match = flags.iter().any(|&f| f == target_flag);",
];

pub const ITERATORS_INTERMEDIATE: &[&str] = &[
    "let res: Vec<String> = items.iter().filter(|x| x.is_valid()).map(|x| x.name.to_uppercase()).collect::<Vec<_>>();",
    "let zipped: HashMap<_, _> = keys.into_iter().zip(values.into_iter()).collect();",
    "let flattened: Vec<u32> = nested.into_iter().flatten().collect();",
    "let total_score = players.iter().fold(0.0, |acc, p| acc + p.score);",
    "fn apply_callback<F>(mut f: F) where F: FnMut(usize) -> bool { for i in 0..10 { if !f(i) { break; } } }",
    "let pairs: Vec<_> = data.chunks_exact(2).map(|c| (c[0], c[1])).collect();",
];

pub const ITERATORS_ADVANCED: &[&str] = &[
    "let grouped: HashMap<Category, Vec<Item>> = items.into_iter().fold(HashMap::new(), |mut acc, item| { acc.entry(item.category).or_default().push(item); acc });",
    "pub fn sliding_windows<'a, T>(slice: &'a [T], window_size: usize) -> impl Iterator<Item = &'a [T]> { slice.windows(window_size) }",
    "let matrix_transposed: Vec<Vec<T>> = (0..cols).map(|c| (0..rows).map(|r| matrix[r][c].clone()).collect()).collect();",
    "let active_ids: HashSet<u64> = records.iter().filter_map(|r| r.as_ref().ok()).filter(|r| r.is_active).map(|r| r.id).collect();",
    "let deduplicated: Vec<_> = stream.scan(None, |prev, curr| { let is_dup = *prev == Some(curr); *prev = Some(curr); Some((curr, is_dup)) }).filter(|&(_, dup)| !dup).map(|(x, _)| x).collect();",
];

// =========================================================================
// 8. Async, Concurrency & Macros
// =========================================================================

pub const ASYNC_BEGINNER: &[&str] = &[
    "println!(\"User count: {}, active: {}\", users.len(), active);",
    "let numbers: Vec<i32> = vec![10, 20, 30, 40, 50];",
    "let msg = format!(\"error code: {}\", 404);",
    "assert!(matches!(status, Status::Ready | Status::Completed));",
    "std::thread::sleep(std::time::Duration::from_millis(50));",
    "eprintln!(\"warning: operation timed out after {}ms\", elapsed);",
];

pub const ASYNC_INTERMEDIATE: &[&str] = &[
    "let handle = std::thread::spawn(move || { compute_heavy_task() }); let result = handle.join().unwrap();",
    "let (tx, rx) = std::sync::mpsc::channel(); tx.send(Event::Ping).unwrap();",
    "async fn fetch_user(client: &reqwest::Client, url: &str) -> Result<User, reqwest::Error> { client.get(url).send().await?.json::<User>().await }",
    "let mut stream = tokio::time::interval(std::time::Duration::from_secs(1)); stream.tick().await;",
    "macro_rules! map { ($($k:expr => $v:expr),* $(,)?) => {{ let mut m = std::collections::HashMap::new(); $(m.insert($k, $v);)* m }}; }",
    "tokio::spawn(async move { if let Err(e) = process_socket(socket).await { log_error(e); } });",
];

pub const ASYNC_ADVANCED: &[&str] = &[
    "pub async fn run_worker_pool<T: Send + 'static>(mut rx: mpsc::Receiver<T>, workers: usize) { let rx = Arc::new(Mutex::new(rx)); for _ in 0..workers { let rx = Arc::clone(&rx); tokio::spawn(async move { while let Some(task) = { rx.lock().await.recv().await } { task.execute().await; } }); } }",
    "tokio::select! { val = rx.recv() => handle_msg(val), _ = tokio::time::sleep(timeout) => handle_timeout(), }",
    "macro_rules! impl_display { ($($t:ty),*) => { $(impl std::fmt::Display for $t { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, \"{}({})\", stringify!($t), self.0) } })* }; }",
    "let (barrier, notify) = (Arc::new(Barrier::new(4)), Arc::new(Notify::new()));",
    "pub struct BroadcastChannel<T: Clone> { sender: tokio::sync::broadcast::Sender<T> }",
];

// =========================================================================
// Snippet Selection & Generation Engine
// =========================================================================

/// Returns the slice of snippets for a specific topic and level.
pub fn get_snippets_for(topic: RustTopic, level: Level) -> Vec<&'static str> {
    match topic {
        RustTopic::OwnershipLifetimes => match level {
            Level::Beginner => OWNERSHIP_BEGINNER.to_vec(),
            Level::Intermediate => OWNERSHIP_INTERMEDIATE.to_vec(),
            Level::Advanced => OWNERSHIP_ADVANCED.to_vec(),
        },
        RustTopic::SmartPointers => match level {
            Level::Beginner => POINTERS_BEGINNER.to_vec(),
            Level::Intermediate => POINTERS_INTERMEDIATE.to_vec(),
            Level::Advanced => POINTERS_ADVANCED.to_vec(),
        },
        RustTopic::Collections => match level {
            Level::Beginner => COLLECTIONS_BEGINNER.to_vec(),
            Level::Intermediate => COLLECTIONS_INTERMEDIATE.to_vec(),
            Level::Advanced => COLLECTIONS_ADVANCED.to_vec(),
        },
        RustTopic::PatternMatching => match level {
            Level::Beginner => PATTERNS_BEGINNER.to_vec(),
            Level::Intermediate => PATTERNS_INTERMEDIATE.to_vec(),
            Level::Advanced => PATTERNS_ADVANCED.to_vec(),
        },
        RustTopic::ErrorHandling => match level {
            Level::Beginner => ERRORS_BEGINNER.to_vec(),
            Level::Intermediate => ERRORS_INTERMEDIATE.to_vec(),
            Level::Advanced => ERRORS_ADVANCED.to_vec(),
        },
        RustTopic::TraitsGenerics => match level {
            Level::Beginner => TRAITS_BEGINNER.to_vec(),
            Level::Intermediate => TRAITS_INTERMEDIATE.to_vec(),
            Level::Advanced => TRAITS_ADVANCED.to_vec(),
        },
        RustTopic::IteratorsClosures => match level {
            Level::Beginner => ITERATORS_BEGINNER.to_vec(),
            Level::Intermediate => ITERATORS_INTERMEDIATE.to_vec(),
            Level::Advanced => ITERATORS_ADVANCED.to_vec(),
        },
        RustTopic::AsyncMacros => match level {
            Level::Beginner => ASYNC_BEGINNER.to_vec(),
            Level::Intermediate => ASYNC_INTERMEDIATE.to_vec(),
            Level::Advanced => ASYNC_ADVANCED.to_vec(),
        },
        RustTopic::All => {
            let mut all = Vec::new();
            for sub_topic in RustTopic::ALL.iter().filter(|t| **t != RustTopic::All) {
                all.extend(get_snippets_for(*sub_topic, level));
            }
            all
        }
    }
}

/// Generates a Rust practice string sized generous to syntax.
///
/// Complete statements are concatenated until `length.word_count()` is reached.
/// Statements are NEVER cut in the middle.
pub fn generate_rust_drill(topic: RustTopic, level: Level, length: RoundLength) -> Vec<char> {
    let target_words = length.word_count();
    let mut pool = get_snippets_for(topic, level);

    let mut rng = thread_rng();
    pool.shuffle(&mut rng);

    let mut chosen_snippets: Vec<&str> = Vec::new();
    let mut word_count = 0;
    let mut pool_idx = 0;

    while word_count < target_words && chosen_snippets.len() < pool.len() * 4 {
        let snippet = pool[pool_idx % pool.len()];
        chosen_snippets.push(snippet);
        word_count += snippet.split_whitespace().count();
        pool_idx += 1;
    }

    chosen_snippets.join(" ").chars().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_topics_have_non_empty_pools_across_levels() {
        for topic in RustTopic::ALL.iter() {
            for level in Level::ALL.iter() {
                let snippets = get_snippets_for(*topic, *level);
                assert!(
                    !snippets.is_empty(),
                    "Topic {:?} at Level {:?} must have non-empty snippet pool",
                    topic,
                    level
                );
            }
        }
    }

    #[test]
    fn test_generate_rust_drill_generous_length() {
        for length in RoundLength::ALL.iter() {
            let drill = generate_rust_drill(RustTopic::OwnershipLifetimes, Level::Intermediate, *length);
            let text: String = drill.iter().collect();
            let count = text.split_whitespace().count();
            // Count must be at least target_words, never cut short
            assert!(
                count >= length.word_count(),
                "Expected at least {} words for {:?}, got {}",
                length.word_count(),
                length,
                count
            );
        }
    }

    #[test]
    fn test_all_topic_selection_contains_diverse_syntax() {
        let drill = generate_rust_drill(RustTopic::All, Level::Advanced, RoundLength::Words50);
        let text: String = drill.into_iter().collect();
        assert!(text.contains("fn") || text.contains("let") || text.contains("pub") || text.contains("impl"));
    }
}
