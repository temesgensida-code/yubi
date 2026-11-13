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
    "let s: String = String::from(\"hello\");\nlet ref_s: &String = &s;",
    "let mut count: i32 = 42;\nlet ref_mut: &mut i32 = &mut count;\n*ref_mut += 1;",
    "let val: u64 = 100;\nlet ptr: *const u64 = &val as *const u64;",
    "let name = String::from(\"yubi\");\nlet borrowed: &str = &name[..];",
    "let greeting = \"hello world\";\nlet len: usize = greeting.len();",
    "let x: i32 = 10;\nlet y = x;\nprintln!(\"x = {}, y = {}\", x, y);",
    "let mut text = String::from(\"buffer\");\ntext.push_str(\"_ready\");",
    "let raw_num: usize = 128;\nlet num_ref: &usize = &raw_num;",
];

pub const OWNERSHIP_INTERMEDIATE: &[&str] = &[
    "fn calculate_len(s: &str) -> usize {\n    s.trim().len()\n}",
    "fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {\n    if x.len() > y.len() {\n        x\n    } else {\n        y\n    }\n}",
    "struct Buffer<'a> {\n    data: &'a [u8],\n    pos: usize,\n}",
    "let text = String::from(\"tokio\");\nlet consumer = move || println!(\"moved: {}\", text);",
    "fn split_first<'a, T>(slice: &'a [T]) -> Option<(&'a T, &'a [T])> {\n    slice.split_first()\n}",
    "let mut data = vec![1, 2, 3];\nfor item in &mut data {\n    *item *= 2;\n}",
    "fn find_substr<'a>(haystack: &'a str, needle: &str) -> Option<&'a str> {\n    haystack.find(needle).map(|i| &haystack[i..])\n}",
    "pub struct RefHolder<'a> {\n    value: &'a mut i32,\n}",
];

pub const OWNERSHIP_ADVANCED: &[&str] = &[
    "pub struct TokenStream<'a, 'b: 'a> {\n    source: &'a str,\n    lookahead: &'b [Token],\n}",
    "fn transform_context<'a, T>(ctx: &'a mut Context<T>) -> Result<&'a T, Error>\nwhere\n    T: 'a + Send,\n{\n    Ok(&ctx.inner)\n}",
    "pub trait Parser<'de>: Sized {\n    type Output: 'de;\n    fn parse(input: &'de str) -> Result<Self::Output, ParseError>;\n}",
    "let worker = std::thread::spawn(move || {\n    let mut buf = [0u8; 64];\n    process_stream(&mut buf)\n});",
    "pub struct RefWrapper<'a, T: ?Sized + 'a> {\n    ptr: &'a T,\n    marker: std::marker::PhantomData<&'a ()>,\n}",
    "unsafe fn as_mut_slice<'a, T>(ptr: *mut T, len: usize) -> &'a mut [T] {\n    std::slice::from_raw_parts_mut(ptr, len)\n}",
];

// =========================================================================
// 2. Smart Pointers & Interior Mutability
// =========================================================================

pub const POINTERS_BEGINNER: &[&str] = &[
    "let b: Box<i32> = Box::new(42);\nassert_eq!(*b, 42);",
    "let rc = std::rc::Rc::new(String::from(\"shared\"));\nlet clone = std::rc::Rc::clone(&rc);",
    "let arc = std::sync::Arc::new(100u32);\nlet shared = std::sync::Arc::clone(&arc);",
    "let cell = std::cell::Cell::new(10);\ncell.set(20);\nassert_eq!(cell.get(), 20);",
    "let cow: std::borrow::Cow<'static, str> = std::borrow::Cow::Borrowed(\"immutable\");",
    "let boxed_slice: Box<[u8]> = vec![0x10, 0x20, 0x30].into_boxed_slice();",
];

pub const POINTERS_INTERMEDIATE: &[&str] = &[
    "let refcell = std::cell::RefCell::new(vec![1, 2, 3]);\nrefcell.borrow_mut().push(4);",
    "let mutex = std::sync::Mutex::new(0);\n{\n    let mut lock = mutex.lock().unwrap();\n    *lock += 1;\n}",
    "let rwlock = std::sync::RwLock::new(HashMap::new());\nlet r = rwlock.read().unwrap();",
    "let weak: std::sync::Weak<String> = std::sync::Arc::downgrade(&arc);\nif let Some(strong) = weak.upgrade() {\n    println!(\"strong: {}\", strong);\n}",
    "fn process_text<'a>(input: &'a str) -> std::borrow::Cow<'a, str> {\n    if input.contains(' ') {\n        Cow::Owned(input.replace(' ', \"_\"))\n    } else {\n        Cow::Borrowed(input)\n    }\n}",
    "let state: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::with_capacity(16)));",
];

pub const POINTERS_ADVANCED: &[&str] = &[
    "type SafeMap<K, V> = std::sync::Arc<std::sync::RwLock<std::collections::HashMap<K, V>>>;",
    "pub struct SharedState<T> {\n    data: std::sync::Arc<std::sync::Mutex<Option<T>>>,\n    notifier: std::sync::Condvar,\n}",
    "let pool: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<Connection>>> =\n    Arc::new(Mutex::new(VecDeque::with_capacity(32)));",
    "impl<T> Deref for CustomBox<T> {\n    type Target = T;\n    fn deref(&self) -> &Self::Target {\n        unsafe { &*self.ptr.as_ptr() }\n    }\n}",
    "let node = std::rc::Rc::new(std::cell::RefCell::new(Node {\n    parent: std::rc::Rc::downgrade(&parent),\n    children: Vec::new(),\n}));",
    "pub struct AtomicRef<T> {\n    inner: std::sync::atomic::AtomicPtr<T>,\n    _phantom: std::marker::PhantomData<T>,\n}",
];

// =========================================================================
// 3. Data Structures & Collections
// =========================================================================

pub const COLLECTIONS_BEGINNER: &[&str] = &[
    "let mut vec: Vec<i32> = Vec::new();\nvec.push(10);\nvec.push(20);",
    "let mut map: HashMap<&str, i32> = HashMap::new();\nmap.insert(\"apple\", 3);",
    "let mut set: HashSet<u32> = HashSet::new();\nset.insert(42);",
    "struct Point(f64, f64);\nlet origin = Point(0.0, 0.0);",
    "enum Status { Active, Inactive, Pending }\nlet s = Status::Active;",
    "let capacity_vec: Vec<String> = Vec::with_capacity(64);",
    "let numbers = vec![1, 2, 3, 4, 5];\nassert_eq!(numbers.len(), 5);",
];

pub const COLLECTIONS_INTERMEDIATE: &[&str] = &[
    "let mut map = HashMap::new();\n*map.entry(\"counter\").or_insert(0) += 1;",
    "let mut deque: VecDeque<Task> = VecDeque::with_capacity(64);\ndeque.push_back(task);\ndeque.pop_front();",
    "let btree: BTreeMap<String, Vec<u8>> = BTreeMap::new();",
    "let intersection: HashSet<_> = set_a.intersection(&set_b).cloned().collect();",
    "enum Message {\n    Quit,\n    Move { x: i32, y: i32 },\n    Write(String),\n    ChangeColor(i32, i32, i32),\n}",
    "let mut registry: HashMap<u64, Vec<String>> = HashMap::with_capacity(128);",
    "let mut sorted_set: BTreeSet<i64> = BTreeSet::new();\nsorted_set.insert(100);",
];

pub const COLLECTIONS_ADVANCED: &[&str] = &[
    "pub struct LRUCache<K: Hash + Eq + Clone, V> {\n    map: HashMap<K, usize>,\n    entries: VecDeque<(K, V)>,\n    cap: usize,\n}",
    "let mut trie: HashMap<char, Box<TrieNode>> = HashMap::with_capacity(26);",
    "let chunked: Vec<Vec<u8>> = stream.chunks(4096)\n    .map(|c| c.to_vec())\n    .collect::<Vec<_>>();",
    "pub struct RingBuffer<T, const N: usize> {\n    storage: [Option<T>; N],\n    head: usize,\n    tail: usize,\n}",
    "let index: BTreeMap<&'static str, Vec<(u32, f64)>> = BTreeMap::from([\n    (\"cpu\", vec![(1, 0.95)]),\n]);",
    "pub struct Graph<N, E> {\n    nodes: Vec<N>,\n    edges: HashMap<usize, Vec<(usize, E)>>,\n}",
];

// =========================================================================
// 4. Pattern Matching & Control Flow
// =========================================================================

pub const PATTERNS_BEGINNER: &[&str] = &[
    "if let Some(val) = maybe_val {\n    println!(\"found: {}\", val);\n}",
    "match number {\n    1 => \"one\",\n    2 => \"two\",\n    3 => \"three\",\n    _ => \"many\",\n};",
    "let is_positive = match x {\n    n if n > 0 => true,\n    _ => false,\n};",
    "while let Some(item) = queue.pop() {\n    handle(item);\n}",
    "let (a, b) = (10, 20);\nassert_eq!(a + b, 30);",
    "if let Ok(result) = parse_val(input) {\n    return result;\n}",
];

pub const PATTERNS_INTERMEDIATE: &[&str] = &[
    "match opt_result {\n    Some(Ok(v)) => process(v),\n    Some(Err(e)) => log_err(e),\n    None => (),\n}",
    "let Some(config) = load_config() else {\n    return Err(AppError::MissingConfig);\n};",
    "match event {\n    Event::KeyPress(k) if k.is_ctrl() => handle_ctrl(k),\n    Event::Resize(w, h) => resize(w, h),\n    _ => (),\n}",
    "let Point { x, y: 0 } = target else {\n    return false;\n};",
    "match &slice[..] {\n    [] => 0,\n    [single] => *single,\n    [first, .., last] => first + last,\n}",
    "let Ok(token) = auth_header.strip_prefix(\"Bearer \")\n    .ok_or(AuthError::InvalidHeader) else {\n    return unauthorized();\n};",
];

pub const PATTERNS_ADVANCED: &[&str] = &[
    "match (&prev_state, &next_state) {\n    (State::Init, State::Running(pid)) if *pid > 0 => resume(*pid),\n    (State::Running(_), State::Stopped(0)) => Ok(()),\n    (_, State::Failed(code)) => Err(Error::Crash(*code)),\n    _ => unreachable!(),\n}",
    "let Ok(Packet::V2 { header: Header { id, flags, .. }, payload }) = parse(&raw) else {\n    return None;\n};",
    "for &[ref k, ref v] in pairs.chunks_exact(2) {\n    registry.insert(k.clone(), v.clone());\n}",
    "match ast_node {\n    Expr::Binary { op: BinOp::Add | BinOp::Sub, lhs, rhs } => eval_arithmetic(lhs, rhs),\n    Expr::Call { callee, args, .. } => eval_call(callee, args),\n    _ => (),\n}",
    "if let (Some(a), Some(b), ref tail) = (iter.next(), iter.next(), iter.as_slice()) {\n    compute_pair(a, b, tail);\n}",
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
    "fn read_username(path: &Path) -> Result<String, io::Error> {\n    let mut file = File::open(path)?;\n    let mut s = String::new();\n    file.read_to_string(&mut s)?;\n    Ok(s)\n}",
    "let user = find_user(id)\n    .ok_or_else(|| AppError::NotFound(id))?;",
    "let parsed = raw.parse::<u64>()\n    .map_err(|e| ParseError::InvalidFormat(e))?;",
    "let res: anyhow::Result<Config> = serde_json::from_str(&data)\n    .context(\"failed parsing JSON\");",
    "match std::fs::remove_file(path) {\n    Ok(()) => (),\n    Err(e) if e.kind() == io::ErrorKind::NotFound => (),\n    Err(e) => return Err(e),\n}",
    "let port = std::env::var(\"PORT\").ok()\n    .and_then(|p| p.parse::<u16>().ok())\n    .unwrap_or(8080);",
];

pub const ERRORS_ADVANCED: &[&str] = &[
    "#[derive(thiserror::Error, Debug)]\npub enum NetworkError {\n    #[error(\"io failed: {0}\")]\n    Io(#[from] io::Error),\n    #[error(\"timeout after {secs}s\")]\n    Timeout { secs: u64 },\n    #[error(\"unknown: {0}\")]\n    Custom(String),\n}",
    "impl fmt::Display for CustomError {\n    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {\n        write!(f, \"[code {}] {}\", self.code, self.message)\n    }\n}",
    "fn execute_pipeline() -> Result<Summary, Box<dyn std::error::Error + Send + Sync>> {\n    let stage1 = init_stage()?;\n    let stage2 = run_stage(stage1)?;\n    Ok(stage2.summarize())\n}",
    "pub type ApiResult<T> = std::result::Result<T, (StatusCode, Json<ErrorResponse>)>;",
    "impl std::error::Error for ParseError {\n    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {\n        self.cause.as_deref()\n    }\n}",
];

// =========================================================================
// 6. Traits, Generics & Bounds
// =========================================================================

pub const TRAITS_BEGINNER: &[&str] = &[
    "#[derive(Debug, Clone, PartialEq, Eq)]\npub struct UserId(pub u64);",
    "fn print_value<T: std::fmt::Display>(val: T) {\n    println!(\"{}\", val);\n}",
    "trait Greeter {\n    fn greet(&self) -> String;\n}",
    "impl Greeter for String {\n    fn greet(&self) -> String {\n        format!(\"Hello, {}!\", self)\n    }\n}",
    "fn make_pair<T>(a: T, b: T) -> (T, T) {\n    (a, b)\n}",
    "#[derive(Default, Copy, Clone)]\nstruct Config {\n    pub timeout_ms: u64,\n}",
];

pub const TRAITS_INTERMEDIATE: &[&str] = &[
    "pub trait Repository<T, Id>: Send + Sync\nwhere\n    Id: Hash + Eq,\n{\n    fn find_by_id(&self, id: &Id) -> Option<T>;\n    fn save(&mut self, entity: T) -> Result<(), RepoError>;\n}",
    "fn parse_stream<T: FromStr>(input: &str) -> impl Iterator<Item = Result<T, T::Err>> + '_ {\n    input.lines().map(str::parse)\n}",
    "pub fn render_widgets(widgets: &[Box<dyn Widget>]) {\n    for w in widgets {\n        w.render();\n    }\n}",
    "impl<T: Clone> Clone for Node<T> {\n    fn clone(&self) -> Self {\n        Node {\n            val: self.val.clone(),\n            next: self.next.clone(),\n        }\n    }\n}",
    "#[derive(Serialize, Deserialize, Default, Hash)]\n#[serde(rename_all = \"camelCase\")]\nstruct Payload {\n    token: String,\n}",
    "pub trait Deserializer<'de>: Sized {\n    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error>;\n}",
];

pub const TRAITS_ADVANCED: &[&str] = &[
    "pub trait Service<Request> {\n    type Response;\n    type Error;\n    type Future: Future<Output = Result<Self::Response, Self::Error>>;\n    fn call(&mut self, req: Request) -> Self::Future;\n}",
    "pub struct Wrapper<T>\nwhere\n    T: for<'a> Deserialize<'a> + Send + 'static,\n{\n    inner: Arc<T>,\n}",
    "fn serialize_generic<S, T>(val: &T, serializer: S) -> Result<S::Ok, S::Error>\nwhere\n    S: Serializer,\n    T: Serialize + ?Sized,\n{\n    val.serialize(serializer)\n}",
    "pub trait Component: 'static {\n    type State: Default + Send + Sync;\n    fn update(&mut self, msg: Message) -> Command<Self::State>;\n}",
    "impl<T, const N: usize> AsRef<[T]> for FixedArray<T, N> {\n    fn as_ref(&self) -> &[T] {\n        &self.data\n    }\n}",
];

// =========================================================================
// 7. Iterators & Closures
// =========================================================================

pub const ITERATORS_BEGINNER: &[&str] = &[
    "let squares: Vec<i32> = (1..=10).map(|x| x * x).collect();",
    "let sum: i32 = vec![1, 2, 3, 4, 5].into_iter().sum();",
    "let evens: Vec<_> = numbers.into_iter().filter(|n| n % 2 == 0).collect();",
    "let add_five = |x: i32| -> i32 { x + 5 };\nassert_eq!(add_five(10), 15);",
    "for (idx, ch) in \"hello\".chars().enumerate() {\n    println!(\"{}: {}\", idx, ch);\n}",
    "let any_match = flags.iter().any(|&f| f == target_flag);",
];

pub const ITERATORS_INTERMEDIATE: &[&str] = &[
    "let res: Vec<String> = items.iter()\n    .filter(|x| x.is_valid())\n    .map(|x| x.name.to_uppercase())\n    .collect::<Vec<_>>();",
    "let zipped: HashMap<_, _> = keys.into_iter()\n    .zip(values.into_iter())\n    .collect();",
    "let flattened: Vec<u32> = nested.into_iter()\n    .flatten()\n    .collect();",
    "let total_score = players.iter()\n    .fold(0.0, |acc, p| acc + p.score);",
    "fn apply_callback<F>(mut f: F)\nwhere\n    F: FnMut(usize) -> bool,\n{\n    for i in 0..10 {\n        if !f(i) {\n            break;\n        }\n    }\n}",
    "let pairs: Vec<_> = data.chunks_exact(2)\n    .map(|c| (c[0], c[1]))\n    .collect();",
];

pub const ITERATORS_ADVANCED: &[&str] = &[
    "let grouped: HashMap<Category, Vec<Item>> = items.into_iter().fold(\n    HashMap::new(),\n    |mut acc, item| {\n        acc.entry(item.category).or_default().push(item);\n        acc\n    },\n);",
    "pub fn sliding_windows<'a, T>(slice: &'a [T], window_size: usize) -> impl Iterator<Item = &'a [T]> {\n    slice.windows(window_size)\n}",
    "let matrix_transposed: Vec<Vec<T>> = (0..cols)\n    .map(|c| (0..rows).map(|r| matrix[r][c].clone()).collect())\n    .collect();",
    "let active_ids: HashSet<u64> = records.iter()\n    .filter_map(|r| r.as_ref().ok())\n    .filter(|r| r.is_active)\n    .map(|r| r.id)\n    .collect();",
    "let deduplicated: Vec<_> = stream.scan(None, |prev, curr| {\n    let is_dup = *prev == Some(curr);\n    *prev = Some(curr);\n    Some((curr, is_dup))\n})\n.filter(|&(_, dup)| !dup)\n.map(|(x, _)| x)\n.collect();",
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
    "let handle = std::thread::spawn(move || {\n    compute_heavy_task()\n});\nlet result = handle.join().unwrap();",
    "let (tx, rx) = std::sync::mpsc::channel();\ntx.send(Event::Ping).unwrap();",
    "async fn fetch_user(client: &reqwest::Client, url: &str) -> Result<User, reqwest::Error> {\n    client.get(url).send().await?.json::<User>().await\n}",
    "let mut stream = tokio::time::interval(std::time::Duration::from_secs(1));\nstream.tick().await;",
    "macro_rules! map {\n    ($($k:expr => $v:expr),* $(,)?) => {{\n        let mut m = std::collections::HashMap::new();\n        $(m.insert($k, $v);)*\n        m\n    }};\n}",
    "tokio::spawn(async move {\n    if let Err(e) = process_socket(socket).await {\n        log_error(e);\n    }\n});",
];

pub const ASYNC_ADVANCED: &[&str] = &[
    "pub async fn run_worker_pool<T: Send + 'static>(mut rx: mpsc::Receiver<T>, workers: usize) {\n    let rx = Arc::new(Mutex::new(rx));\n    for _ in 0..workers {\n        let rx = Arc::clone(&rx);\n        tokio::spawn(async move {\n            while let Some(task) = { rx.lock().await.recv().await } {\n                task.execute().await;\n            }\n        });\n    }\n}",
    "tokio::select! {\n    val = rx.recv() => handle_msg(val),\n    _ = tokio::time::sleep(timeout) => handle_timeout(),\n}",
    "macro_rules! impl_display {\n    ($($t:ty),*) => {\n        $(impl std::fmt::Display for $t {\n            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n                write!(f, \"{}({})\", stringify!($t), self.0)\n            }\n        })*\n    };\n}",
    "let (barrier, notify) = (\n    Arc::new(Barrier::new(4)),\n    Arc::new(Notify::new()),\n);",
    "pub struct BroadcastChannel<T: Clone> {\n    sender: tokio::sync::broadcast::Sender<T>,\n}",
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

    chosen_snippets.join("\n").chars().collect()
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
