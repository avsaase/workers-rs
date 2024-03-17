use super::SomeSharedData;
use blake2::Blake2b512;
use blake2::Digest;
use serde::{Deserialize, Serialize};
use worker::kv;
use worker::{FormEntry, Response};
use worker::{Request, Result, RouteContext};

pub async fn handle_formdata_name(
    mut req: Request,
    _ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
    let form = req.form_data().await?;
    const NAME: &str = "name";
    let bad_request = Response::error("Bad Request", 400);

    if !form.has(NAME) {
        return bad_request;
    }

    let names: Vec<String> = form
        .get_all(NAME)
        .unwrap_or_default()
        .into_iter()
        .map(|entry| match entry {
            FormEntry::Field(s) => s,
            FormEntry::File(f) => f.name(),
        })
        .collect();
    if names.len() > 1 {
        return Response::from_json(&serde_json::json!({ "names": names }));
    }

    if let Some(value) = form.get(NAME) {
        match value {
            FormEntry::Field(v) => Response::from_json(&serde_json::json!({ NAME: v })),
            _ => bad_request,
        }
    } else {
        bad_request
    }
}

#[derive(Deserialize, Serialize)]
struct FileSize {
    name: String,
    size: u32,
}

pub async fn handle_formdata_file_size(
    mut req: Request,
    ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
    let form = req.form_data().await?;

    if let Some(entry) = form.get("file") {
        return match entry {
            FormEntry::File(file) => {
                let kv: kv::KvStore = ctx.kv("FILE_SIZES")?;

                // create a new FileSize record to store
                let b = file.bytes().await?;
                let record = FileSize {
                    name: file.name(),
                    size: b.len() as u32,
                };

                // hash the file, and use result as the key
                let mut hasher = Blake2b512::new();
                hasher.update(b);
                let hash = hasher.finalize();
                let key = hex::encode(&hash[..]);

                // serialize the record and put it into kv
                let val = serde_json::to_string(&record)?;
                kv.put(&key, val)?.execute().await?;

                // list the default number of keys from the namespace
                Response::from_json(&kv.list().execute().await?.keys)
            }
            _ => Response::error("Bad Request", 400),
        };
    }

    Response::error("Bad Request", 400)
}

pub async fn handle_formdata_file_size_hash(
    _req: Request,
    ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
    if let Some(hash) = ctx.param("hash") {
        let kv = ctx.kv("FILE_SIZES")?;
        return match kv.get(hash).json::<FileSize>().await? {
            Some(val) => Response::from_json(&val),
            None => Response::error("Not Found", 404),
        };
    }

    Response::error("Bad Request", 400)
}

pub async fn handle_is_secret(
    mut req: Request,
    ctx: RouteContext<SomeSharedData>,
) -> Result<Response> {
    let form = req.form_data().await?;
    if let Some(secret) = form.get("secret") {
        match secret {
            FormEntry::Field(name) => {
                let val = ctx.secret(&name)?;
                return Response::ok(val.to_string());
            }
            _ => return Response::error("Bad Request", 400),
        };
    }

    Response::error("Bad Request", 400)
}