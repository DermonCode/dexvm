//! Live test for the M+ (mangaplus) keiyoushi extension: exercises the
//! kotlinx-serialization-protobuf decode path end-to-end against the real
//! M+ API. Requires DEXVM_LIVE=1 and network access.

use std::rc::Rc;

use dexvm::keiyoushi::{FilterState, HttpResp, Keiyoushi};

const APK: &str = "/tmp/opencode/mangaplus.apk";
const QUERY: &str = "one piece";

fn real_http(req: &dexvm::keiyoushi::HttpData) -> HttpResp {
    static AGENT: std::sync::OnceLock<ureq::Agent> = std::sync::OnceLock::new();
    let agent = AGENT.get_or_init(|| {
        ureq::Agent::config_builder()
            .user_agent(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) \
                 Chrome/126.0 Safari/537.36",
            )
            .timeout_global(Some(std::time::Duration::from_secs(30)))
            .http_status_as_error(false)
            .build()
            .into()
    });
    if req.method == "POST" {
        let mut rq = agent.post(&req.url);
        for (k, v) in &req.headers {
            rq = rq.header(k, v);
        }
        let r = rq.send(req.body.as_deref().unwrap_or(""));
        to_resp(r)
    } else {
        let mut rq = agent.get(&req.url);
        for (k, v) in &req.headers {
            rq = rq.header(k, v);
        }
        to_resp(rq.call())
    }
}

fn to_resp(result: Result<ureq::http::Response<ureq::Body>, ureq::Error>) -> HttpResp {
    match result {
        Ok(r) => {
            let code = r.status().as_u16() as i32;
            let bytes = r.into_body().read_to_vec().unwrap_or_default();
            HttpResp {
                code,
                message: "OK".into(),
                headers: Vec::new(),
                body: Some(bytes),
            }
        }
        Err(e) => HttpResp {
            code: 0,
            message: e.to_string(),
            headers: Vec::new(),
            body: None,
        },
    }
}

#[test]
fn mangaplus_search_protobuf() {
    if std::env::var("DEXVM_LIVE").is_err() {
        eprintln!("note: set DEXVM_LIVE=1");
        return;
    }

    let mut ext = Keiyoushi::open(APK).unwrap();
    ext.set_http_rc(Rc::new(real_http));

    let srcs = ext.sources().unwrap();
    assert!(!srcs.is_empty(), "no sources");
    let src = &srcs[0];

    let fl = ext.filters(src).unwrap_or_default();
    let states: Vec<FilterState> = fl
        .iter()
        .map(|f| FilterState {
            name: f.name.clone(),
            state: f.state,
        })
        .collect();

    // Phase 1: first open triggers the lazy language-list bootstrap
    // (background download of filters.json.zst). The returned filter list
    // is intentionally minimal.
    if let Err(e) = ext.search_coro(src, 1, QUERY, &states) {
        eprintln!(
            "phase-1 search note (bootstrap may be async): {}",
            ext.describe_error(&e)
        );
    }

    // Phase 2: fresh engine picks up the cached filters.json.zst and builds
    // the full filter list, whose Language filter instance drives the
    // protobuf decode path under test.
    let mut ext = Keiyoushi::open(APK).unwrap();
    ext.set_http_rc(Rc::new(real_http));

    let srcs2 = ext.sources().unwrap();
    let src = if srcs2.is_empty() { src } else { &srcs2[0] };

    let fl = ext.filters(src).unwrap_or_default();
    eprintln!("phase-2 filter count: {}", fl.len());
    let states: Vec<FilterState> = fl
        .iter()
        .map(|f| FilterState {
            name: f.name.clone(),
            state: f.state,
        })
        .collect();

    let found = ext
        .search_coro(src, 1, QUERY, &states)
        .unwrap_or_else(|e| panic!("search_coro failed: {}", ext.describe_error(&e)));

    eprintln!(
        "mangaplus search '{}' -> {} mangas (has_next={})",
        QUERY,
        found.mangas.len(),
        found.has_next
    );
    assert!(
        !found.mangas.is_empty(),
        "expected protobuf-decoded search results"
    );

    for m in found.mangas.iter().take(5) {
        eprintln!("  - {} ({})", m.title, m.url);
        assert!(!m.title.is_empty());
    }
}
